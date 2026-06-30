//! [DOC: docs/system/game_flow.md]
//! Action pipeline orchestration and execution

use std::sync::Arc;

use crate::application::action_pipeline::phases::PipelineInputs;
use crate::application::context::{GameServiceContext, load_or_fresh, save_message_and_snapshot};

use crate::domain::model::character::NpcCard;
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::state::trigger_context::StoredTriggerContext;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};

use crate::application::narrative_prompt::LayeredPromptAssembler;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::agents::registry::AgentRegistry;
use crate::domain::model::agent::{AgentContext, AgentResult, ExecutionPhase, StatePatch};

pub struct ActionPipeline {
    pub(super) assembler: Arc<LayeredPromptAssembler>,
    pub(super) recorder: Arc<LlmCallRecorder>,
    pub(super) agents: Arc<AgentRegistry>,
}

#[derive(Debug)]
pub enum ActionOutcome {
    Completed,
    Cancelled,
}

pub(super) type PipelineResult<T> = Result<T, ActionOutcome>;

impl ActionPipeline {
    pub fn new(
        assembler: Arc<LayeredPromptAssembler>,
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
        ctx: &GameServiceContext,
        mut state: GameState,
        input: String,
    ) -> PipelineResult<()> {
        tracing::debug!("run_from_input: called");
        let world = Arc::clone(&state.world);
        let map = Arc::clone(&state.map);
        let player = Arc::clone(&state.player);
        let all_npcs: Vec<NpcCard> = state.npcs.values().cloned().collect();

        let inputs = PipelineInputs {
            input: input.clone(),
            world,
            map,
            player,
            all_npcs,
        };

        state = self.phase_pre_main_snapshot(state, ctx)?;

        let (mut state, narration_text, backend_name, model_name) =
            self.map_cancelled(self.phase_narrate(state, &inputs, ctx), ctx)?;

        if state
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_some()
        {
            self.phase_finalize(&mut state, ctx);
            return Ok(());
        }
        state.narrative.last_backend_name = Some(backend_name);
        state.narrative.last_model_name = Some(model_name);

        let quantifier_result =
            self.phase_post_generation(ctx, &mut state, &input, &narration_text);
        if let Err(e) = save_message_and_snapshot(ctx, &mut state) {
            tracing::warn!("Failed to save post-quantifier metadata: {e}");
        }

        let turn_result =
            match Self::phase_engine_commit(&state, &narration_text, &quantifier_result) {
                Ok(r) => r,
                Err(e) => {
                    state.narrative.input_buffer.status =
                        GenerationStatus::Error(format!("Error: {e}"));
                    self.phase_finalize(&mut state, ctx);
                    return Ok(());
                }
            };
        let mut next_state = turn_result.next_state;

        let trigger_request = turn_result
            .trigger_match
            .as_ref()
            .and_then(|trigger_match| {
                self.build_trigger_request(
                    &next_state,
                    &narration_text,
                    &inputs,
                    trigger_match,
                    ctx,
                )
            });

        if let Some(trigger) = &trigger_request {
            next_state.narrative.last_trigger = Some(trigger.clone());
        }

        if self.persist_snapshot_failed(&mut next_state, "post-engine snapshot", ctx) {
            self.phase_finalize(&mut next_state, ctx);
            return Ok(());
        }

        if let Some(target) = next_state.narrative.retry_target.take() {
            next_state.narrative.history.append(target);
        }

        if let Some(request) = trigger_request {
            match self.phase_trigger_continuation(next_state, &request, ctx) {
                Ok((updated_state, continuation_text)) => {
                    next_state = updated_state;

                    if !continuation_text.is_empty() {
                        next_state = self.reconcile_post_trigger_npcs(
                            next_state,
                            &input,
                            &continuation_text,
                            ctx,
                        );
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        self.phase_finalize(&mut next_state, ctx);
        tracing::debug!("run_from_input: done");
        Ok(())
    }

    fn phase_pre_main_snapshot(
        &self,
        mut state: GameState,
        ctx: &GameServiceContext,
    ) -> PipelineResult<GameState> {
        tracing::info!("Pipeline ▶ Narrating");
        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::Narrating;
        if self.persist_snapshot_failed(&mut state, "pre-main snapshot", ctx) {
            self.phase_finalize(&mut state, ctx);
            return Ok(state);
        }
        Ok(state)
    }

    pub(crate) fn phase_trigger_continuation(
        &self,
        state: GameState,
        trigger: &StoredTriggerContext,
        ctx: &GameServiceContext,
    ) -> PipelineResult<(GameState, String)> {
        self.map_cancelled(
            self.phase_trigger_continuation_raw(state, trigger, ctx),
            ctx,
        )
    }

    fn map_cancelled<T>(
        &self,
        result: PipelineResult<T>,
        ctx: &GameServiceContext,
    ) -> PipelineResult<T> {
        match result {
            Err(ActionOutcome::Cancelled) => Err(self.handle_cancellation(ctx)),
            other => other,
        }
    }

    pub(crate) fn phase_finalize(&self, state: &mut GameState, ctx: &GameServiceContext) {
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
        self.persist(state, ctx);
    }

    fn handle_cancellation(&self, ctx: &GameServiceContext) -> ActionOutcome {
        tracing::warn!("Pipeline cancelled — aborting remaining stages");
        let mut state = load_or_fresh(ctx);
        state.narrative.input_buffer.status = GenerationStatus::Idle;
        state.narrative.input_buffer.phase = GenerationPhase::default();
        self.persist(&state, ctx);
        ActionOutcome::Cancelled
    }

    /// Run post-generation agents inline, aggregating their state patches into QuantifierResult
    pub(super) fn run_post_generation_agents(
        &self,
        state: &GameState,
        player_input: &str,
        main_response: &str,
    ) -> QuantifierResult {
        let mut result = QuantifierResult::default();

        let agent_ctx = AgentContext {
            state,
            main_response: Some(main_response),
            player_input,
            current_room: state.current_room(),
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

impl ActionOutcome {
    pub(crate) fn from_pipeline_result(result: PipelineResult<()>) -> Self {
        match result {
            Ok(()) => ActionOutcome::Completed,
            Err(outcome) => outcome,
        }
    }
}
