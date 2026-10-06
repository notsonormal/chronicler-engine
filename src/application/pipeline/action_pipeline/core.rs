//! [DOC: docs/diataxis/reference/game_flow.md]
//! Shared action-pipeline state, constructors, and orchestration helpers.

use std::collections::HashMap;
use std::sync::Arc;

use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::application::pipeline::phase_error::PhaseError;
use crate::application::pipeline::narration_generation;
use crate::application::pipeline::pipeline_run::PipelineRun;
use crate::adapters::driven::storage::worlds::WorldBundle;
use crate::adapters::driven::storage::Storage;

use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::map::MapDef;
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::state::trigger_context::StoredTriggerContext;
use crate::domain::model::state::game_state::{ActionResult, FreeActionContext, GameState};
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};

use crate::application::errors::ProcessActionResult;
use crate::application::generation::gate::GenerationGate;
use crate::application::prompting::PromptAssembler;
use crate::application::prompting::token_budget::MAX_CONTEXT_TOKENS;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::agents::quantifier::QuantifierAgent;
use crate::application::agents::registry::AgentRegistry;
use crate::application::message_service::MessageService;
use crate::application::ports::llm_provider::LlmProvider;
use crate::domain::model::agent::{AgentContext, AgentResult, ExecutionPhase, StatePatch};
use crate::error::EngineError;

#[derive(Clone)]
pub struct ActionPipeline {
    pub(crate) prompt_assembler: Arc<PromptAssembler>,
    pub(crate) recorder: Arc<LlmCallRecorder>,
    pub(crate) agent_registry: Arc<AgentRegistry>,
    pub(crate) message_service: Arc<MessageService>,
    pub(crate) storage: Arc<Storage>,
    pub(crate) shutdown_token: CancellationToken,
}

impl ActionPipeline {
    pub fn with_storage(
        shutdown_token: CancellationToken,
        recorder: Arc<LlmCallRecorder>,
        agent_registry: AgentRegistry,
        message_service: Arc<MessageService>,
        storage: Arc<Storage>,
    ) -> Self {
        tracing::info!("ActionPipeline: provider={}", recorder.provider_label());
        Self::with_backends(
            shutdown_token,
            recorder,
            agent_registry,
            message_service,
            storage,
        )
    }

    pub fn with_backends(
        shutdown_token: CancellationToken,
        recorder: Arc<LlmCallRecorder>,
        agent_registry: AgentRegistry,
        message_service: Arc<MessageService>,
        storage: Arc<Storage>,
    ) -> Self {
        Self {
            prompt_assembler: Arc::new(
                PromptAssembler::new(MAX_CONTEXT_TOKENS).with_storage(Arc::clone(&storage)),
            ),
            recorder,
            agent_registry: Arc::new(agent_registry),
            message_service,
            storage,
            shutdown_token,
        }
    }

    pub fn with_mock_quantifier(
        shutdown_token: CancellationToken,
        recorder: Arc<LlmCallRecorder>,
        quantifier_provider: Arc<dyn LlmProvider>,
        message_service: Arc<MessageService>,
        storage: Arc<Storage>,
    ) -> Self {
        let agent = QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider);
        let registry = AgentRegistry::with_agent(Box::new(agent));
        Self::with_backends(shutdown_token, recorder, registry, message_service, storage)
    }

    pub fn backend_info(&self) -> (String, String) {
        match self.recorder.provider() {
            Ok(p) => (p.name().to_string(), p.model().to_string()),
            Err(e) => (format!("<unresolved: {e}>"), "<unresolved>".to_string()),
        }
    }

    pub fn recorder(&self) -> &Arc<LlmCallRecorder> {
        &self.recorder
    }

    pub fn rebind_for_test(
        mut self,
        message_service: Arc<MessageService>,
        storage: Arc<Storage>,
        shutdown_token: CancellationToken,
    ) -> Self {
        self.message_service = message_service;
        self.storage = storage;
        self.shutdown_token = shutdown_token;
        self
    }

    pub fn is_shutting_down(&self) -> bool {
        self.shutdown_token.is_cancelled()
    }

    pub fn reset_persisted_status(&self) -> Result<(), EngineError> {
        let mut game_state = self.message_service.load_or_fresh();
        game_state.narrative.input_buffer.status = GenerationStatus::Idle;
        game_state.narrative.input_buffer.phase = GenerationPhase::default();
        self.message_service.save_state(&game_state)?;
        Ok(())
    }

    fn heal_stale_if_changed(
        &self,
        generation_gate: &GenerationGate,
        game_id: u64,
        game_state: &mut GameState,
    ) -> Result<(), EngineError> {
        let before = game_state.narrative.input_buffer.status.clone();
        generation_gate.heal_stale(game_id, game_state);
        if game_state.narrative.input_buffer.status != before {
            self.message_service.save_state(game_state)?;
        }
        Ok(())
    }

    pub fn heal_stale_status(&self, generation_gate: &GenerationGate) -> Result<(), EngineError> {
        let game_id = self.storage.current_game_id();
        let mut game_state = self.message_service.load_or_fresh();
        self.heal_stale_if_changed(generation_gate, game_id, &mut game_state)
    }

    pub(super) fn claim_and_spawn<E, BeforeClaim, PreSpawn, SpawnTask>(
        &self,
        generation_gate: &GenerationGate,
        before_claim: BeforeClaim,
        pre_spawn: PreSpawn,
        spawn_task: SpawnTask,
    ) -> Result<ProcessActionResult, E>
    where
        E: From<EngineError>,
        BeforeClaim: FnOnce(u64, &mut GameState) -> Result<(), E>,
        PreSpawn: FnOnce() -> Result<(), E>,
        SpawnTask: FnOnce(&ActionPipeline) + Send + 'static,
    {
        if self.is_shutting_down() {
            return Ok(ProcessActionResult::ShuttingDown);
        }
        let mut game_state = self.message_service.load_or_fresh();
        let game_id = self.storage.current_game_id();

        // Heal before validating or claiming, so a panicked turn cannot strand the page.
        self.heal_stale_if_changed(generation_gate, game_id, &mut game_state)?;

        before_claim(game_id, &mut game_state)?;

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

        // On `pre_spawn` failure no `guard` drops, so release the claimed slot
        // manually to keep the registry in sync with the returned `Err`.
        if let Err(e) = pre_spawn() {
            generation_gate.release_generation_slot(started_game_id, started_generation_id);
            return Err(e);
        }

        let gate = generation_gate.clone();
        let pipeline_arc = Arc::new(self.clone());
        tokio::task::spawn_blocking(move || {
            tracing::debug!("spawn_blocking: task started");
            let _guard = gate.guard(started_game_id, started_generation_id);
            if pipeline_arc.is_shutting_down() {
                tracing::debug!("spawn_blocking: shutting down before executing task");
                return;
            }
            spawn_task(&pipeline_arc);
            tracing::debug!("spawn_blocking: task completed");
        });
        Ok(ProcessActionResult::Started)
    }

    #[instrument(skip(self, impersonated, steering_instruction), fields(input_length))]
    pub fn run_from_input(
        &self,
        mut state: GameState,
        input: String,
        impersonated: bool,
        steering_instruction: Option<String>,
    ) -> Result<(), PhaseError> {
        tracing::debug!("run_from_input: called");
        let started_for = self.storage.current_game_id();
        let run = PipelineRun::new(self, started_for);

        // Fresh entry inputs win; a redo falls back to the retry target's
        // stored inputs. Guide and impersonate cannot collide — one
        // `steering_instruction` field, discriminated by `impersonated`.
        let (impersonated, steering_instruction) = if impersonated || steering_instruction.is_some()
        {
            (impersonated, steering_instruction)
        } else {
            match state.narrative.retry_target.as_ref() {
                Some(target) => (
                    target.impersonated(),
                    target.steering_instruction().map(|d| d.to_string()),
                ),
                None => (false, None),
            }
        };
        let generation_inputs = narration_generation::GenerationInputs {
            input: input.clone(),
            guide: if impersonated {
                None
            } else {
                steering_instruction.clone()
            },
            impersonate: impersonated.then_some(narration_generation::ImpersonateInputs {
                steering_instruction,
            }),
        };

        if let Err(e) = run.phase_pre_main_snapshot(&mut state) {
            Self::finalize_phase_error(&run, Some(&mut state), e);
            return Ok(());
        }

        let outcome = match narration_generation::NarrationGeneration::new(&run, generation_inputs)
            .run(&mut state)
        {
            Err(PhaseError::Cancelled) => return Err(run.handle_cancellation()),
            Err(e) => {
                Self::finalize_phase_error(&run, Some(&mut state), e);
                return Ok(());
            }
            Ok(outcome) => outcome,
        };
        let narration_text = outcome.narration_text;
        state.narrative.last_backend_name = Some(outcome.backend_name);
        state.narrative.last_model_name = Some(outcome.model_name);
        let (map, persona, npcs) = {
            let bundle = &outcome.bundle;
            (
                Arc::clone(&bundle.map),
                Arc::clone(&bundle.persona),
                bundle.npcs.clone(),
            )
        };

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
                    &outcome.bundle,
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

        // Non-impersonate turns rewrite the offered set at turn end;
        // impersonate output is the player acting.
        if !impersonated {
            let options_enabled = run.resolve_options_always_on(&outcome.bundle.world);
            self.rewrite_options_after_turn(
                &mut post_commit_state,
                &map,
                &persona,
                &npcs,
                options_enabled,
            );
        }

        run.phase_finalize(&mut post_commit_state);
        tracing::debug!("run_from_input: done");
        Ok(())
    }

    pub(crate) fn load_world_bundle(&self, started_for: u64) -> Result<WorldBundle, EngineError> {
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
        state: &mut GameState,
        trigger: &StoredTriggerContext,
        map: &Arc<MapDef>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<String, PhaseError> {
        let started_for = self.storage.current_game_id();
        let run = PipelineRun::new(self, started_for);
        match run.phase_trigger_continuation_llm_call(state, trigger, map, npcs) {
            Err(PhaseError::Cancelled) => Err(run.handle_cancellation()),
            other => other,
        }
    }

    pub(crate) fn run_post_generation_agents(
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
                // Options agents dispatch by phase at gated call sites, never
                // through this merge loop; the variant is unreachable here.
                Ok(AgentResult::NoOp)
                | Ok(AgentResult::PromptDirective(_))
                | Ok(AgentResult::Options(_)) => None,
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

    pub(crate) fn log_cancellation(&self, outcome: Result<(), PhaseError>) {
        // Non-Cancelled errors are persisted by `finalize_phase_error`, which
        // preserves in-flight state.
        if let Err(PhaseError::Cancelled) = outcome {
            tracing::debug!("Pipeline cancelled");
        }
    }

    pub(crate) fn retry_event_continuation(&self, state: &mut GameState) -> Result<(), PhaseError> {
        let started_for = self.storage.current_game_id();
        let run = PipelineRun::new(self, started_for);

        let Some(trigger) = state.narrative.last_trigger.clone() else {
            Self::finalize_phase_error(&run, Some(state), PhaseError::TriggerMissing);
            return Ok(());
        };
        let input_text = state
            .narrative
            .history
            .last_input_text()
            .unwrap_or_default();
        let WorldBundle {
            world,
            map,
            persona,
            npcs: npcs_map,
            ..
        } = match self.load_world_bundle(started_for) {
            Ok(b) => b,
            Err(e) => {
                Self::finalize_phase_error(
                    &run,
                    Some(state),
                    PhaseError::FetchFailed(e.to_string()),
                );
                return Ok(());
            }
        };
        let continuation_text =
            match self.phase_trigger_continuation(state, &trigger, &map, &npcs_map) {
                Ok(t) => t,
                Err(PhaseError::Cancelled) => return Err(PhaseError::Cancelled),
                Err(e) => {
                    Self::finalize_phase_error(&run, Some(state), e);
                    return Ok(());
                }
            };
        if !continuation_text.is_empty() {
            match run.reconcile_post_trigger_npcs(
                state,
                &input_text,
                &continuation_text,
                &map,
                &persona,
                &npcs_map,
            ) {
                Ok(()) => {}
                Err(PhaseError::Cancelled) => return Err(PhaseError::Cancelled),
                Err(e) => {
                    Self::finalize_phase_error(&run, Some(state), e);
                    return Ok(());
                }
            }
        }
        if let Some(target) = state.narrative.retry_target.take() {
            state.narrative.history.append(target);
        }
        // An event-only retry is a narration turn and never an impersonation,
        // so the offered set follows the same turn-end rewrite rule.
        let options_enabled = run.resolve_options_always_on(&world);
        self.rewrite_options_after_turn(state, &map, &persona, &npcs_map, options_enabled);
        run.phase_finalize(state);
        Ok(())
    }

    pub(crate) fn retry_main_narration(
        &self,
        state: GameState,
        input_text: String,
    ) -> Result<(), PhaseError> {
        self.run_from_input(state, input_text, false, None)
    }

    fn phase_engine_commit(
        state: GameState,
        narration_text: &str,
        quantifier_result: &QuantifierResult,
        map: &Arc<MapDef>,
        persona: &Arc<PersonaCard>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<ActionResult, EngineError> {
        state.execute_freeaction_impl(
            &FreeActionContext {
                narration_text,
                quantifier_result,
            },
            map,
            persona,
            npcs,
        )
    }
}
