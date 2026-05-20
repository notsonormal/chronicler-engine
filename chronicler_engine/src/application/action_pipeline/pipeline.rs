use std::sync::Arc;

use crate::application::context::{
    GameServiceContext, load_state, map_llm_error, save_committed_state, save_state,
};
use crate::engine::action_processing::{
    FreeActionContext, TriggerContinuationRequest, TriggerMatch, apply_npc_events,
    commit_trigger_narration, execute_freeaction_impl,
};
use crate::error::EngineError;
use crate::model::character::{NpcCard, PlayerCard};
use crate::model::map::MapDef;
use crate::model::quantifier::{
    MovementParseResult, QuantifierConfidence, QuantifierParseResult, QuantifierResult,
    compute_npc_events,
};
use crate::model::state::StoredTriggerContext;
use crate::model::state::{GameState, GenerationPhase, GenerationStatus, LogType};
use crate::model::world::WorldCard;
use crate::narrative::llm::backend::LlmCallResult;
use crate::narrative::prompt::{PromptBuilder, make_prompt_context};

pub trait ActionPipelineBackend: Send + Sync {
    fn narrate_action(
        &self,
        context: &crate::narrative::prompt::PromptContext,
    ) -> Result<LlmCallResult, EngineError>;

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
    service: &'a B,
    ctx: &'a GameServiceContext,
}

#[derive(Debug)]
pub enum ActionOutcome {
    Completed,
    Error { message: String },
    Cancelled,
}

type PipelineResult<T> = Result<T, ActionOutcome>;

impl<'a, B: ActionPipelineBackend> ActionPipeline<'a, B> {
    pub fn new(service: &'a B, ctx: &'a GameServiceContext) -> Self {
        Self { service, ctx }
    }

    /// [DOC: docs/architecture/system.md]
    pub fn run_from_input(&self, state: GameState, input: String) -> ActionOutcome {
        let world = Arc::clone(&state.world);
        let map = Arc::clone(&state.map);
        let player = Arc::clone(&state.player);
        let all_npcs: Vec<NpcCard> = state.npcs.values().cloned().collect();

        let mut state = match self.phase_pre_main_snapshot(state) {
            Ok(s) => s,
            Err(outcome) => return outcome,
        };

        let (narration_text, backend_name, model_name) =
            match self.phase_narrate(&state, &input, &world, &map, &player, &all_npcs) {
                Ok((text, backend, model)) => (text, backend, model),
                Err(outcome) => return outcome,
            };
        state.narrative.last_backend_name = Some(backend_name);
        state.narrative.last_model_name = Some(model_name);

        let quantifier_result = self.phase_post_generation(&mut state, &input, &narration_text);

        let turn_result = match self.phase_engine_commit(state, &narration_text, &quantifier_result)
        {
            Ok(result) => result,
            Err(outcome) => return outcome,
        };
        let mut next_state = turn_result.next_state;

        if let Some(trigger_match) = turn_result.trigger_match {
            let (response_length, max_context_tokens, max_tokens) = self.ctx.prompt_build_params();
            let system_prompt_override = {
                let settings = self.ctx.settings.read().unwrap_or_else(|e| e.into_inner());
                settings.active_system_prompt.clone()
            };
            let trigger_request = build_trigger_request(
                &next_state,
                &narration_text,
                &world,
                &player,
                &all_npcs,
                &response_length,
                max_context_tokens,
                max_tokens,
                &trigger_match,
                system_prompt_override,
            );

            if let Some(request) = trigger_request {
                let (updated_state, continuation_text) =
                    match self.phase_trigger_continuation(next_state, &request) {
                        Ok(pair) => pair,
                        Err(outcome) => return outcome,
                    };
                next_state = updated_state;

                if !continuation_text.is_empty() {
                    next_state = match self.phase_post_trigger_reconcile(
                        next_state,
                        &input,
                        &continuation_text,
                    ) {
                        Ok(updated) => updated,
                        Err(outcome) => return outcome,
                    };
                }
            }
        }

        self.phase_finalize(&mut next_state);
        ActionOutcome::Completed
    }

    fn phase_pre_main_snapshot(&self, mut state: GameState) -> PipelineResult<GameState> {
        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::Narrating;
        if let Err(e) = save_committed_state(self.ctx, &mut state) {
            log::error!("Failed to save pre-main snapshot: {e}");
            return Err(ActionOutcome::Error {
                message: format!("Failed to save pre-main snapshot: {e}"),
            });
        }
        Ok(state)
    }

    fn phase_narrate(
        &self,
        state: &GameState,
        input: &str,
        world: &WorldCard,
        map: &MapDef,
        player: &PlayerCard,
        all_npcs: &[NpcCard],
    ) -> PipelineResult<(String, String, String)> {
        let Some(room) = map.get_room_by_id(&state.movement.current_room_id) else {
            return Err(self.save_early_error("Room not found"));
        };
        let history = state.narrative.history();
        let system_prompt_override = {
            let settings = self.ctx.settings.read().unwrap_or_else(|e| e.into_inner());
            settings.active_system_prompt.clone()
        };
        let context = make_prompt_context(
            world,
            room,
            all_npcs,
            &state.scene.npcs_in_area,
            player,
            input,
            &history,
            system_prompt_override,
        );

        let narration_result = match self.service.narrate_action(&context) {
            Ok(result) => result,
            Err(e) => return Err(self.save_early_error(map_llm_error(&e))),
        };
        let narration_text = narration_result.text;

        if self.ctx.cancel_token.is_cancelled() {
            return Err(self.handle_cancellation());
        }

        if narration_text.trim().is_empty() {
            return Err(self.save_early_error("LLM Error: empty response"));
        }

        Ok((
            narration_text,
            narration_result.backend_name,
            narration_result.model_name,
        ))
    }

    fn phase_post_generation(
        &self,
        state: &mut GameState,
        input: &str,
        narration_text: &str,
    ) -> QuantifierResult {
        let mut quantifier_result = default_quantifier_result(&[]);
        self.service.run_post_generation_agents(
            state,
            input,
            narration_text,
            &mut quantifier_result,
        );

        state.scene.quantifier_confidence =
            Some(format!("{:?}", quantifier_result.npcs.confidence));

        if quantifier_result.npcs.confidence == QuantifierConfidence::Low {
            state.add_log(
                "[System] NPC detection uncertain — using room defaults".to_string(),
                None,
                LogType::System,
            );
        }

        quantifier_result
    }

    fn phase_engine_commit(
        &self,
        state: GameState,
        narration_text: &str,
        quantifier_result: &QuantifierResult,
    ) -> PipelineResult<crate::engine::action_processing::TurnResult> {
        execute_freeaction_impl(
            &state,
            &FreeActionContext {
                narration_text,
                quantifier_result,
            },
        )
        .map_err(|e| self.save_early_error(format!("Error: {e}")))
    }

    fn phase_trigger_continuation(
        &self,
        mut state: GameState,
        request: &TriggerContinuationRequest,
    ) -> PipelineResult<(GameState, String)> {
        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::GeneratingEvent;
        state.narrative.last_trigger = Some(request.stored.clone());

        if self.ctx.cancel_token.is_cancelled() {
            return Err(self.handle_cancellation());
        }

        if let Err(e) = save_committed_state(self.ctx, &mut state) {
            log::error!("Failed to save pre-event snapshot: {e}");
            return Err(ActionOutcome::Error {
                message: format!("Failed to save pre-event snapshot: {e}"),
            });
        }

        let continuation_result = match self.service.complete(
            crate::narrative::llm::backend::AGENT_TRIGGER,
            &request.stored.system_prompt,
            &request.stored.user_prompt,
            request.stored.max_tokens,
        ) {
            Ok(result) => result,
            Err(e) => {
                log::error!("Trigger narration failed: {e}");
                state.add_log(
                    format!("[Trigger narration failed: {e}]"),
                    None,
                    LogType::System,
                );
                state.narrative.input_buffer.status =
                    GenerationStatus::Error(format!("Error: {e}"));
                if let Err(e) = save_state(self.ctx, &mut state) {
                    log::error!("Critical: failed to persist trigger error state: {e}");
                }
                return Err(ActionOutcome::Error {
                    message: format!("Trigger narration failed: {e}"),
                });
            }
        };
        let continuation_text = continuation_result.text;

        if self.ctx.cancel_token.is_cancelled() {
            return Err(self.handle_cancellation());
        }

        if continuation_text.trim().is_empty() {
            state.narrative.input_buffer.status =
                GenerationStatus::Error("LLM Error: empty response".to_string());
            if let Err(e) = save_state(self.ctx, &mut state) {
                log::error!("Critical: failed to persist empty trigger state: {e}");
            }
            return Err(ActionOutcome::Error {
                message: "LLM Error: empty response".to_string(),
            });
        }

        state = match commit_trigger_narration(state.clone(), request, &continuation_text) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Trigger commit failed: {e}");
                state.narrative.input_buffer.status =
                    GenerationStatus::Error(format!("Trigger error: {e}"));
                if let Err(e) = save_state(self.ctx, &mut state) {
                    log::error!("Critical: failed to persist trigger commit error state: {e}");
                }
                return Err(ActionOutcome::Error {
                    message: format!("Trigger commit failed: {e}"),
                });
            }
        };

        Ok((state, continuation_text))
    }

    fn phase_post_trigger_reconcile(
        &self,
        mut state: GameState,
        input: &str,
        continuation_text: &str,
    ) -> PipelineResult<GameState> {
        match self.reconcile_post_trigger_npcs(state.clone(), input, continuation_text) {
            Ok(updated) => Ok(updated),
            Err(e) => {
                log::error!("Failed to apply post-trigger NPC events: {e}");
                state.narrative.input_buffer.status =
                    GenerationStatus::Error(format!("NPC event error: {e}"));
                if let Err(e) = save_state(self.ctx, &mut state) {
                    log::error!("Critical: failed to persist NPC error state: {e}");
                }
                Err(ActionOutcome::Error {
                    message: format!("NPC event error: {e}"),
                })
            }
        }
    }

    fn phase_finalize(&self, state: &mut GameState) {
        state.narrative.input_buffer.status = GenerationStatus::Idle;
        state.narrative.input_buffer.phase = GenerationPhase::default();
        if let Err(e) = save_state(self.ctx, state) {
            log::error!("Failed to persist finished action: {e}");
        }
    }

    /// [DOC: docs/architecture/system.md]
    pub fn run_trigger_continuation(
        &self,
        mut state: GameState,
        trigger: StoredTriggerContext,
        input_text: &str,
    ) -> ActionOutcome {
        if self.ctx.cancel_token.is_cancelled() {
            log::warn!("Retry event continuation cancelled — aborting");
            state.narrative.input_buffer.status = GenerationStatus::Idle;
            state.narrative.input_buffer.phase = GenerationPhase::default();
            if let Err(e) = save_state(self.ctx, &mut state) {
                log::error!("Failed to persist cancelled retry state: {e}");
            }
            return ActionOutcome::Cancelled;
        }

        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::GeneratingEvent;

        let continuation_result = match self.service.complete(
            crate::narrative::llm::backend::AGENT_TRIGGER,
            &trigger.system_prompt,
            &trigger.user_prompt,
            trigger.max_tokens,
        ) {
            Ok(result) => result,
            Err(e) => {
                log::error!("Trigger narration retry failed: {e}");
                state.narrative.input_buffer.status =
                    GenerationStatus::Error(format!("Trigger narration failed: {e}"));
                if let Err(e) = save_state(self.ctx, &mut state) {
                    log::error!("Critical: failed to persist trigger retry error state: {e}");
                }
                return ActionOutcome::Error {
                    message: format!("Retry failed: {e}"),
                };
            }
        };
        let continuation_text = continuation_result.text;

        if continuation_text.trim().is_empty() {
            state.narrative.input_buffer.status =
                GenerationStatus::Error("LLM Error: empty response".to_string());
            if let Err(e) = save_state(self.ctx, &mut state) {
                log::error!("Critical: failed to persist empty trigger retry state: {e}");
            }
            return ActionOutcome::Error {
                message: "LLM Error: empty response".to_string(),
            };
        }

        let request = TriggerContinuationRequest { stored: trigger };

        let mut committed_state =
            match commit_trigger_narration(state.clone(), &request, &continuation_text) {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Trigger commit failed on retry: {e}");
                    state.narrative.input_buffer.status =
                        GenerationStatus::Error(format!("Trigger error: {e}"));
                    if let Err(e) = save_state(self.ctx, &mut state) {
                        log::error!("Critical: failed to persist trigger commit error state: {e}");
                    }
                    return ActionOutcome::Error {
                        message: format!("Trigger error: {e}"),
                    };
                }
            };

        match self.reconcile_post_trigger_npcs(
            committed_state.clone(),
            input_text,
            &continuation_text,
        ) {
            Ok(updated) => committed_state = updated,
            Err(e) => {
                log::error!("Failed to apply post-trigger NPC events on retry: {e}");
                committed_state.narrative.input_buffer.status =
                    GenerationStatus::Error(format!("NPC event error: {e}"));
                if let Err(e) = save_state(self.ctx, &mut committed_state) {
                    log::error!("Critical: failed to persist retry NPC error state: {e}");
                }
                return ActionOutcome::Error {
                    message: format!("NPC event error: {e}"),
                };
            }
        }

        committed_state.narrative.input_buffer.status = GenerationStatus::Idle;
        committed_state.narrative.input_buffer.phase = GenerationPhase::default();
        if let Err(e) = save_state(self.ctx, &mut committed_state) {
            log::error!("Failed to persist finished retry action: {e}");
        }

        ActionOutcome::Completed
    }

    fn reconcile_post_trigger_npcs(
        &self,
        mut state: GameState,
        player_input: &str,
        continuation_text: &str,
    ) -> Result<GameState, EngineError> {
        state.narrative.input_buffer.phase = GenerationPhase::Quantifying;

        let previous_ids: Vec<String> = state
            .scene
            .npcs_in_area
            .iter()
            .map(|n| n.id.clone())
            .collect();
        let mut post_trigger_result = default_quantifier_result(&previous_ids);
        self.service.run_post_generation_agents(
            &state,
            player_input,
            continuation_text,
            &mut post_trigger_result,
        );

        state.scene.quantifier_confidence =
            Some(format!("{:?}", post_trigger_result.npcs.confidence));

        let npc_cards: Vec<NpcCard> = post_trigger_result
            .npcs
            .npc_ids
            .iter()
            .filter_map(|id| state.npcs.get(id).cloned())
            .collect();
        let new_ids: Vec<String> = npc_cards.iter().map(|n| n.id.clone()).collect();

        state.scene.npcs_in_area = npc_cards;

        let events = compute_npc_events(&previous_ids, &new_ids);
        apply_npc_events(state, &events.events)
    }

    fn save_early_error(&self, error: impl Into<String>) -> ActionOutcome {
        let mut state = load_state(self.ctx);
        let message = error.into();
        state.narrative.input_buffer.status = GenerationStatus::Error(message.clone());
        if let Err(e) = save_state(self.ctx, &mut state) {
            log::error!("Critical: failed to persist error state: {e}");
        }
        ActionOutcome::Error { message }
    }

    fn handle_cancellation(&self) -> ActionOutcome {
        log::warn!("Pipeline cancelled — aborting remaining stages");
        let mut state = load_state(self.ctx);
        state.narrative.input_buffer.status = GenerationStatus::Idle;
        state.narrative.input_buffer.phase = GenerationPhase::default();
        if let Err(e) = save_state(self.ctx, &mut state) {
            log::error!("Critical: failed to persist cancelled state: {e}");
        }
        ActionOutcome::Cancelled
    }
}

pub(crate) fn default_quantifier_result(fallback_npc_ids: &[String]) -> QuantifierResult {
    QuantifierResult {
        npcs: QuantifierParseResult {
            npc_ids: fallback_npc_ids.to_vec(),
            confidence: QuantifierConfidence::Low,
        },
        movement: MovementParseResult::default(),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_trigger_request(
    state: &GameState,
    narration_text: &str,
    world: &WorldCard,
    player: &PlayerCard,
    all_npcs: &[NpcCard],
    response_length: &str,
    max_context_tokens: u32,
    max_tokens: Option<u32>,
    trigger_match: &TriggerMatch,
    system_prompt_override: Option<String>,
) -> Option<TriggerContinuationRequest> {
    let continuation_user_msg = format!(
        "Previous narration:\n{}\n\nTrigger event: {}\n\n\
         Continue the scene naturally, incorporating the trigger event into the narrative. \
         Do NOT repeat or contradict what was already described. Build naturally on the existing scene.",
        narration_text, trigger_match.trigger_narration_prompt
    );

    let room_data = state.current_room()?;
    let history = state.narrative.history();
    let trigger_ctx = make_prompt_context(
        world,
        room_data,
        all_npcs,
        &state.scene.npcs_in_area,
        player,
        &continuation_user_msg,
        &history,
        system_prompt_override,
    );

    let mut pb = PromptBuilder::from_context(&trigger_ctx);
    pb.max_context_tokens = Some(max_context_tokens);
    pb.requested_max_tokens = max_tokens;
    pb.response_length = Some(response_length);

    let (system_prompt, user_prompt, fitted_max_tokens) = pb.build_split().ok()?;

    Some(TriggerContinuationRequest {
        stored: StoredTriggerContext {
            npc_id: trigger_match.npc_id.clone(),
            trigger_idx: trigger_match.trigger_idx,
            trigger_name: trigger_match.trigger_name.clone(),
            trigger_repeat: trigger_match.trigger_repeat,
            trigger_narration_prompt: trigger_match.trigger_narration_prompt.clone(),
            system_prompt,
            user_prompt,
            max_tokens: Some(fitted_max_tokens),
        },
    })
}
