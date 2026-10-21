//! [DOC: docs/diataxis/reference/game_flow.md]
//! The Job: the plan of one request, its generations, and the steps of a generation

use std::sync::Arc;

use tracing::instrument;

use crate::adapters::driven::storage::worlds::WorldBundle;
use crate::adapters::driven::storage::stories::STORY_LIMIT;
use crate::adapters::driven::storage::Storage;
use crate::application::prompt_preset_service::PromptPresetService;
use crate::application::agents::runner::AgentRunner;
use crate::application::generation::gate::{
    GameWriter, JobFailure, JobWriter, OptionsResult, Refusal, SavedText,
};
use crate::application::generation::job_tasks::JobTasks;
use crate::application::narration::narrator::{NarrationValues, Narrator};
use crate::application::prompting::token_budget::TokenBudget;
use crate::application::prompting::NpcContext;
use crate::domain::model::agent::{AgentSettings, HookInput, Kind, AGENT_HISTORY_WINDOW};
use crate::domain::model::character::NpcCard;
use crate::domain::model::game::{Game, GameId};
use crate::domain::model::generation::{GenerationOrder, JobPlan, JobRequest, OptionsRun, Place};
use crate::domain::model::message::SwipeAt;
use crate::domain::model::prompt_preset::PresetType;
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::generation_status::{GenerationFailure, Record, Step};
use crate::domain::model::state::message_types::StoryRow;
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
    /// no Input. Each step loads by id what it uses and returns its result; only
    /// ids, texts and step results move between steps. A fired Trigger gives the
    /// event order that follows.
    fn generate(
        &self,
        order: &GenerationOrder,
        input: Option<&str>,
        narration_saved: &mut bool,
    ) -> Result<Option<GenerationOrder>, JobFailure> {
        let kind = order.kind();
        let step = kind.step();

        self.run_step(step.clone(), Some(kind), || self.room_check(order))?;

        if let Some(text) = input {
            self.run_step(step.clone(), Some(kind), || self.save_input(order, text))?;
        }

        let text = self.run_step(step.clone(), Some(kind), || self.narrate(order, input))?;

        let saved = self.run_step(step, Some(kind), || self.save_text(order, text.clone()))?;
        let at = saved.at();
        *narration_saved |= kind.saves_a_narration();

        if !order.is_analysed() {
            return Ok(None);
        }

        let quantified = self.run_step(Step::Quantifying, Some(kind), || {
            self.analyse(order, at, &text)
        })?;
        self.run_step(Step::Quantifying, Some(kind), || {
            self.update_state(order, saved, &quantified)
        })
    }

    /// Every step goes through here, so a step cannot forget its status name. The
    /// Options run belongs to no generation, so it names no kind.
    #[instrument(
        skip(self, run),
        fields(game = %self.writer.game(), job = self.writer.job_id(), kind = ?kind, step = ?step)
    )]
    fn run_step<T>(
        &self,
        step: Step,
        kind: Option<Kind>,
        run: impl FnOnce() -> Result<T, JobFailure>,
    ) -> Result<T, JobFailure> {
        self.writer.step(step)?;
        run()
    }

    fn room_check(&self, order: &GenerationOrder) -> Result<(), JobFailure> {
        let bundle = self.world_bundle()?;
        let start = self.start_state(order)?;
        start
            .room(&bundle.map)
            .map(|_| ())
            .ok_or_else(|| Self::invalid_room(&start.movement.current_room_id))
    }

    fn save_input(&self, order: &GenerationOrder, text: &str) -> Result<(), JobFailure> {
        let start = self.start_state(order)?;
        self.writer.save_input(text, &start).map(|_| ())
    }

    fn narrate(&self, order: &GenerationOrder, input: Option<&str>) -> Result<String, JobFailure> {
        let game = self.game_row()?;
        let settings = Self::read(Record::Settings, self.storage.get_settings())?;
        let preset = Self::read(
            Record::Preset,
            self.presets
                .for_narration(&game, order.generation.preset_type()),
        )?;
        let bundle = self.world_bundle()?;
        let start = self.start_state(order)?;
        let room = start
            .room(&bundle.map)
            .ok_or_else(|| Self::invalid_room(&start.movement.current_room_id))?;
        let tail = self.tail()?;
        let history = self.window_before(
            match order.place {
                Place::NewMessage => None,
                Place::NewSwipeOnLast => Some(tail.last.id),
            },
            STORY_LIMIT,
        )?;
        let budget = Self::budget(&settings)?;

        let all_npcs: Vec<NpcCard> = bundle.npcs.values().cloned().collect();
        let values = NarrationValues {
            world: &bundle.world,
            narrative_perspective: game.narrative_perspective,
            narrative_tense: game.narrative_tense,
            room,
            npcs: NpcContext {
                all_npcs: &all_npcs,
                npcs_in_area: &start.scene.npcs_in_area,
            },
            persona: &bundle.persona,
            history: &history,
            input,
            tail: &tail,
        };

        let narrated = self.narrator.narrate(
            &order.generation,
            &values,
            &preset,
            &settings.response_length,
            budget,
        )?;
        Ok(narrated.text)
    }

    fn save_text(&self, order: &GenerationOrder, text: String) -> Result<SavedText, JobFailure> {
        let start = self.start_state(order)?;
        let tail = self.tail()?;
        self.writer.save_text(order, text, &start, &tail)
    }

    /// The Quantifier must not see the new text, so its window ends at the Message
    /// the text save made.
    fn analyse(
        &self,
        order: &GenerationOrder,
        saved: SwipeAt,
        text: &str,
    ) -> Result<QuantifierResult, JobFailure> {
        let settings = self.hook_settings(PresetType::Quantifier)?;
        let bundle = self.world_bundle()?;
        let start = self.start_state(order)?;
        let history = self.window_before(Some(saved.message), AGENT_HISTORY_WINDOW)?;
        let world = bundle.view(&start);
        let hook_input = HookInput {
            text: Some(text),
            story: &history,
            world: &world,
        };
        Ok(self.agent_runner.after_narration(&hook_input, &settings))
    }

    /// The rules start from the state of the text save. For an event, that state
    /// already marks the one-shot Trigger fired.
    fn update_state(
        &self,
        order: &GenerationOrder,
        saved: SavedText,
        quantified: &QuantifierResult,
    ) -> Result<Option<GenerationOrder>, JobFailure> {
        let at = saved.at();
        let bundle = self.world_bundle()?;
        let state = self.swipe_state_of(at, Record::SavedState)?;
        let outcome = game_rules::apply(state, quantified, &bundle.game_world(), order.kind())
            .map_err(|error| GenerationFailure::contradiction(error.to_string()))?;
        self.writer.save_state_update(saved, outcome.state)?;
        Ok(outcome
            .fired
            .map(|record| GenerationOrder::event_after(at, record)))
    }

    fn options_step(&self, run: OptionsRun) -> Result<(), JobFailure> {
        // The toggle decides the run that follows a Narration; `/options` asks for
        // the run itself.
        if run == OptionsRun::AfterNarration && !self.options_enabled()? {
            return Ok(());
        }
        self.run_step(Step::Options, None, || {
            let settings = self.hook_settings(PresetType::Options)?;
            let bundle = self.world_bundle()?;
            let tail = self.tail()?;
            let current = tail
                .current_state()
                .ok_or_else(|| GenerationFailure::missing(Record::CurrentState))?;
            // The run reads the four newest Messages, the Narration it serves among
            // them.
            let story = self.window_before(None, AGENT_HISTORY_WINDOW)?;
            let world = bundle.view(current);
            let hook_input = HookInput {
                text: None,
                story: &story,
                world: &world,
            };

            let outcome = match self.agent_runner.end_of_job(&hook_input, &settings) {
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
            self.writer.save_options(outcome)
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

    /// The room the state stands in is not on the map; no engine error lies behind
    /// the line.
    fn invalid_room(room_id: &str) -> JobFailure {
        GenerationFailure::invalid_room(room_id).into()
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
        self.swipe_state_of(order.start, Record::StartState)
    }

    fn swipe_state_of(&self, at: SwipeAt, record: Record) -> Result<SwipeState, JobFailure> {
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
