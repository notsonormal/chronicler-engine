//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Action pipeline orchestration and execution

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::application::pipeline::phase_error::PhaseError;
use crate::application::pipeline::phases::{PipelineInputs, PipelineRun};
use crate::application::pipeline::spawn::spawn_pipeline_task;
use crate::adapters::driven::storage::worlds::WorldBundle;
use crate::adapters::driven::storage::{PresetStore, Storage};

use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::map::MapDef;
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::state::trigger_context::StoredTriggerContext;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;

use crate::application::errors::{ApplicationError, ProcessActionResult};
use crate::application::generation::gate::GenerationGate;
use crate::application::prompting::PromptAssembler;
use crate::application::prompting::token_budget::MAX_CONTEXT_TOKENS;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::agents::quantifier::QuantifierAgent;
use crate::application::agents::registry::AgentRegistry;
use crate::application::message_service::MessageService;
use crate::application::ports::llm_provider::LlmProvider;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::agent::{AgentContext, AgentResult, ExecutionPhase, StatePatch};
use crate::error::EngineError;

#[derive(Clone)]
pub struct ActionPipeline {
    pub(super) prompt_assembler: Arc<PromptAssembler>,
    pub(super) recorder: Arc<LlmCallRecorder>,
    pub(super) agent_registry: Arc<AgentRegistry>,
    pub(super) message_service: Arc<MessageService>,
    pub(super) storage: Arc<Storage>,
    pub(super) preset_store: Arc<PresetStore>,
    pub(super) settings: Arc<RwLock<AppSettings>>,
    pub(super) shutdown_token: CancellationToken,
}

impl ActionPipeline {
    pub fn with_storage(
        shutdown_token: CancellationToken,
        recorder: Arc<LlmCallRecorder>,
        agent_registry: AgentRegistry,
        message_service: Arc<MessageService>,
        storage: Arc<Storage>,
        preset_store: Arc<PresetStore>,
        settings: Arc<RwLock<AppSettings>>,
    ) -> Self {
        tracing::info!(
            "ActionPipeline: backend={}, model={}",
            recorder.provider().name(),
            recorder.provider().model()
        );
        Self::with_backends(
            shutdown_token,
            recorder,
            agent_registry,
            message_service,
            storage,
            preset_store,
            settings,
        )
    }

    pub fn with_backends(
        shutdown_token: CancellationToken,
        recorder: Arc<LlmCallRecorder>,
        agent_registry: AgentRegistry,
        message_service: Arc<MessageService>,
        storage: Arc<Storage>,
        preset_store: Arc<PresetStore>,
        settings: Arc<RwLock<AppSettings>>,
    ) -> Self {
        Self {
            prompt_assembler: Arc::new(
                PromptAssembler::new(MAX_CONTEXT_TOKENS).with_settings(settings.clone()),
            ),
            recorder,
            agent_registry: Arc::new(agent_registry),
            message_service,
            storage,
            preset_store,
            settings,
            shutdown_token,
        }
    }

    pub fn with_mock_quantifier(
        shutdown_token: CancellationToken,
        recorder: Arc<LlmCallRecorder>,
        quantifier_provider: Arc<dyn LlmProvider>,
        message_service: Arc<MessageService>,
        storage: Arc<Storage>,
        preset_store: Arc<PresetStore>,
        settings: Arc<RwLock<AppSettings>>,
    ) -> Self {
        let agent = QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider);
        let registry = AgentRegistry::with_agent(Box::new(agent));
        Self::with_backends(
            shutdown_token,
            recorder,
            registry,
            message_service,
            storage,
            preset_store,
            settings,
        )
    }

    pub fn backend_info(&self) -> (&str, &str) {
        (
            self.recorder.provider().name(),
            self.recorder.provider().model(),
        )
    }

    pub fn recorder(&self) -> &Arc<LlmCallRecorder> {
        &self.recorder
    }

    pub fn prompt_assembler(&self) -> &Arc<PromptAssembler> {
        &self.prompt_assembler
    }

    /// Aligns injected pipeline with seeded application state.
    pub fn rebind_for_test(
        mut self,
        message_service: Arc<MessageService>,
        storage: Arc<Storage>,
        preset_store: Arc<PresetStore>,
        settings: Arc<RwLock<AppSettings>>,
        shutdown_token: CancellationToken,
    ) -> Self {
        self.message_service = message_service;
        self.storage = storage;
        self.preset_store = preset_store;
        self.settings = settings;
        self.shutdown_token = shutdown_token;
        self
    }

    pub fn is_shutting_down(&self) -> bool {
        self.shutdown_token.is_cancelled()
    }

    /// Reset the persisted generation status/phase to Idle for the current game.
    pub fn reset_persisted_status(&self) -> Result<(), EngineError> {
        let mut game_state = self.message_service.load_or_fresh();
        game_state.narrative.input_buffer.status = GenerationStatus::Idle;
        game_state.narrative.input_buffer.phase = GenerationPhase::default();
        self.message_service.save_state(&game_state)?;
        Ok(())
    }

    pub fn process_action(
        &self,
        generation_gate: &GenerationGate,
        input: String,
    ) -> Result<ProcessActionResult, EngineError> {
        let mut game_state = self.message_service.load_or_fresh();
        let game_id = self.storage.current_game_id();

        generation_gate.heal_stale(game_id, &mut game_state);

        let game = self.storage.require_game(game_id)?;
        let persona = self.storage.require_persona(&game.persona_key)?;
        let player_name = persona.sheet.name.clone();
        if !input.is_empty() {
            game_state.add_message(input.clone(), Some(player_name.clone()), MessageType::Input);
        }

        let (started_game_id, started_generation_id, claim_result) =
            generation_gate.try_claim(game_id, &mut game_state, self.message_service.as_ref())?;
        match claim_result {
            ProcessActionResult::ConcurrentGeneration => {
                return Ok(ProcessActionResult::ConcurrentGeneration);
            }
            ProcessActionResult::Started => {}
            ProcessActionResult::ShuttingDown => {
                return Ok(ProcessActionResult::ShuttingDown);
            }
        }

        let gate = generation_gate.clone();
        let pipeline_arc = Arc::new(self.clone());
        spawn_pipeline_task(pipeline_arc, move |pipeline| {
            tracing::debug!("spawn_blocking: task started");
            let _guard = gate.guard(started_game_id, started_generation_id);
            let shutting = pipeline.is_shutting_down();
            if shutting {
                tracing::debug!("spawn_blocking: shutting down before execute_action");
                return;
            }
            pipeline.execute_action(input);
            tracing::debug!("spawn_blocking: execute_action completed");
        });
        Ok(ProcessActionResult::Started)
    }

    #[instrument(skip(self), fields(input_length))]
    pub fn execute_action(&self, input: String) {
        let mut state = self.message_service.load_or_fresh();
        state.narrative.last_trigger = None;
        if let Err(PhaseError::Cancelled) = self.run_from_input(state, input) {
            tracing::debug!("Pipeline cancelled");
        }
    }

    #[instrument(skip(self))]
    pub fn retry_last_response(&self) {
        let messages = match self.message_service.load_messages() {
            Ok(m) => m,
            Err(e) => {
                self.persist_generation_error(format!("Retry failed: {e}"));
                return;
            }
        };

        let Some((anchor_idx, _anchor_msg, snapshot_id)) =
            self.message_service.find_retry_anchor(&messages)
        else {
            self.persist_generation_error("Retry failed: no anchor message");
            return;
        };

        let is_event = messages
            .last()
            .map(|m| m.event_header().is_some())
            .unwrap_or(false);

        let old_target = messages
            .iter()
            .rev()
            .find(|m| {
                if is_event {
                    m.event_header().is_some()
                } else {
                    matches!(
                        m.message_type,
                        MessageType::Narration | MessageType::Dialogue
                    ) && m.event_header().is_none()
                }
            })
            .cloned();

        let snapshot = match self.storage.load_snapshot_by_id(snapshot_id) {
            Ok(Some(s)) => s,
            Ok(None) => {
                self.persist_generation_error(format!(
                    "Retry failed: no snapshot found for id {snapshot_id}"
                ));
                return;
            }
            Err(e) => {
                self.persist_generation_error(format!("Retry failed: {e}"));
                return;
            }
        };

        let mut state = GameState::from_snapshot(&snapshot);
        let mut truncated = messages;
        truncated.truncate(anchor_idx + 1);
        state.narrative.history.replace(truncated);
        state.narrative.retry_target = old_target;

        let input_text = match state.narrative.history.last_input_text() {
            Some((_, text)) => text,
            None => {
                self.persist_generation_error("Retry failed: no input to retry");
                return;
            }
        };

        let outcome = if is_event {
            self.retry_event_continuation(state)
        } else {
            self.retry_main_narration(state, input_text)
        };

        self.handle_retry_outcome(outcome);
    }

    #[instrument(skip(self))]
    pub fn retrigger_event(&self) {
        let state = self.message_service.load_or_fresh();
        let outcome = self.retry_event_continuation(state);
        self.handle_retry_outcome(outcome);
    }

    pub fn continue_narration(
        &self,
        generation_gate: &GenerationGate,
    ) -> Result<ProcessActionResult, EngineError> {
        self.process_action(generation_gate, String::new())
    }

    pub fn retry(&self) -> Result<(), ApplicationError> {
        let game_state = self.message_service.load_or_fresh();

        if game_state.narrative.history.last_input_text().is_none() {
            return Err(ApplicationError::validation("No input to retry"));
        }

        let (_, cancelled) = self.prepare_retry_state(
            game_state,
            GenerationStatus::Generating,
            GenerationPhase::Narrating,
        )?;
        if cancelled {
            return Err(ApplicationError::ShuttingDown);
        }

        let pipeline_arc = Arc::new(self.clone());
        spawn_pipeline_task(pipeline_arc, move |pipeline| {
            if pipeline.is_shutting_down() {
                return;
            }
            pipeline.retry_last_response();
        });

        Ok(())
    }

    pub fn retrigger(&self) -> Result<(), ApplicationError> {
        let game_state = self.message_service.load_or_fresh();

        if game_state.narrative.last_trigger.is_none() {
            return Err(ApplicationError::validation("No trigger context available"));
        }

        let messages = self.message_service.load_messages()?;
        let Some(last_msg) = messages.last() else {
            return Err(ApplicationError::validation("No messages to retrigger"));
        };

        let is_narration = last_msg.message_type == MessageType::Narration
            || last_msg.message_type == MessageType::Dialogue;

        if !is_narration || last_msg.event_header().is_some() {
            return Err(ApplicationError::validation(
                "Last message must be a narration to retrigger",
            ));
        }

        let (_, cancelled) = self.prepare_retry_state(
            game_state,
            GenerationStatus::Generating,
            GenerationPhase::Narrating,
        )?;
        if cancelled {
            return Err(ApplicationError::ShuttingDown);
        }

        let pipeline_arc = Arc::new(self.clone());
        spawn_pipeline_task(pipeline_arc, move |pipeline| {
            if pipeline.is_shutting_down() {
                return;
            }
            pipeline.retrigger_event();
        });

        Ok(())
    }

    /// Set generation status/phase, persist the resulting snapshot, and return the
    /// mutated state paired with the current shutdown flag (`cancelled=true` if
    /// shutdown was requested mid-call).
    pub(crate) fn prepare_retry_state(
        &self,
        mut game_state: GameState,
        status: GenerationStatus,
        phase: GenerationPhase,
    ) -> Result<(GameState, bool), ApplicationError> {
        game_state.narrative.input_buffer.status = status;
        game_state.narrative.input_buffer.phase = phase;
        self.message_service.save_state(&game_state)?;
        let cancelled = self.is_shutting_down();
        Ok((game_state, cancelled))
    }

    pub fn run_from_input(&self, mut state: GameState, input: String) -> Result<(), PhaseError> {
        tracing::debug!("run_from_input: called");
        let started_for = self.storage.current_game_id();
        let run = PipelineRun::new(self, started_for);

        let WorldBundle {
            world,
            map,
            persona,
            npcs,
        } = match self.load_world_bundle(started_for) {
            Ok(bundle) => bundle,
            Err(e) => {
                tracing::error!("run_from_input: {e}");
                self.persist_generation_error(e.to_string());
                return Ok(());
            }
        };
        let all_npcs: Vec<NpcCard> = npcs.values().cloned().collect();
        let inputs = PipelineInputs {
            input: input.clone(),
            world: Arc::clone(&world),
            map: Arc::clone(&map),
            persona: Arc::clone(&persona),
            all_npcs,
        };

        if let Err(e) = run.phase_pre_main_snapshot(&mut state) {
            Self::finalize_phase_error(&run, Some(&mut state), e);
            return Ok(());
        }

        let (narration_text, backend_name, model_name) =
            match run.phase_narrate(&mut state, &inputs) {
                Err(PhaseError::Cancelled) => return Err(run.handle_cancellation()),
                Err(e) => {
                    Self::finalize_phase_error(&run, Some(&mut state), e);
                    return Ok(());
                }
                Ok(t) => t,
            };
        state.narrative.last_backend_name = Some(backend_name);
        state.narrative.last_model_name = Some(model_name);

        let quantifier_result = match run.phase_post_generation(
            &mut state,
            &input,
            &narration_text,
            &map,
            &persona,
            &npcs,
        ) {
            Ok(r) => r,
            Err(e) => {
                Self::finalize_phase_error(&run, Some(&mut state), e);
                return Ok(());
            }
        };

        let turn_result = match Self::phase_engine_commit(
            state,
            &narration_text,
            &quantifier_result,
            &map,
            &persona,
            &npcs,
        ) {
            Ok(r) => r,
            Err(e) => {
                Self::finalize_phase_error(
                    &run,
                    None,
                    PhaseError::PersistFailed {
                        label: "engine commit",
                        source: e,
                    },
                );
                return Ok(());
            }
        };
        let mut post_commit_state = turn_result.post_commit_state;

        let trigger_request = turn_result
            .trigger_match
            .as_ref()
            .and_then(|trigger_match| {
                run.build_trigger_request(
                    &post_commit_state,
                    &narration_text,
                    &inputs,
                    trigger_match,
                )
            });
        if let Err(e) = run.persist_snapshot_or_err(&mut post_commit_state, "post-engine snapshot")
        {
            Self::finalize_phase_error(&run, Some(&mut post_commit_state), e);
            return Ok(());
        }
        if let Some(target) = post_commit_state.narrative.retry_target.take() {
            post_commit_state.narrative.history.append(target);
        }

        if let Some(request) = trigger_request {
            let continuation_text = match run.phase_trigger_continuation_llm_call(
                &mut post_commit_state,
                &request,
                &map,
                &npcs,
            ) {
                Err(PhaseError::Cancelled) => return Err(run.handle_cancellation()),
                Err(e) => {
                    Self::finalize_phase_error(&run, Some(&mut post_commit_state), e);
                    return Ok(());
                }
                Ok(t) => t,
            };
            if !continuation_text.is_empty() {
                if let Err(e) = run.reconcile_post_trigger_npcs(
                    &mut post_commit_state,
                    &input,
                    &continuation_text,
                    &map,
                    &persona,
                    &npcs,
                ) {
                    Self::finalize_phase_error(&run, Some(&mut post_commit_state), e);
                    return Ok(());
                }
            }
        }

        run.phase_finalize(&mut post_commit_state);
        tracing::debug!("run_from_input: done");
        Ok(())
    }

    pub(super) fn load_world_bundle(&self, started_for: u64) -> Result<WorldBundle, EngineError> {
        self.storage.world_bundle_for(started_for)
    }

    pub(super) fn finalize_phase_error(
        run: &PipelineRun<'_>,
        state: Option<&mut GameState>,
        e: PhaseError,
    ) {
        let msg = match e {
            PhaseError::NarratorFailed(msg) => msg,
            PhaseError::FetchFailed(msg) => msg,
            PhaseError::PersistFailed { label, source } => {
                tracing::error!("{label}: {source}");
                source.to_string()
            }
            PhaseError::TriggerMissing => "Retry failed: missing trigger context".to_string(),
            PhaseError::SnapshotMissing => "World data unavailable for current game".to_string(),
            PhaseError::Cancelled => {
                unreachable!("Cancelled must be handled before calling finalize_phase_error")
            }
        };

        match state {
            Some(state) => {
                state.narrative.input_buffer.status = GenerationStatus::Error(msg);
                state.narrative.input_buffer.phase = GenerationPhase::default();
                if let Err(e) = run
                    .pipeline
                    .message_service
                    .save_message_and_snapshot(state)
                {
                    tracing::error!("Failed to persist error state: {e}");
                }
            }
            None => run.pipeline.persist_generation_error(msg),
        }
    }

    pub(crate) fn phase_trigger_continuation(
        &self,
        mut state: GameState,
        trigger: &StoredTriggerContext,
        map: &Arc<MapDef>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<(GameState, String), PhaseError> {
        let started_for = self.storage.current_game_id();
        let run = PipelineRun::new(self, started_for);
        let continuation_text =
            match run.phase_trigger_continuation_llm_call(&mut state, trigger, map, npcs) {
                Err(PhaseError::Cancelled) => return Err(run.handle_cancellation()),
                other => other,
            }?;
        Ok((state, continuation_text))
    }

    pub(super) fn run_post_generation_agents(
        &self,
        state: &GameState,
        player_input: &str,
        main_response: &str,
        map: &Arc<MapDef>,
        persona: &Arc<PersonaCard>,
        npcs: &HashMap<String, NpcCard>,
    ) -> QuantifierResult {
        let mut result = QuantifierResult::default();

        let current_room = map
            .get_room_by_id(&state.movement.current_room_id)
            .or_else(|| {
                state
                    .movement
                    .dynamic_rooms
                    .get(&state.movement.current_room_id)
            });
        let agent_ctx = AgentContext {
            state,
            main_response: Some(main_response),
            player_input,
            current_room,
            map,
            persona,
            npcs,
        };

        let patches: Vec<_> = self
            .agent_registry
            .agents_for_phase(ExecutionPhase::PostGeneration)
            .filter_map(|agent| match agent.execute(&agent_ctx) {
                Ok(AgentResult::StatePatch(patch)) => Some(patch),
                Ok(AgentResult::NoOp) | Ok(AgentResult::PromptDirective(_)) => None,
                Err(e) => {
                    tracing::warn!("Agent {} failed: {e}", agent.name());
                    None
                }
            })
            .collect();

        if let Some(first_patch) = patches.into_iter().reduce(StatePatch::merge) {
            let StatePatch {
                npc_ids,
                movement_destination,
                confidence,
            } = first_patch;
            result.npcs.npc_ids = npc_ids;
            result.movement.destination = movement_destination;
            result.npcs.confidence = confidence.into();
        }

        result
    }

    pub(crate) fn persist_generation_error(&self, message: impl Into<String>) {
        let mut state = self.message_service.load_or_fresh();
        state.narrative.input_buffer.status = GenerationStatus::Error(message.into());
        state.narrative.input_buffer.phase = GenerationPhase::default();
        if let Err(e) = self.message_service.save_state(&state) {
            tracing::error!("Critical: failed to persist generation error state: {e}");
        }
    }

    pub(crate) fn handle_retry_outcome(&self, outcome: Result<(), PhaseError>) {
        match outcome {
            Err(PhaseError::Cancelled) => {
                let mut state = self.message_service.load_or_fresh();
                state.narrative.input_buffer.status = GenerationStatus::Idle;
                state.narrative.input_buffer.phase = GenerationPhase::default();
                let _ = self.message_service.save_state(&state);
            }
            Err(e) => {
                let started_for = self.storage.current_game_id();
                let run = PipelineRun::new(self, started_for);
                Self::finalize_phase_error(&run, None, e);
            }
            Ok(()) => {}
        }
    }

    pub(crate) fn retry_event_continuation(&self, state: GameState) -> Result<(), PhaseError> {
        let Some(trigger) = state.narrative.last_trigger.clone() else {
            return Err(PhaseError::TriggerMissing);
        };
        let input_text = match state.narrative.history.last_input_text() {
            Some((_, text)) => text,
            None => String::new(),
        };
        let WorldBundle {
            map,
            persona,
            npcs: npcs_map,
            ..
        } = match self.load_world_bundle(self.storage.current_game_id()) {
            Ok(b) => b,
            Err(e) => return Err(PhaseError::FetchFailed(e.to_string())),
        };
        let (mut state, continuation_text) =
            self.phase_trigger_continuation(state, &trigger, &map, &npcs_map)?;
        if !continuation_text.is_empty() {
            let started_for = self.storage.current_game_id();
            let run = PipelineRun::new(self, started_for);
            run.reconcile_post_trigger_npcs(
                &mut state,
                &input_text,
                &continuation_text,
                &map,
                &persona,
                &npcs_map,
            )?;
        }
        if let Some(target) = state.narrative.retry_target.take() {
            state.narrative.history.append(target);
        }
        {
            let started_for = self.storage.current_game_id();
            let run = PipelineRun::new(self, started_for);
            run.phase_finalize(&mut state);
        }
        Ok(())
    }

    pub(crate) fn retry_main_narration(
        &self,
        state: GameState,
        input_text: String,
    ) -> Result<(), PhaseError> {
        self.run_from_input(state, input_text)
    }
}

impl<'a> PipelineRun<'a> {
    pub(super) fn phase_pre_main_snapshot(&self, state: &mut GameState) -> Result<(), PhaseError> {
        tracing::info!("Pipeline ▶ Narrating");
        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::Narrating;
        self.persist_snapshot_or_err(state, "pre-main snapshot")?;
        Ok(())
    }

    pub(super) fn phase_finalize(&self, state: &mut GameState) {
        tracing::info!(
            "Pipeline ✓ Finalize (status={:?})",
            state.narrative.input_buffer.status
        );

        if state
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_none()
        {
            state.narrative.input_buffer.status = GenerationStatus::Idle;
        }
        state.narrative.input_buffer.phase = GenerationPhase::default();
        self.persist(state);
    }

    pub(super) fn handle_cancellation(&self) -> PhaseError {
        tracing::warn!("Pipeline cancelled — aborting remaining stages");
        let mut state = self.pipeline.message_service.load_or_fresh();
        state.narrative.input_buffer.status = GenerationStatus::Idle;
        state.narrative.input_buffer.phase = GenerationPhase::default();
        self.persist(&state);
        PhaseError::Cancelled
    }
}
