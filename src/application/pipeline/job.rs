//! [DOC: docs/diataxis/reference/game_flow.md]
//! The Job: the plan of one request, its generations, and the steps of a generation

use std::sync::Arc;

use tracing::instrument;

use crate::adapters::driven::storage::worlds::WorldBundle;
use crate::adapters::driven::storage::stories::STORY_LIMIT;
use crate::adapters::driven::storage::Storage;
use crate::application::agents::runner::AgentRunner;
use crate::application::generation::gate::{
    GameWriter, JobPlan, JobRequest, JobWriter, OptionsResult, OptionsRun, Refusal, Stopped,
};
use crate::application::generation::job_tasks::JobTasks;
use crate::application::narration::narrator::Narrator;
use crate::application::prompting::token_budget::TokenBudget;
use crate::application::prompting::{NpcContext, PromptContext};
use crate::domain::model::agent::{AgentSettings, HookInput, Kind, WorldView, STORY_WINDOW};
use crate::domain::model::character::NpcCard;
use crate::domain::model::game::{Game, GameId};
use crate::domain::model::generation::{GenerationOrder, Place};
use crate::domain::model::map::{MapDef, Room};
use crate::domain::model::message::{StoredInputs, SwipeAt};
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::quantifier::{MovementParseResult, QuantifierResult};
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::generation_status::{GenerationFailure, GenerationFailureKind, Step};
use crate::domain::model::state::message_types::{MessageKind, MessageType, StoryRow};
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::state::trigger_context::EventRecord;
use crate::domain::model::tail::Tail;
use crate::domain::model::utils::game_rules::{apply_game_rules, GameRuleRequest};
use crate::error::EngineError;

#[derive(Clone)]
pub struct JobDeps {
    pub narrator: Arc<Narrator>,
    pub agent_runner: Arc<AgentRunner>,
    pub storage: Arc<Storage>,
    pub tasks: Arc<JobTasks>,
}

struct Generated {
    narration: bool,
    at: SwipeAt,
    fired: Option<EventRecord>,
}

struct RuleOutcome {
    fired: Option<EventRecord>,
    room_report: Option<String>,
}

#[derive(Debug)]
enum Failure {
    /// A save unit refused: the Job ends with no further write.
    Stopped(Stopped),
    /// The generation failed, and its line is the Job's to write.
    Line(GenerationFailure),
}

impl Failure {
    fn missing(what: &str) -> Self {
        Self::Line(GenerationFailure::missing_record(what, None))
    }

    /// A failed event generation reports the event line, with the line of the
    /// failure under it in the raw text.
    fn event_narration_failed(self) -> Self {
        match self {
            Self::Line(failure) => Self::Line(GenerationFailure::new(
                GenerationFailureKind::EventNarrationFailed,
                failure.line(),
            )),
            Self::Stopped(stopped) => Self::Stopped(stopped),
        }
    }
}

impl From<Stopped> for Failure {
    fn from(stopped: Stopped) -> Self {
        Self::Stopped(stopped)
    }
}

impl From<EngineError> for Failure {
    fn from(error: EngineError) -> Self {
        Self::missing(&error.to_string())
    }
}

/// A Narration saved before the failure still gets its Options run.
#[derive(Debug)]
struct Ended {
    failure: Failure,
    narration_saved: bool,
}

impl From<Failure> for Ended {
    fn from(failure: Failure) -> Self {
        Self {
            failure,
            narration_saved: false,
        }
    }
}

impl Ended {
    fn after_text_save(failure: Failure, kind: Kind) -> Self {
        Self {
            failure,
            narration_saved: kind.saves_a_narration(),
        }
    }

    fn into_event_failure(self) -> Self {
        Self {
            failure: self.failure.event_narration_failed(),
            narration_saved: self.narration_saved,
        }
    }
}

pub struct Job {
    writer: JobWriter,
    narrator: Arc<Narrator>,
    agent_runner: Arc<AgentRunner>,
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
            tracing::field::debug(plan.first.as_ref().map(|order| order.kind)),
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
            storage,
        };
        let handle = tokio::task::spawn_blocking(move || {
            let _entered = span.enter();
            job.run(plan);
        });
        tasks.watch(handle);
        Ok(())
    }

    /// A step that fails ends the Job, and every save unit that landed before it stays.
    fn run(&self, plan: JobPlan) {
        let JobPlan {
            input,
            first,
            options,
        } = plan;
        let mut saved_a_narration = false;
        if let Some(order) = first {
            let generated = match self.generate(&order, input.as_deref()) {
                Ok(generated) => generated,
                // A failed event generation names the event wherever it runs: the
                // second generation of a Narration's Job, or the redo of an event
                // Narration.
                Err(ended) if order.kind == Kind::Event => {
                    return self.failed(ended.into_event_failure(), options);
                }
                Err(ended) => return self.failed(ended, options),
            };
            saved_a_narration = generated.narration;
            if let Some(record) = generated.fired.clone() {
                let event = Self::event_order(record, generated.at);
                match self.generate(&event, None) {
                    Ok(generated) => saved_a_narration = generated.narration,
                    // The Narration of the first generation already landed, so the
                    // event generation's failure still leaves one for the Options run.
                    Err(mut ended) => {
                        ended.narration_saved |= saved_a_narration;
                        return self.failed(ended.into_event_failure(), options);
                    }
                }
            }
        }
        if options == OptionsRun::OnDemand || saved_a_narration {
            if let Err(failure) = self.options_step(options, saved_a_narration) {
                self.end(failure);
            }
        }
    }

    fn failed(&self, ended: Ended, options: OptionsRun) {
        let stopped = matches!(ended.failure, Failure::Stopped(_));
        if ended.narration_saved && !stopped && options == OptionsRun::AfterNarration {
            if let Err(failure) = self.options_step(options, true) {
                tracing::warn!(
                    "job {}: the Options run after a failed step ended: {failure:?}",
                    self.writer.job_id()
                );
            }
        }
        self.end(ended.failure);
    }

    /// The request's Input save runs after the room check, so an invalid room saves
    /// no Input.
    fn generate(&self, order: &GenerationOrder, input: Option<&str>) -> Result<Generated, Ended> {
        let kind = order.kind;
        let step = kind.step();
        let bundle = self.world_bundle()?;
        let start = self.start_state(order)?;

        self.run_step(step.clone(), kind, || self.room_check(&bundle, &start))?;

        if let Some(text) = input {
            let at = order.start;
            self.run_step(step.clone(), kind, || {
                self.writer
                    .save_input(text, at)
                    .map(|_| ())
                    .map_err(Failure::from)
            })?;
        }

        let narrated = self.run_step(step.clone(), kind, || {
            self.narrate_step(order, &bundle, &start, input)
        })?;

        let pre = match kind {
            Kind::UserRegen => start.clone(),
            _ => start.pre_analysis(order.event.as_ref()),
        };
        let saved = self.run_step(step, kind, || {
            self.writer
                .save_text(order, narrated.clone(), pre)
                .map_err(Failure::from)
        })?;
        let at = saved.at();

        if kind == Kind::UserRegen {
            return Ok(Generated {
                narration: false,
                at,
                fired: None,
            });
        }

        // The analysis must not see the new text, so the window ends at the Message
        // the text save made.
        let history = self.window_before(Some(at.message), STORY_WINDOW)?;
        let player_text = Self::player_text(input, &history);
        let analysis = match self.run_step(Step::Quantifying, kind, || {
            self.analyse_step(order, &bundle, &start, &narrated, &history, &player_text)
        }) {
            Ok(analysis) => analysis,
            Err(failure) => return Err(Ended::after_text_save(failure, kind)),
        };

        let mut state = start.clone();
        let rules = match self.run_step(Step::Quantifying, kind, || {
            self.apply_rules(order, &bundle, &mut state, &analysis, &narrated)
        }) {
            Ok(rules) => rules,
            Err(failure) => return Err(Ended::after_text_save(failure, kind)),
        };
        let RuleOutcome { fired, room_report } = rules;
        match self.run_step(Step::Quantifying, kind, || {
            self.writer
                .save_state_update(saved, state, room_report)
                .map_err(Failure::from)
        }) {
            Ok(()) => {}
            Err(failure) => return Err(Ended::after_text_save(failure, kind)),
        }

        Ok(Generated {
            narration: kind.saves_a_narration(),
            at,
            fired,
        })
    }

    fn event_order(record: EventRecord, start: SwipeAt) -> GenerationOrder {
        GenerationOrder {
            kind: Kind::Event,
            place: Place::NewMessage,
            start,
            inputs: StoredInputs::default(),
            event: Some(record),
        }
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
        run: impl FnOnce() -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        self.writer.step(step).map_err(Failure::from)?;
        run()
    }

    fn room_check(&self, bundle: &WorldBundle, start: &SwipeState) -> Result<(), Failure> {
        self.resolve_room(bundle, start).map(|_| ())
    }

    fn resolve_room<'s>(
        &self,
        bundle: &'s WorldBundle,
        start: &'s SwipeState,
    ) -> Result<&'s Room, Failure> {
        Self::room_of(start, &bundle.map).ok_or_else(|| {
            // The engine has no error behind this one: the room it stands in is
            // simply not on the map.
            Failure::Line(GenerationFailure::missing_room(
                Self::room_name(bundle, &start.movement.current_room_id),
                None,
            ))
        })
    }

    fn narrate_step(
        &self,
        order: &GenerationOrder,
        bundle: &WorldBundle,
        start: &SwipeState,
        input: Option<&str>,
    ) -> Result<String, Failure> {
        let game = self.game_row()?;
        let room = self.resolve_room(bundle, start)?;
        let before = self.place_before(order)?;
        let history = self.window_before(before, STORY_LIMIT)?;
        let user_message = self.user_message(order, input, &history)?;
        let preset_id = match order.kind {
            Kind::Impersonate => game.active_preset_id(PresetType::Impersonate).to_string(),
            _ => game.active_preset_id(PresetType::System).to_string(),
        };
        let (preset, response_length) = self.preset(&preset_id)?;
        let budget = self.budget()?;

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
        let context = match order.kind {
            Kind::Impersonate => context.with_impersonate(true),
            _ => context.with_guide(order.inputs.guide_text.clone()),
        };

        match self.narrator.narrate(
            order.kind.role(),
            &context,
            &preset,
            &response_length,
            budget,
        ) {
            Ok(narrated) => Ok(narrated.text),
            Err(failure) => Err(Failure::Line(failure)),
        }
    }

    fn analyse_step(
        &self,
        order: &GenerationOrder,
        bundle: &WorldBundle,
        start: &SwipeState,
        text: &str,
        history: &[StoryRow],
        player_text: &str,
    ) -> Result<QuantifierResult, Failure> {
        let world = WorldView {
            room: Self::room_of(start, &bundle.map),
            map: &bundle.map,
            persona: &bundle.persona,
            characters: &bundle.npcs,
            present: &start.scene.npcs_in_area,
        };
        let input = HookInput {
            kind: order.kind,
            text: Some(text),
            player_text,
            story: history,
            world: &world,
            event: order.event.as_ref(),
        };
        Ok(self
            .agent_runner
            .analysis(&input, &self.analysis_settings()?))
    }

    /// An event generation writes no room report, so the opening's room stays the one
    /// the text save wrote.
    fn apply_rules(
        &self,
        order: &GenerationOrder,
        bundle: &WorldBundle,
        state: &mut SwipeState,
        analysis: &QuantifierResult,
        text: &str,
    ) -> Result<RuleOutcome, Failure> {
        if order.kind == Kind::Event {
            let record = order
                .event
                .as_ref()
                .ok_or_else(|| Failure::missing("the event record"))?;
            // An event generation moves nobody and fires no Trigger: the movement
            // is taken out, and the Trigger the rules report is ignored.
            let still = QuantifierResult {
                movement: MovementParseResult::default(),
                ..analysis.clone()
            };
            apply_game_rules(
                state,
                GameRuleRequest::Analysis {
                    quantifier_result: &still,
                    persona_name: &bundle.persona.sheet.name,
                },
                &bundle.map,
                &bundle.npcs,
            )
            .map_err(Self::contradiction)?;
            apply_game_rules(
                state,
                GameRuleRequest::TriggerNarration {
                    trigger: &record.trigger,
                    text,
                },
                &bundle.map,
                &bundle.npcs,
            )
            .map_err(Self::contradiction)?;
            state.trigger_context = None;
            return Ok(RuleOutcome {
                fired: None,
                room_report: None,
            });
        }

        let outcome = apply_game_rules(
            state,
            GameRuleRequest::Analysis {
                quantifier_result: analysis,
                persona_name: &bundle.persona.sheet.name,
            },
            &bundle.map,
            &bundle.npcs,
        )
        .map_err(Self::contradiction)?;
        let fired = outcome.trigger_match.as_ref().map(EventRecord::from);
        // The Swipe an analysed generation leaves holds the Trigger that fired, so
        // a Retrigger is allowed on it.
        state.trigger_context = fired.as_ref().map(|record| record.trigger.clone());
        let room_report = match &outcome.arrived_room_name {
            Some(room_name) => Some(room_name.clone()),
            None if self.is_the_opening(order)? => {
                Some(Self::room_name(bundle, &state.movement.current_room_id))
            }
            None => None,
        };
        Ok(RuleOutcome { fired, room_report })
    }

    fn options_step(&self, run: OptionsRun, after_narration: bool) -> Result<(), Failure> {
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
                .ok_or_else(|| Failure::missing("the current state"))?;
            // The run reads the four newest Messages, the Narration it serves among
            // them.
            let story = self.window_before(None, STORY_WINDOW)?;
            let world = WorldView {
                room: Self::room_of(&start, &bundle.map),
                map: &bundle.map,
                persona: &bundle.persona,
                characters: &bundle.npcs,
                present: &start.scene.npcs_in_area,
            };
            let input = HookInput {
                kind,
                text: None,
                player_text: "",
                story: &story,
                world: &world,
                event: None,
            };

            let outcome = match self.agent_runner.options(&input, &self.options_settings()?) {
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
                .save_options(outcome, after_narration)
                .map_err(Failure::from)
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
    ) -> Result<String, Failure> {
        Ok(match order.kind {
            Kind::Impersonate => order
                .inputs
                .impersonate_direction
                .clone()
                .unwrap_or_default(),
            Kind::UserRegen => self.tail()?.last.text().to_string(),
            Kind::Event => {
                let record = order
                    .event
                    .as_ref()
                    .ok_or_else(|| Failure::missing("the event record"))?;
                Self::continuation_user_message(history, &record.direction)
            }
            Kind::Narration => match input {
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
    fn place_before(&self, order: &GenerationOrder) -> Result<Option<u64>, Failure> {
        Ok(match order.place {
            Place::NewMessage => None,
            Place::NewSwipeOnLast => Some(self.tail()?.last.id),
        })
    }

    fn window_before(&self, before: Option<u64>, limit: usize) -> Result<Vec<StoryRow>, Failure> {
        self.storage
            .stories()
            .entries(self.writer.game(), before, limit)
            .map_err(Failure::from)
    }

    fn is_the_opening(&self, order: &GenerationOrder) -> Result<bool, Failure> {
        if order.place != Place::NewSwipeOnLast {
            return Ok(false);
        }
        Ok(self.tail()?.count == 1)
    }

    fn end(&self, failure: Failure) {
        match failure {
            Failure::Line(failure) => {
                tracing::warn!("job {}: {failure:?}", self.writer.job_id());
                if let Err(stopped) = self.writer.save_failure(failure) {
                    self.log_refusal(&stopped);
                }
            }
            Failure::Stopped(Stopped::SaveFailed(error)) => {
                tracing::warn!("job {}: save failed: {error}", self.writer.job_id());
                let line =
                    GenerationFailure::new(GenerationFailureKind::SaveFailed, error.to_string());
                if let Err(stopped) = self.writer.save_failure(line) {
                    self.log_refusal(&stopped);
                }
            }
            Failure::Stopped(Stopped::EntryMoved) => {
                tracing::warn!(
                    "job {}: the entry moved on; the Job ends",
                    self.writer.job_id()
                );
            }
        }
    }

    fn log_refusal(&self, stopped: &Stopped) {
        match stopped {
            Stopped::EntryMoved => tracing::warn!(
                "job {}: the entry moved on; the write is refused",
                self.writer.job_id()
            ),
            Stopped::SaveFailed(error) => {
                tracing::warn!("job {}: the write failed: {error}", self.writer.job_id())
            }
        }
    }

    fn contradiction(error: EngineError) -> Failure {
        Failure::Line(GenerationFailure::new(
            GenerationFailureKind::Contradiction,
            error.to_string(),
        ))
    }

    fn room_of<'s>(state: &'s SwipeState, map: &'s MapDef) -> Option<&'s Room> {
        map.get_room_by_id(&state.movement.current_room_id)
            .or_else(|| {
                state
                    .movement
                    .dynamic_rooms
                    .get(&state.movement.current_room_id)
            })
    }

    fn room_name(bundle: &WorldBundle, room_id: &str) -> String {
        bundle
            .map
            .get_room_by_id(room_id)
            .map(|room| room.name.clone())
            .unwrap_or_else(|| room_id.to_string())
    }

    fn world_bundle(&self) -> Result<WorldBundle, Failure> {
        self.storage
            .world_bundle_for(self.writer.game().0)
            .map_err(Failure::from)
    }

    fn game_row(&self) -> Result<Game, Failure> {
        self.storage
            .require_game(self.writer.game().0)
            .map_err(Failure::from)
    }

    fn tail(&self) -> Result<Tail, Failure> {
        self.storage
            .stories()
            .tail(self.writer.game())
            .map_err(Failure::from)?
            .ok_or_else(|| Failure::missing("the story"))
    }

    fn start_state(&self, order: &GenerationOrder) -> Result<SwipeState, Failure> {
        self.storage
            .stories()
            .swipe_state(self.writer.game(), order.start)
            .map_err(Failure::from)?
            .ok_or_else(|| Failure::missing("the start state"))
    }

    fn preset(&self, preset_id: &str) -> Result<(PromptPreset, String), Failure> {
        let response_length = self
            .storage
            .get_settings()
            .map_err(Failure::from)?
            .response_length
            .clone();
        match self.storage.get_preset(preset_id) {
            Ok(Some(preset)) => Ok((preset, response_length)),
            Ok(None) => Err(Failure::Line(GenerationFailure::from_engine_error(
                &EngineError::PresetNotFound(preset_id.to_string()),
            ))),
            Err(error) => Err(Failure::from(error)),
        }
    }

    fn budget(&self) -> Result<TokenBudget, Failure> {
        let settings = self.storage.get_settings().map_err(Failure::from)?;
        let connection = settings.narration_connection().map_err(Failure::from)?;
        Ok(TokenBudget {
            max_context_tokens: connection.resolve_max_context_tokens(),
            max_tokens: connection.max_tokens,
        })
    }

    fn options_enabled(&self) -> Result<bool, Failure> {
        Ok(self.game_row()?.options_always_on)
    }

    fn analysis_settings(&self) -> Result<AgentSettings, Failure> {
        Ok(AgentSettings {
            preset: self.preset_text(Storage::active_quantifier_preset_id)?,
            max_tokens: None,
        })
    }

    fn options_settings(&self) -> Result<AgentSettings, Failure> {
        Ok(AgentSettings {
            preset: self.preset_text(Storage::active_options_preset_id)?,
            max_tokens: None,
        })
    }

    /// The preset's assembled text, or `None` when the Game's preset row is gone.
    /// A read failure is the Job's failure, not a silent no-preset.
    fn preset_text(
        &self,
        pick: fn(&Storage, GameId, &AppSettings) -> String,
    ) -> Result<Option<String>, Failure> {
        let settings = self.storage.get_settings().map_err(Failure::from)?;
        let preset_id = pick(&self.storage, self.writer.game(), &settings);
        match self.storage.get_preset(&preset_id) {
            Ok(Some(preset)) => Ok(Some(preset.assemble_text(&[], None, None))),
            Ok(None) => Ok(None),
            Err(error) => Err(Failure::from(error)),
        }
    }
}
