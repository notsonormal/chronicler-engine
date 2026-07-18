//! [DOC: docs/system/game_flow.md]
//! Action pipeline orchestration and execution

use std::collections::HashMap;
use std::sync::Arc;

use crate::application::action_pipeline::phase_error::PhaseError;
use crate::application::action_pipeline::phases::{PipelineInputs, PipelineRun};
use crate::application::application_service::DefaultApplicationService;
use crate::adapters::driven::storage::worlds::WorldBundle;

use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::map::MapDef;
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::state::trigger_context::StoredTriggerContext;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};

use crate::application::narrative_prompt::PromptAssembler;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::agents::registry::AgentRegistry;
use crate::domain::model::agent::{AgentContext, AgentResult, ExecutionPhase, StatePatch};
use crate::EngineError;

pub struct ActionPipeline {
    pub(super) assembler: Arc<PromptAssembler>,
    pub(super) recorder: Arc<LlmCallRecorder>,
    pub(super) agents: Arc<AgentRegistry>,
}

impl ActionPipeline {
    pub fn new(
        assembler: Arc<PromptAssembler>,
        recorder: Arc<LlmCallRecorder>,
        agents: Arc<AgentRegistry>,
    ) -> Self {
        Self {
            assembler,
            recorder,
            agents,
        }
    }

    pub fn run_from_input(
        &self,
        app: &DefaultApplicationService,
        mut state: GameState,
        input: String,
    ) -> Result<(), PhaseError> {
        tracing::debug!("run_from_input: called");
        let started_for = app.current_game_id();
        let run = PipelineRun::new(self, app, started_for);

        let WorldBundle {
            world,
            map,
            persona,
            npcs,
        } = match Self::load_world_bundle(app, started_for) {
            Ok(bundle) => bundle,
            Err(e) => {
                tracing::error!("run_from_input: {e}");
                let mut state = app.load_or_fresh();
                state.narrative.input_buffer.status = GenerationStatus::Error(e.to_string());
                run.phase_finalize(&mut state);
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

        state = match run.phase_pre_main_snapshot(state) {
            Ok(s) => s,
            Err(e) => {
                Self::finalize_phase_error(&run, e);
                return Ok(());
            }
        };

        let (mut state, narration_text, backend_name, model_name) =
            match run.phase_narrate(state, &inputs) {
                Err(PhaseError::Cancelled) => return Err(run.handle_cancellation()),
                Err(e) => {
                    Self::finalize_phase_error(&run, e);
                    return Ok(());
                }
                Ok(t) => t,
            };
        state.narrative.last_backend_name = Some(backend_name);
        state.narrative.last_model_name = Some(model_name);

        let quantifier_result =
            run.phase_post_generation(&mut state, &input, &narration_text, &map, &persona, &npcs);
        if let Err(e) = run.app.save_message_and_snapshot(&mut state) {
            tracing::warn!("Failed to save post-quantifier metadata: {e}");
        }

        let turn_result = match Self::phase_engine_commit(
            &state,
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
                    PhaseError::PersistFailed {
                        label: "engine commit",
                        source: e,
                    },
                );
                return Ok(());
            }
        };
        let mut next_state = turn_result.next_state;

        let trigger_request = turn_result
            .trigger_match
            .as_ref()
            .and_then(|trigger_match| {
                run.build_trigger_request(&next_state, &narration_text, &inputs, trigger_match)
            });
        if let Some(trigger) = &trigger_request {
            next_state.narrative.last_trigger = Some(trigger.clone());
        }
        if let Err(e) = run.persist_snapshot_or_err(&mut next_state, "post-engine snapshot") {
            Self::finalize_phase_error(&run, e);
            return Ok(());
        }
        if let Some(target) = next_state.narrative.retry_target.take() {
            next_state.narrative.history.append(target);
        }

        if let Some(request) = trigger_request {
            let (updated_state, continuation_text) = match run
                .phase_trigger_continuation_with_cancel_handling(next_state, &request, &map, &npcs)
            {
                Err(PhaseError::Cancelled) => return Err(run.handle_cancellation()),
                Err(e) => {
                    Self::finalize_phase_error(&run, e);
                    return Ok(());
                }
                Ok(t) => t,
            };
            next_state = updated_state;
            if !continuation_text.is_empty() {
                next_state = run.reconcile_post_trigger_npcs(
                    next_state,
                    &input,
                    &continuation_text,
                    &map,
                    &persona,
                    &npcs,
                );
            }
        }

        run.phase_finalize(&mut next_state);
        tracing::debug!("run_from_input: done");
        Ok(())
    }

    pub(super) fn load_world_bundle(
        app: &DefaultApplicationService,
        started_for: u64,
    ) -> Result<WorldBundle, EngineError> {
        app.storage().world_bundle_for(started_for)
    }

    pub(super) fn finalize_phase_error(run: &PipelineRun<'_>, e: PhaseError) {
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
        let mut state = run.app.load_or_fresh();
        state.narrative.input_buffer.status = GenerationStatus::Error(msg);
        run.phase_finalize(&mut state);
    }

    pub(crate) fn phase_trigger_continuation(
        &self,
        state: GameState,
        trigger: &StoredTriggerContext,
        app: &DefaultApplicationService,
        map: &Arc<MapDef>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<(GameState, String), PhaseError> {
        let started_for = app.current_game_id();
        let run = PipelineRun::new(self, app, started_for);
        run.phase_trigger_continuation_with_cancel_handling(state, trigger, map, npcs)
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
            .agents
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
}

impl<'a> PipelineRun<'a> {
    pub(super) fn phase_pre_main_snapshot(
        &self,
        mut state: GameState,
    ) -> Result<GameState, PhaseError> {
        tracing::info!("Pipeline ▶ Narrating");
        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::Narrating;
        self.persist_snapshot_or_err(&mut state, "pre-main snapshot")?;
        Ok(state)
    }

    fn phase_trigger_continuation_with_cancel_handling(
        &self,
        state: GameState,
        trigger: &StoredTriggerContext,
        map: &Arc<MapDef>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<(GameState, String), PhaseError> {
        match self.phase_trigger_continuation_llm_call(state, trigger, map, npcs) {
            Err(PhaseError::Cancelled) => Err(self.handle_cancellation()),
            other => other,
        }
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
        let mut state = self.app.load_or_fresh();
        state.narrative.input_buffer.status = GenerationStatus::Idle;
        state.narrative.input_buffer.phase = GenerationPhase::default();
        self.persist(&state);
        PhaseError::Cancelled
    }
}
