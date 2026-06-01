use std::sync::Arc;

use crate::application::context::{
    GameServiceContext, load_or_fresh, map_llm_error, save_message_and_snapshot, save_state,
};
use crate::engine::action_processing::{
    FreeActionContext, TriggerMatch, apply_npc_events, commit_trigger_narration,
    execute_freeaction_impl,
};
use crate::error::EngineError;
use crate::model::character::{NpcCard, PlayerCard};
use crate::model::map::MapDef;
use crate::model::prompt_preset::PromptPreset;
use crate::model::quantifier::{QuantifierConfidence, QuantifierResult, compute_npc_events};
use crate::model::state::StoredTriggerContext;
use crate::model::state::{GameState, GenerationPhase, GenerationStatus, MessageType};
use crate::model::world::WorldCard;
use crate::narrative::llm::backend::LlmCallResult;
use crate::narrative::prompt::{PromptAssembler, make_prompt_context};

pub trait ActionPipelineBackend: Send + Sync {
    fn assembler(&self) -> &dyn PromptAssembler;

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
    pub fn run_from_input(&self, state: GameState, input: String) -> PipelineResult<()> {
        tracing::debug!("run_from_input: called");
        let world = Arc::clone(&state.world);
        let map = Arc::clone(&state.map);
        let player = Arc::clone(&state.player);
        let all_npcs: Vec<NpcCard> = state.npcs.values().cloned().collect();

        let state = self.phase_pre_main_snapshot(state)?;

        let (mut state, narration_text, backend_name, model_name) =
            self.phase_narrate(state, &input, &world, &map, &player, &all_npcs)?;
        state.narrative.last_backend_name = Some(backend_name);
        state.narrative.last_model_name = Some(model_name);

        let quantifier_result = self.phase_post_generation(&mut state, &input, &narration_text);
        if let Err(e) = save_message_and_snapshot(self.ctx, &mut state) {
            tracing::warn!("Failed to save post-quantifier metadata: {e}");
        }

        let turn_result = self.phase_engine_commit(state, &narration_text, &quantifier_result)?;
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

        if let Some(ref trigger) = trigger_request {
            next_state.narrative.last_trigger = Some(trigger.clone());
        }

        if let Err(e) = save_message_and_snapshot(self.ctx, &mut next_state) {
            tracing::error!("Failed to save post-engine snapshot: {e}");
            return Err(ActionOutcome::Error {
                message: format!("Failed to save post-engine snapshot: {e}"),
            });
        }

        if let Some(target) = next_state.narrative.retry_target.take() {
            next_state.narrative.history.append(target);
        }

        if let Some(request) = trigger_request {
            let (updated_state, continuation_text) =
                self.phase_trigger_continuation(next_state, &request)?;
            next_state = updated_state;

            if !continuation_text.is_empty() {
                next_state =
                    self.phase_post_trigger_reconcile(next_state, &input, &continuation_text)?;
            }
        }

        self.phase_finalize(&mut next_state);
        tracing::debug!("run_from_input: done");
        Ok(())
    }

    fn phase_pre_main_snapshot(&self, mut state: GameState) -> PipelineResult<GameState> {
        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::Narrating;
        if let Err(e) = save_message_and_snapshot(self.ctx, &mut state) {
            tracing::error!("Failed to save pre-main snapshot: {e}");
            return Err(ActionOutcome::Error {
                message: format!("Failed to save pre-main snapshot: {e}"),
            });
        }
        Ok(state)
    }

    fn phase_narrate(
        &self,
        mut state: GameState,
        input: &str,
        world: &WorldCard,
        map: &MapDef,
        player: &PlayerCard,
        all_npcs: &[NpcCard],
    ) -> PipelineResult<(GameState, String, String, String)> {
        let Some(room) = map.get_room_by_id(&state.movement.current_room_id) else {
            return Err(self.save_early_error("Room not found"));
        };
        let history = state.narrative.history();

        let (preset, response_length) = self.load_preset_and_response_length()?;

        let context = make_prompt_context(
            world,
            room,
            all_npcs,
            &state.scene.npcs_in_area,
            player,
            input,
            &history,
        );

        let assembled = match self.service.assembler().assemble(
            &context,
            &preset,
            &self.ctx.world.global_rules,
            Some(&response_length),
        ) {
            Ok(a) => a,
            Err(e) => return Err(self.save_early_error(map_llm_error(&e))),
        };

        let narration_result = match self.service.complete(
            crate::narrative::llm::backend::AGENT_NARRATOR,
            &assembled.system_prompt,
            &assembled.user_prompt,
            Some(assembled.max_tokens),
        ) {
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

        state.add_message(narration_text.clone(), None, MessageType::Narration);
        if let Err(e) = save_message_and_snapshot(self.ctx, &mut state) {
            tracing::warn!("Failed to save pre-quantifier narration: {e}");
        }

        Ok((
            state,
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
        let mut quantifier_result = QuantifierResult::default();
        self.service.run_post_generation_agents(
            state,
            input,
            narration_text,
            &mut quantifier_result,
        );

        state.scene.quantifier_confidence =
            Some(format!("{:?}", quantifier_result.npcs.confidence));

        if quantifier_result.npcs.npc_ids.is_empty() && !quantifier_result.npcs.confidence.is_high()
        {
            let room_default_npcs = state
                .scene
                .npcs_in_area
                .iter()
                .map(|n| n.id.clone())
                .collect();
            quantifier_result.npcs.npc_ids = room_default_npcs;
            quantifier_result.npcs.confidence = QuantifierConfidence::Low;
            state.add_message(
                "[System] NPC detection uncertain — using room defaults".to_string(),
                None,
                MessageType::System,
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
        trigger: &StoredTriggerContext,
    ) -> PipelineResult<(GameState, String)> {
        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::GeneratingEvent;
        state.narrative.last_trigger = Some(trigger.clone());

        if self.ctx.cancel_token.is_cancelled() {
            return Err(self.handle_cancellation());
        }

        if let Err(e) = save_message_and_snapshot(self.ctx, &mut state) {
            tracing::error!("Failed to save pre-event snapshot: {e}");
            return Err(ActionOutcome::Error {
                message: format!("Failed to save pre-event snapshot: {e}"),
            });
        }

        let continuation_result = match self.service.complete(
            crate::narrative::llm::backend::AGENT_TRIGGER,
            &trigger.system_prompt,
            &trigger.user_prompt,
            trigger.max_tokens,
        ) {
            Ok(result) => result,
            Err(e) => {
                tracing::error!("Trigger narration failed: {e}");
                state.add_message(
                    format!("[Trigger narration failed: {e}]"),
                    None,
                    MessageType::System,
                );
                state.narrative.input_buffer.status =
                    GenerationStatus::Error(format!("Error: {e}"));
                if let Err(e) = save_message_and_snapshot(self.ctx, &mut state) {
                    tracing::error!("Critical: failed to persist trigger error state: {e}");
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
            if let Err(e) = save_message_and_snapshot(self.ctx, &mut state) {
                tracing::error!("Critical: failed to persist empty trigger state: {e}");
            }
            return Err(ActionOutcome::Error {
                message: "LLM Error: empty response".to_string(),
            });
        }

        state = match commit_trigger_narration(state.clone(), trigger, &continuation_text) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Trigger commit failed: {e}");
                state.narrative.input_buffer.status =
                    GenerationStatus::Error(format!("Trigger error: {e}"));
                if let Err(e) = save_message_and_snapshot(self.ctx, &mut state) {
                    tracing::error!("Critical: failed to persist trigger commit error state: {e}");
                }
                return Err(ActionOutcome::Error {
                    message: format!("Trigger commit failed: {e}"),
                });
            }
        };

        if let Err(e) = save_message_and_snapshot(self.ctx, &mut state) {
            tracing::error!("Failed to save post-trigger snapshot: {e}");
            return Err(ActionOutcome::Error {
                message: format!("Failed to save post-trigger snapshot: {e}"),
            });
        }

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
                tracing::error!("Failed to apply post-trigger NPC events: {e}");
                state.narrative.input_buffer.status =
                    GenerationStatus::Error(format!("NPC event error: {e}"));
                if let Err(e) = save_state(self.ctx, &state) {
                    tracing::error!("Critical: failed to persist NPC error state: {e}");
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
            tracing::error!("Failed to persist finished action: {e}");
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
            tracing::warn!("Retry event continuation cancelled — aborting");
            state.narrative.input_buffer.status = GenerationStatus::Idle;
            state.narrative.input_buffer.phase = GenerationPhase::default();
            if let Err(e) = save_state(self.ctx, &state) {
                tracing::error!("Failed to persist cancelled retry state: {e}");
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
                tracing::error!("Trigger narration retry failed: {e}");
                state.narrative.input_buffer.status =
                    GenerationStatus::Error(format!("Trigger narration failed: {e}"));
                if let Err(e) = save_state(self.ctx, &state) {
                    tracing::error!("Critical: failed to persist trigger retry error state: {e}");
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
            if let Err(e) = save_state(self.ctx, &state) {
                tracing::error!("Critical: failed to persist empty trigger retry state: {e}");
            }
            return ActionOutcome::Error {
                message: "LLM Error: empty response".to_string(),
            };
        }

        let mut committed_state =
            match commit_trigger_narration(state.clone(), &trigger, &continuation_text) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Trigger commit failed on retry: {e}");
                    state.narrative.input_buffer.status =
                        GenerationStatus::Error(format!("Trigger error: {e}"));
                    if let Err(e) = save_message_and_snapshot(self.ctx, &mut state) {
                        tracing::error!(
                            "Critical: failed to persist trigger commit error state: {e}"
                        );
                    }
                    return ActionOutcome::Error {
                        message: format!("Trigger error: {e}"),
                    };
                }
            };

        if let Err(e) = save_message_and_snapshot(self.ctx, &mut committed_state) {
            tracing::error!("Failed to save post-trigger retry snapshot: {e}");
            return ActionOutcome::Error {
                message: format!("Failed to save post-trigger retry snapshot: {e}"),
            };
        }

        match self.reconcile_post_trigger_npcs(
            committed_state.clone(),
            input_text,
            &continuation_text,
        ) {
            Ok(updated) => committed_state = updated,
            Err(e) => {
                tracing::error!("Failed to apply post-trigger NPC events on retry: {e}");
                committed_state.narrative.input_buffer.status =
                    GenerationStatus::Error(format!("NPC event error: {e}"));
                if let Err(e) = save_state(self.ctx, &committed_state) {
                    tracing::error!("Critical: failed to persist retry NPC error state: {e}");
                }
                return ActionOutcome::Error {
                    message: format!("NPC event error: {e}"),
                };
            }
        }

        if let Some(target) = committed_state.narrative.retry_target.take() {
            committed_state.narrative.history.append(target);
        }

        committed_state.narrative.input_buffer.status = GenerationStatus::Idle;
        committed_state.narrative.input_buffer.phase = GenerationPhase::default();
        if let Err(e) = save_state(self.ctx, &committed_state) {
            tracing::error!("Failed to persist finished retry action: {e}");
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
        let mut post_trigger_result = QuantifierResult::default();
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
        let mut state = load_or_fresh(self.ctx);
        let message = error.into();
        state.narrative.input_buffer.status = GenerationStatus::Error(message.clone());
        if let Err(e) = save_state(self.ctx, &state) {
            tracing::error!("Critical: failed to persist error state: {e}");
        }
        ActionOutcome::Error { message }
    }

    fn handle_cancellation(&self) -> ActionOutcome {
        tracing::warn!("Pipeline cancelled — aborting remaining stages");
        let mut state = load_or_fresh(self.ctx);
        state.narrative.input_buffer.status = GenerationStatus::Idle;
        state.narrative.input_buffer.phase = GenerationPhase::default();
        if let Err(e) = save_state(self.ctx, &state) {
            tracing::error!("Critical: failed to persist cancelled state: {e}");
        }
        ActionOutcome::Cancelled
    }

    fn load_preset_and_response_length(&self) -> Result<(PromptPreset, String), ActionOutcome> {
        let settings = self.ctx.settings.read().unwrap_or_else(|e| e.into_inner());
        let preset_id = settings.active_system_prompt_preset_id.clone();
        let response_length = settings.response_length.clone();
        match self.ctx.preset_storage.get_preset(&preset_id) {
            Ok(Some(p)) => Ok((p, response_length)),
            Ok(None) => {
                tracing::error!(
                    "active system preset '{preset_id}' not found — defaults not seeded?"
                );
                Err(self.save_early_error("Active system preset not found"))
            }
            Err(e) => {
                tracing::error!("preset storage inaccessible: {e}");
                Err(self.save_early_error("Preset storage inaccessible"))
            }
        }
    }

    fn build_trigger_request(
        &self,
        state: &GameState,
        narration_text: &str,
        world: &WorldCard,
        player: &PlayerCard,
        all_npcs: &[NpcCard],
        trigger_match: &TriggerMatch,
    ) -> Option<StoredTriggerContext> {
        let continuation_user_msg = format!(
            "Previous narration:\n{}\n\nTrigger event: {}\n\n\
             Continue the scene naturally, incorporating the trigger event into the narrative. \
             Do NOT repeat or contradict what was already described. Build naturally on the existing scene.",
            narration_text, trigger_match.trigger_narration_prompt
        );

        let room_data = state.current_room()?;
        let history = state.narrative.history();

        let (preset, response_length) = self.load_preset_and_response_length().ok()?;

        let trigger_ctx = make_prompt_context(
            world,
            room_data,
            all_npcs,
            &state.scene.npcs_in_area,
            player,
            &continuation_user_msg,
            &history,
        );

        let assembled = self
            .service
            .assembler()
            .assemble(
                &trigger_ctx,
                &preset,
                &self.ctx.world.global_rules,
                Some(&response_length),
            )
            .ok()?;

        Some(StoredTriggerContext {
            npc_id: trigger_match.npc_id.clone(),
            trigger_idx: trigger_match.trigger_idx,
            trigger_name: trigger_match.trigger_name.clone(),
            trigger_repeat: trigger_match.trigger_repeat,
            trigger_narration_prompt: trigger_match.trigger_narration_prompt.clone(),
            system_prompt: assembled.system_prompt,
            user_prompt: assembled.user_prompt,
            max_tokens: Some(assembled.max_tokens),
        })
    }
}

impl ActionOutcome {
    /// Bridge between refactored pipeline and legacy callers.
    pub(crate) fn from_pipeline_result(result: PipelineResult<()>) -> Self {
        match result {
            Ok(()) => ActionOutcome::Completed,
            Err(outcome) => outcome,
        }
    }
}
