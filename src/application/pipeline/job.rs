//! [DOC: docs/diataxis/reference/game_flow.md]
//! The Job: the plan of one request, its generations, and the steps of a generation

use std::sync::Arc;

use tracing::instrument;

use crate::adapters::driven::storage::worlds::WorldBundle;
use crate::adapters::driven::storage::stories::STORY_LIMIT;
use crate::adapters::driven::storage::Storage;
use crate::application::prompt_preset_service::PromptPresetService;
use crate::application::agents::runner::AgentRunner;
use crate::application::agents::trait_def::BeforeNarrationAgent;
use crate::application::generation::gate::{
    AcceptedJob, GameWriter, JobFailure, JobWriter, OptionsResult, NotAccepted, SavedText,
};
use crate::application::autoplay::pause::AutoplayPause;
use crate::application::generation::job_tasks::JobTasks;
use crate::application::narration::narrator::{NarrationValues, Narrator};
use crate::application::prompting::token_budget::{
    TokenBudget, MEMORY_WRITER_ANSWER_TOKENS, MEMORY_WRITER_CALL_TOKENS,
};
use crate::application::prompting::CharacterContext;
use crate::domain::model::agent::{
    AgentSettings, Cast, HookInput, Kind, StoryMemoryHookInput, AGENT_HISTORY_WINDOW,
};
use crate::domain::model::character::CharacterCard;
use crate::domain::model::game::{Game, GameId};
use crate::domain::model::generation_parameters::CallSettings;
use crate::domain::model::generation::{GenerationOrder, JobPlan, JobRequest, OptionsRun, Place};
use crate::domain::model::message::SwipeAt;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::generation_status::{GenerationFailure, Record, Step};
use crate::domain::model::state::message_types::StoryRow;
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::memory_entry::{MemoryEntries, MemoryEntry};
use crate::domain::model::tail::Tail;
use crate::domain::model::utils::game_rules;
use crate::error::EngineError;

#[derive(Clone)]
pub struct JobDeps {
    pub narrator: Arc<Narrator>,
    pub agent_runner: Arc<AgentRunner>,
    /// Outside the agent config: no setting turns the story memory off.
    pub story_memory_writer: Arc<dyn BeforeNarrationAgent>,
    pub storage: Arc<Storage>,
    pub tasks: Arc<JobTasks>,
    pub pause: Arc<AutoplayPause>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobEnd {
    /// Every part ran, and the Job wrote no failure line.
    Done,
    /// A part failed; the Job tried to write its failure line.
    Failed,
    /// A stop, a reset or a delete ended the Job: its writes landed nowhere.
    EntryMoved,
}

struct CallPlan {
    settings: CallSettings,
    budget: TokenBudget,
}

pub struct Job {
    writer: JobWriter,
    narrator: Arc<Narrator>,
    agent_runner: Arc<AgentRunner>,
    story_memory_writer: Arc<dyn BeforeNarrationAgent>,
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
    ) -> Result<(), NotAccepted> {
        let AcceptedJob {
            writer: job_writer,
            plan,
        } = writer.accept(game, request)?;
        Self::record_on_span(&job_writer, &plan);
        let span = tracing::Span::current();
        let job = Self::new(job_writer, &deps);
        let handle = tokio::task::spawn_blocking(move || {
            let _entered = span.enter();
            job.run(plan);
        });
        deps.tasks.watch(handle);
        Ok(())
    }

    /// Runs on the calling thread, so the run's loop starts no second Job before
    /// this one ends.
    #[instrument(
        skip(accepted, deps),
        fields(
            game = %accepted.writer.game(),
            job = tracing::field::Empty,
            kind = tracing::field::Empty,
            step = tracing::field::Empty
        )
    )]
    pub fn run_accepted(accepted: AcceptedJob, deps: &JobDeps) -> JobEnd {
        Self::record_on_span(&accepted.writer, &accepted.plan);
        let job = Self::new(accepted.writer, deps);
        job.run(accepted.plan)
    }

    fn new(writer: JobWriter, deps: &JobDeps) -> Self {
        Self {
            writer,
            narrator: Arc::clone(&deps.narrator),
            agent_runner: Arc::clone(&deps.agent_runner),
            story_memory_writer: Arc::clone(&deps.story_memory_writer),
            presets: PromptPresetService::new(Arc::clone(&deps.storage)),
            storage: Arc::clone(&deps.storage),
        }
    }

    fn record_on_span(writer: &JobWriter, plan: &JobPlan) {
        let span = tracing::Span::current();
        span.record("job", writer.job_id());
        span.record(
            "kind",
            tracing::field::debug(plan.first.as_ref().map(GenerationOrder::kind)),
        );
        span.record("step", tracing::field::debug(plan.first_step()));
    }

    /// A step that fails ends the Job, and every transaction that landed before it
    /// stays. The Options run follows a Narration that was saved, whatever failed
    /// after it except a moved entry.
    fn run(&self, plan: JobPlan) -> JobEnd {
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
        self.end(failure)
    }

    /// The request's Input save runs after the room check, so an invalid room saves
    /// no Input. Each step loads by id what it uses and returns its result; only
    /// ids, texts and step results move between steps.
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

        let (text, memory_entries) =
            self.run_step(step.clone(), Some(kind), || self.narrate(order, input))?;

        let saved = self.run_step(step, Some(kind), || {
            self.save_text(order, text.clone(), &memory_entries)
        })?;
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

    /// One load serves the history cut, the story memory writer and the narrator
    /// call, so a Connection or preset edited during the writer call cannot move the
    /// cut between the two. Rule 7 keeps loaded records inside one step, so the
    /// writer runs inside this step and not as a step of its own in `generate`.
    fn narrate(
        &self,
        order: &GenerationOrder,
        input: Option<&str>,
    ) -> Result<(String, Vec<MemoryEntry>), JobFailure> {
        let game = self.game_row()?;
        let settings = Self::read(Record::Settings, self.storage.get_settings())?;
        let preset = Self::read(
            Record::Preset,
            self.presets
                .chosen_preset(&game, order.generation.preset_type()),
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
        let call = self.call_plan(&settings, order.generation.preset_type(), &preset)?;
        let memory_entries = self.memory_entries_before(match order.place {
            Place::NewMessage => None,
            Place::NewSwipeOnLast => Some(tail.last.id),
        })?;

        let all_characters: Vec<CharacterCard> = bundle.characters.values().cloned().collect();
        let planned = NarrationValues {
            world: &bundle.world,
            narrative_perspective: game.narrative_perspective,
            narrative_tense: game.narrative_tense,
            room,
            characters: CharacterContext {
                all_characters: &all_characters,
                characters_present: &start.scene.characters_present,
            },
            persona: &bundle.persona,
            history: &history,
            input,
            tail: &tail,
            memory_entries: &memory_entries,
        };

        let cut = self
            .narrator
            .plan_cut(&order.generation, &planned, &preset, call.budget);
        let dropped = history.get(..cut.kept_from()).unwrap_or_default();
        let first_uncovered = dropped.len() - memory_entries.uncovered(dropped).len();
        let leaving_calls: Vec<&[StoryRow]> = cut
            .writer_calls(
                cut.dropped_within_one_space(first_uncovered),
                MEMORY_WRITER_CALL_TOKENS,
            )
            .into_iter()
            .map(|range| history.get(range).unwrap_or_default())
            .collect();
        let cast = Cast::new(&bundle.persona.sheet.name, &all_characters);
        let written = self.write_story_memory(
            order.kind(),
            &settings,
            &leaving_calls,
            memory_entries.last(),
            &cast,
        )?;

        let shown = memory_entries.with_written(&written);
        let narrated = self.narrator.narrate(
            &order.generation,
            &NarrationValues {
                memory_entries: &shown,
                ..planned
            },
            &preset,
            &call.settings,
            call.budget,
        )?;
        Ok((narrated.text, written))
    }

    /// Each later call reads the last entry of the call before. A failed call ends the
    /// run: its Messages and the later ones stay uncovered, so the next cut gives them
    /// to the writer again and the entries keep covering the story with no gap.
    fn write_story_memory(
        &self,
        kind: Kind,
        settings: &AppSettings,
        leaving_calls: &[&[StoryRow]],
        previous_entry: Option<&MemoryEntry>,
        cast: &Cast,
    ) -> Result<Vec<MemoryEntry>, JobFailure> {
        let mut written: Vec<MemoryEntry> = Vec::new();
        if leaving_calls.is_empty() {
            return Ok(written);
        }
        for leaving in leaving_calls {
            self.writer.step(Step::WritingStoryMemory)?;
            let Some(hook_input) =
                StoryMemoryHookInput::new(written.last().or(previous_entry), cast, leaving)
            else {
                break;
            };
            let entries = self
                .story_memory_call(settings)
                .and_then(|call| self.story_memory_writer.run(&hook_input, &call));
            match entries {
                Ok(entries) => written.extend(entries),
                Err(error) => {
                    tracing::warn!("Agent {} failed: {error}", self.story_memory_writer.name());
                    break;
                }
            }
        }
        self.writer.step(kind.step())?;
        Ok(written)
    }

    /// The writer records its call through the Narration slot, so the Connection of
    /// that slot sets the ceiling for every kind of generation.
    fn story_memory_call(&self, settings: &AppSettings) -> Result<CallSettings, EngineError> {
        let connection = settings.connection_for(PresetType::System)?;
        Ok(CallSettings {
            max_tokens: connection.max_tokens_for(Some(MEMORY_WRITER_ANSWER_TOKENS)),
            game: Some(self.writer.game()),
            ..CallSettings::default()
        })
    }

    fn save_text(
        &self,
        order: &GenerationOrder,
        text: String,
        memory_entries: &[MemoryEntry],
    ) -> Result<SavedText, JobFailure> {
        let start = self.start_state(order)?;
        let tail = self.tail()?;
        self.writer
            .save_text(order, text, &start, &tail, memory_entries)
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

    fn memory_entries_before(&self, before: Option<u64>) -> Result<MemoryEntries, JobFailure> {
        Self::read(
            Record::Story,
            self.storage
                .stories()
                .memory_entries(self.writer.game(), before),
        )
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

    fn end(&self, failure: Option<JobFailure>) -> JobEnd {
        match failure {
            None => JobEnd::Done,
            Some(JobFailure::Line(line)) => {
                tracing::warn!("job {}: {line:?}", self.writer.job_id());
                match self.writer.save_failure(line) {
                    Ok(()) => JobEnd::Failed,
                    Err(JobFailure::EntryMoved) => {
                        tracing::warn!(
                            "job {}: the failure line was refused: the entry moved on",
                            self.writer.job_id()
                        );
                        JobEnd::EntryMoved
                    }
                    Err(refused) => {
                        tracing::warn!(
                            "job {}: the failure line was refused: {refused:?}",
                            self.writer.job_id()
                        );
                        JobEnd::Failed
                    }
                }
            }
            Some(JobFailure::EntryMoved) => {
                tracing::warn!(
                    "job {}: the entry moved on; the Job ends",
                    self.writer.job_id()
                );
                JobEnd::EntryMoved
            }
        }
    }

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

    /// The plan of one call. The Connection comes from the type of the call, not from
    /// `preset.preset_type`: a Game stores one preset id for each type, and no check ties
    /// that id to a preset of the same type.
    fn call_plan(
        &self,
        settings: &AppSettings,
        preset_type: PresetType,
        preset: &PromptPreset,
    ) -> Result<CallPlan, JobFailure> {
        let connection = Self::read(Record::Settings, settings.connection_for(preset_type))?;
        let max_tokens = connection.max_tokens_for(preset.parameters.max_tokens);
        Ok(CallPlan {
            settings: CallSettings::of(&preset.parameters, max_tokens, self.writer.game()),
            budget: TokenBudget {
                max_context_tokens: connection.resolve_max_context_tokens(),
                max_history_tokens: connection.resolve_max_history_tokens(),
                max_tokens,
            },
        })
    }

    fn options_enabled(&self) -> Result<bool, JobFailure> {
        Ok(self.game_row()?.options_always_on)
    }

    fn hook_settings(&self, preset_type: PresetType) -> Result<AgentSettings, JobFailure> {
        let game = self.game_row()?;
        let settings = Self::read(Record::Settings, self.storage.get_settings())?;
        let preset = Self::read(
            Record::Preset,
            self.presets.chosen_preset(&game, preset_type),
        )?;
        let plan = self.call_plan(&settings, preset_type, &preset)?;
        Ok(AgentSettings {
            preset: Some(preset.assemble_text(&[], None)),
            call: plan.settings,
            option_count: preset.option_count,
        })
    }
}
