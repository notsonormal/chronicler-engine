//! [DOC: docs/diataxis/reference/game_flow.md]
//! The Job: the plan of one request, its generations, and the steps of a generation

use std::sync::Arc;

use tracing::instrument;

use crate::adapters::driven::storage::worlds::WorldBundle;
use crate::adapters::driven::storage::stories::STORY_LIMIT;
use crate::adapters::driven::storage::Storage;
use crate::application::prompt_preset_service::PromptPresetService;
use crate::application::agents::runner::AgentRunner;
use crate::application::generation::gate::{GameWriter, JobFailure, JobWriter, OptionsResult, Refusal};
use crate::application::generation::job_tasks::JobTasks;
use crate::application::narration::narrator::Narrator;
use crate::application::prompting::token_budget::TokenBudget;
use crate::application::prompting::{NpcContext, PromptContext};
use crate::domain::model::agent::{AgentSettings, HookInput, Kind, WorldView, STORY_WINDOW};
use crate::domain::model::character::NpcCard;
use crate::domain::model::game::{Game, GameId};
use crate::domain::model::generation::{
    Generation, GenerationOrder, JobPlan, JobRequest, OptionsRun, Place,
};
use crate::domain::model::map::Room;
use crate::domain::model::message::SwipeAt;
use crate::domain::model::prompt_preset::PresetType;
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::generation_status::{GenerationFailure, Record, Step};
use crate::domain::model::state::message_types::{MessageKind, MessageType, StoryRow};
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::tail::Tail;
use crate::domain::model::utils::game_rules;
use crate::error::EngineError;

#[derive(Clone)]
pub struct JobDeps {
    pub narrator: Arc<Narrator>,
    pub agent_runner: Arc<AgentRunner>,
    pub storage: Arc<Storage>,
    pub tasks: Arc<JobTasks>,
}

pub struct Job {
    writer: JobWriter,
    narrator: Arc<Narrator>,
    agent_runner: Arc<AgentRunner>,
    presets: PromptPresetService,
    storage: Arc<Storage>,
}

impl Job {
    #[instrument(
        skip(writer, deps, request),
        fields(
            game = %game,
            job = tracing::field::Empty,
            kind = tracing::field::Empty,
            step = tracing::field::Empty
        )
    )]
    pub fn start(
        writer: &GameWriter,
        deps: JobDeps,
        game: GameId,
        request: JobRequest,
    ) -> Result<(), Refusal> {
        let (job_writer, plan) = writer.accept(game, request)?;
        let span = tracing::Span::current();
        span.record("job", job_writer.job_id());
        span.record(
            "kind",
            tracing::field::debug(plan.first.as_ref().map(GenerationOrder::kind)),
        );
        span.record("step", tracing::field::debug(plan.first_step()));

        let JobDeps {
            narrator,
            agent_runner,
            storage,
            tasks,
        } = deps;
        let job = Self {
            writer: job_writer,
            narrator,
            agent_runner,
            presets: PromptPresetService::new(Arc::clone(&storage)),
            storage,
        };
        let handle = tokio::task::spawn_blocking(move || {
            let _entered = span.enter();
            job.run(plan);
        });
        tasks.watch(handle);
        Ok(())
    }

    /// A step that fails ends the Job, and every save unit that landed before it
    /// stays. The Options run follows a Narration that was saved, whatever failed
    /// after it except a moved entry.
    fn run(&self, plan: JobPlan) {
        let JobPlan {
            mut input,
            first,
            options,
        } = plan;
        let mut narration_saved = false;
        let mut failure = None;
        let mut next = first;
        while let Some(order) = next.take() {
            match self.generate(&order, input.take().as_deref(), &mut narration_saved) {
                Ok(event) => next = event,
                Err(error) => failure = Some(error.of_generation(order.kind())),
            }
        }
        let entry_moved = matches!(failure, Some(JobFailure::EntryMoved));
        if options.follows(narration_saved, entry_moved) {
            if let Err(error) = self.options_step(options) {
                match failure {
                    None => failure = Some(error),
                    Some(_) => tracing::warn!(
                        "job {}: the Options run after a failed step ended: {error:?}",
                        self.writer.job_id()
                    ),
                }
            }
        }
        self.end(failure);
    }

    /// The request's Input save runs after the room check, so an invalid room saves
    /// no Input. A fired Trigger gives the event order that follows.
    fn generate(
        &self,
        order: &GenerationOrder,
        input: Option<&str>,
        narration_saved: &mut bool,
    ) -> Result<Option<GenerationOrder>, JobFailure> {
        let kind = order.kind();
        let step = kind.step();
        let bundle = self.world_bundle()?;
        let start = self.start_state(order)?;

        self.run_step(step.clone(), kind, || self.room_check(&bundle, &start))?;

        if let Some(text) = input {
            let at = order.start;
            self.run_step(step.clone(), kind, || {
                self.writer.save_input(text, at).map(|_| ())
            })?;
        }

        let narrated = self.run_step(step.clone(), kind, || {
            self.narrate_step(order, &bundle, &start, input)
        })?;

        let saved = self.run_step(step, kind, || {
            self.writer.save_text(order, narrated.clone(), &start)
        })?;
        let at = saved.at();
        *narration_saved |= kind.saves_a_narration();

        if !order.is_analysed() {
            return Ok(None);
        }

        // The Quantifier must not see the new text, so the window ends at the Message
        // the text save made.
        let history = self.window_before(Some(at.message), STORY_WINDOW)?;
        let player_text = Self::player_text(input, &history);
        let quantified = self.run_step(Step::Quantifying, kind, || {
            self.quantify_step(&bundle, &start, &narrated, &history, &player_text)
        })?;

        let outcome = self.run_step(Step::Quantifying, kind, || {
            self.rules_step(kind, &bundle, at, &quantified)
        })?;
        self.run_step(Step::Quantifying, kind, || {
            self.writer.save_state_update(saved, outcome.state)
        })?;

        Ok(outcome
            .fired
            .map(|record| GenerationOrder::event_after(at, record)))
    }

    /// Every step goes through here, so a step cannot forget its status name.
    #[instrument(
        skip(self, run),
        fields(game = %self.writer.game(), job = self.writer.job_id(), kind = ?kind, step = ?step)
    )]
    fn run_step<T>(
        &self,
        step: Step,
        kind: Kind,
        run: impl FnOnce() -> Result<T, JobFailure>,
    ) -> Result<T, JobFailure> {
        self.writer.step(step)?;
        run()
    }

    fn room_check(&self, bundle: &WorldBundle, start: &SwipeState) -> Result<(), JobFailure> {
        self.resolve_room(bundle, start).map(|_| ())
    }

    fn resolve_room<'s>(
        &self,
        bundle: &'s WorldBundle,
        start: &'s SwipeState,
    ) -> Result<&'s Room, JobFailure> {
        start.room(&bundle.map).ok_or_else(|| {
            // The engine has no error behind this one: the room it stands in is
            // simply not on the map.
            GenerationFailure::invalid_room(start.movement.current_room_id.clone()).into()
        })
    }

    fn narrate_step(
        &self,
        order: &GenerationOrder,
        bundle: &WorldBundle,
        start: &SwipeState,
        input: Option<&str>,
    ) -> Result<String, JobFailure> {
        let game = self.game_row()?;
        let room = self.resolve_room(bundle, start)?;
        let before = self.place_before(order)?;
        let history = self.window_before(before, STORY_LIMIT)?;
        let user_message = self.user_message(order, input, &history)?;
        let preset_type = match order.kind() {
            Kind::Impersonate => PresetType::Impersonate,
            _ => PresetType::System,
        };
        let preset = Self::read(
            Record::Preset,
            self.presets.for_narration(&game, preset_type),
        )?;
        let settings = Self::read(Record::Settings, self.storage.get_settings())?;
        let budget = Self::budget(&settings)?;

        let all_npcs: Vec<NpcCard> = bundle.npcs.values().cloned().collect();
        let context = PromptContext::new(
            &bundle.world,
            game.narrative_perspective,
            game.narrative_tense,
            room,
            NpcContext {
                all_npcs: &all_npcs,
                npcs_in_area: &start.scene.npcs_in_area,
            },
            &bundle.persona,
            &user_message,
            &history,
        );
        let context = match &order.generation {
            Generation::Impersonate { .. } => context.with_impersonate(true),
            Generation::Narration { guide } => context.with_guide(guide.clone()),
            Generation::UserRegen | Generation::Event(_) => context,
        };

        match self.narrator.narrate(
            order.kind().role(),
            &context,
            &preset,
            &settings.response_length,
            budget,
        ) {
            Ok(narrated) => Ok(narrated.text),
            Err(failure) => Err(failure.into()),
        }
    }

    fn quantify_step(
        &self,
        bundle: &WorldBundle,
        start: &SwipeState,
        text: &str,
        history: &[StoryRow],
        player_text: &str,
    ) -> Result<QuantifierResult, JobFailure> {
        let world = WorldView {
            room: start.room(&bundle.map),
            map: &bundle.map,
            persona: &bundle.persona,
            characters: &bundle.npcs,
            present: &start.scene.npcs_in_area,
        };
        let input = HookInput {
            text: Some(text),
            player_text,
            story: history,
            world: &world,
        };
        Ok(self
            .agent_runner
            .after_narration(&input, &self.quantifier_settings()?))
    }

    /// The rules start from the state of the text save. For an event, that state
    /// already marks the one-shot Trigger fired.
    fn rules_step(
        &self,
        kind: Kind,
        bundle: &WorldBundle,
        at: SwipeAt,
        quantified: &QuantifierResult,
    ) -> Result<game_rules::RulesOutcome, JobFailure> {
        let state = self.swipe_state_at(at, Record::SavedState)?;
        game_rules::apply(state, quantified, &bundle.game_world(), kind)
            .map_err(|error| GenerationFailure::contradiction(error.to_string()).into())
    }

    fn options_step(&self, run: OptionsRun) -> Result<(), JobFailure> {
        // The toggle decides the run that follows a Narration; `/options` asks for
        // the run itself.
        if run == OptionsRun::AfterNarration && !self.options_enabled()? {
            return Ok(());
        }
        let tail = self.tail()?;
        let kind = Self::scene_kind(&tail);
        self.run_step(Step::Options, kind, || {
            let bundle = self.world_bundle()?;
            let start = tail
                .current_state()
                .cloned()
                .ok_or_else(|| GenerationFailure::missing(Record::CurrentState))?;
            // The run reads the four newest Messages, the Narration it serves among
            // them.
            let story = self.window_before(None, STORY_WINDOW)?;
            let world = WorldView {
                room: start.room(&bundle.map),
                map: &bundle.map,
                persona: &bundle.persona,
                characters: &bundle.npcs,
                present: &start.scene.npcs_in_area,
            };
            let input = HookInput {
                text: None,
                player_text: "",
                story: &story,
                world: &world,
            };

            let outcome = match self
                .agent_runner
                .end_of_job(&input, &self.options_settings()?)
            {
                Ok(Some(offered)) => OptionsResult::Offered(offered),
                Ok(None) => OptionsResult::Unavailable {
                    reason: "Options agent is not available".to_string(),
                    raw: "no Options Agent is registered".to_string(),
                },
                Err(error) => OptionsResult::Unavailable {
                    reason: "Options generation failed".to_string(),
                    raw: error.to_string(),
                },
            };
            self.writer
                .save_options(outcome, run == OptionsRun::AfterNarration)
        })
    }

    fn scene_kind(tail: &Tail) -> Kind {
        match tail.last.kind {
            MessageKind::EventNarration(_) => Kind::Event,
            _ => Kind::Narration,
        }
    }

    /// The Player's direction for an Impersonate, the Input's own text for a user
    /// regen, the Trigger's continuation prompt for an event, and otherwise the Input
    /// the new text follows.
    fn user_message(
        &self,
        order: &GenerationOrder,
        input: Option<&str>,
        history: &[StoryRow],
    ) -> Result<String, JobFailure> {
        Ok(match &order.generation {
            Generation::Impersonate { direction } => direction.clone().unwrap_or_default(),
            Generation::UserRegen => self.tail()?.last.text().to_string(),
            Generation::Event(record) => {
                Self::continuation_user_message(history, &record.direction)
            }
            Generation::Narration { .. } => match input {
                Some(text) if !text.is_empty() => text.to_string(),
                _ => Self::previous_input(history)
                    .unwrap_or_default()
                    .to_string(),
            },
        })
    }

    fn continuation_user_message(history: &[StoryRow], direction: &str) -> String {
        let previous = history
            .last()
            .map(|entry| entry.text.as_str())
            .unwrap_or("");
        format!(
            "Previous narration:\n{previous}\n\nTrigger event: {direction}\n\n\
             Continue the scene naturally, incorporating the trigger event into the narrative. \
             Do NOT repeat or contradict what was already described. Build naturally on the existing scene."
        )
    }

    fn player_text(input: Option<&str>, history: &[StoryRow]) -> String {
        match input {
            Some(text) if !text.is_empty() => text.to_string(),
            _ => history
                .last()
                .map(|entry| entry.text.clone())
                .unwrap_or_default(),
        }
    }

    fn previous_input(history: &[StoryRow]) -> Option<&str> {
        history
            .last()
            .filter(|entry| matches!(entry.message_type, MessageType::Input))
            .map(|entry| entry.text.as_str())
    }

    /// A new Message follows the whole story; a Retry replaces the last Message, so
    /// its window ends before it.
    fn place_before(&self, order: &GenerationOrder) -> Result<Option<u64>, JobFailure> {
        Ok(match order.place {
            Place::NewMessage => None,
            Place::NewSwipeOnLast => Some(self.tail()?.last.id),
        })
    }

    fn window_before(
        &self,
        before: Option<u64>,
        limit: usize,
    ) -> Result<Vec<StoryRow>, JobFailure> {
        Self::read(
            Record::Story,
            self.storage
                .stories()
                .entries(self.writer.game(), before, limit),
        )
    }

    fn end(&self, failure: Option<JobFailure>) {
        match failure {
            None => {}
            Some(JobFailure::Line(line)) => {
                tracing::warn!("job {}: {line:?}", self.writer.job_id());
                if let Err(refused) = self.writer.save_failure(line) {
                    tracing::warn!(
                        "job {}: the failure line was refused: {refused:?}",
                        self.writer.job_id()
                    );
                }
            }
            Some(JobFailure::EntryMoved) => tracing::warn!(
                "job {}: the entry moved on; the Job ends",
                self.writer.job_id()
            ),
        }
    }

    /// A read that fails shows the line of its record, with the error in the
    /// disclosure.
    fn read<T>(record: Record, result: Result<T, EngineError>) -> Result<T, JobFailure> {
        result.map_err(|error| GenerationFailure::unreadable(record, &error).into())
    }

    /// A read that finds no row shows the same line as one that fails.
    fn present<T>(record: Record, result: Result<Option<T>, EngineError>) -> Result<T, JobFailure> {
        Self::read(record, result)?.ok_or_else(|| GenerationFailure::missing(record).into())
    }

    fn world_bundle(&self) -> Result<WorldBundle, JobFailure> {
        Self::read(
            Record::World,
            self.storage.world_bundle_for(self.writer.game().0),
        )
    }

    fn game_row(&self) -> Result<Game, JobFailure> {
        Self::read(
            Record::Game,
            self.storage.require_game(self.writer.game().0),
        )
    }

    fn tail(&self) -> Result<Tail, JobFailure> {
        Self::present(
            Record::Story,
            self.storage.stories().tail(self.writer.game()),
        )
    }

    fn start_state(&self, order: &GenerationOrder) -> Result<SwipeState, JobFailure> {
        self.swipe_state_at(order.start, Record::StartState)
    }

    fn swipe_state_at(&self, at: SwipeAt, record: Record) -> Result<SwipeState, JobFailure> {
        Self::present(
            record,
            self.storage.stories().swipe_state(self.writer.game(), at),
        )
    }

    fn budget(settings: &AppSettings) -> Result<TokenBudget, JobFailure> {
        let connection = Self::read(Record::Settings, settings.narration_connection())?;
        Ok(TokenBudget {
            max_context_tokens: connection.resolve_max_context_tokens(),
            max_tokens: connection.max_tokens,
        })
    }

    fn options_enabled(&self) -> Result<bool, JobFailure> {
        Ok(self.game_row()?.options_always_on)
    }

    fn quantifier_settings(&self) -> Result<AgentSettings, JobFailure> {
        self.hook_settings(PresetType::Quantifier)
    }

    fn options_settings(&self) -> Result<AgentSettings, JobFailure> {
        self.hook_settings(PresetType::Options)
    }

    fn hook_settings(&self, preset_type: PresetType) -> Result<AgentSettings, JobFailure> {
        let game = self.game_row()?;
        Ok(AgentSettings {
            preset: Some(Self::read(
                Record::Preset,
                self.presets.hook_text(&game, preset_type),
            )?),
            max_tokens: None,
        })
    }
}
