//! [DOC: docs/system/game_flow.md]
//! Action pipeline orchestration and execution

use std::sync::Arc;

use crate::application::context::{GameServiceContext, load_or_fresh, save_message_and_snapshot};
use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::quantifier::QuantifierResult;
use crate::model::state::StoredTriggerContext;
use crate::model::state::{GameState, GenerationPhase, GenerationStatus};
use crate::narrative::llm::backend::LlmCallResult;
use crate::narrative::prompt::LayeredPromptAssembler;

pub trait ActionPipelineBackend: Send + Sync {
    fn assembler(&self) -> &LayeredPromptAssembler;

    fn complete(
        &self,
        agent_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError>;

    fn run_post_generation_agents(
        &self,
        state: &GameState,
        player_input: &str,
        main_response: &str,
        result: &mut QuantifierResult,
    );
}

pub struct ActionPipeline<'a, B: ActionPipelineBackend> {
    pub(super) service: &'a B,
    pub(super) ctx: &'a GameServiceContext,
}

#[derive(Debug)]
pub enum ActionOutcome {
    Completed,
    Cancelled,
}

pub(super) type PipelineResult<T> = Result<T, ActionOutcome>;

impl<'a, B: ActionPipelineBackend> ActionPipeline<'a, B> {
    pub fn new(service: &'a B, ctx: &'a GameServiceContext) -> Self {
        Self { service, ctx }
    }

    pub fn run_from_input(&self, mut state: GameState, input: String) -> PipelineResult<()> {
        tracing::debug!("run_from_input: called");
        let world = Arc::clone(&state.world);
        let map = Arc::clone(&state.map);
        let player = Arc::clone(&state.player);
        let all_npcs: Vec<NpcCard> = state.npcs.values().cloned().collect();

        state = self.phase_pre_main_snapshot(state)?;

        let (mut state, narration_text, backend_name, model_name) = self
            .map_cancelled(self.phase_narrate(state, &input, &world, &map, &player, &all_npcs))?;

        if state
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_some()
        {
            self.phase_finalize(&mut state);
            return Ok(());
        }
        state.narrative.last_backend_name = Some(backend_name);
        state.narrative.last_model_name = Some(model_name);

        let quantifier_result = self.phase_post_generation(&mut state, &input, &narration_text);
        if let Err(e) = save_message_and_snapshot(self.ctx, &mut state) {
            tracing::warn!("Failed to save post-quantifier metadata: {e}");
        }

        let turn_result =
            match Self::phase_engine_commit(&state, &narration_text, &quantifier_result) {
                Ok(r) => r,
                Err(e) => {
                    state.narrative.input_buffer.status =
                        GenerationStatus::Error(format!("Error: {e}"));
                    self.phase_finalize(&mut state);
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
                    &world,
                    &player,
                    &all_npcs,
                    trigger_match,
                )
            });

        if let Some(trigger) = &trigger_request {
            next_state.narrative.last_trigger = Some(trigger.clone());
        }

        if self.persist_snapshot_failed(&mut next_state, "post-engine snapshot") {
            self.phase_finalize(&mut next_state);
            return Ok(());
        }

        if let Some(target) = next_state.narrative.retry_target.take() {
            next_state.narrative.history.append(target);
        }

        if let Some(request) = trigger_request {
            match self.phase_trigger_continuation(next_state, &request) {
                Ok((updated_state, continuation_text)) => {
                    next_state = updated_state;

                    if !continuation_text.is_empty() {
                        next_state = self.reconcile_post_trigger_npcs(
                            next_state,
                            &input,
                            &continuation_text,
                        );
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        self.phase_finalize(&mut next_state);
        tracing::debug!("run_from_input: done");
        Ok(())
    }

    fn phase_pre_main_snapshot(&self, mut state: GameState) -> PipelineResult<GameState> {
        tracing::info!("Pipeline ▶ Narrating");
        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::Narrating;
        if self.persist_snapshot_failed(&mut state, "pre-main snapshot") {
            self.phase_finalize(&mut state);
            return Ok(state);
        }
        Ok(state)
    }

    pub(crate) fn phase_trigger_continuation(
        &self,
        state: GameState,
        trigger: &StoredTriggerContext,
    ) -> PipelineResult<(GameState, String)> {
        self.map_cancelled(self.phase_trigger_continuation_raw(state, trigger))
    }

    fn map_cancelled<T>(&self, result: PipelineResult<T>) -> PipelineResult<T> {
        match result {
            Err(ActionOutcome::Cancelled) => Err(self.handle_cancellation()),
            other => other,
        }
    }

    pub(crate) fn phase_finalize(&self, state: &mut GameState) {
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

    fn handle_cancellation(&self) -> ActionOutcome {
        tracing::warn!("Pipeline cancelled — aborting remaining stages");
        let mut state = load_or_fresh(self.ctx);
        state.narrative.input_buffer.status = GenerationStatus::Idle;
        state.narrative.input_buffer.phase = GenerationPhase::default();
        self.persist(&state);
        ActionOutcome::Cancelled
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
