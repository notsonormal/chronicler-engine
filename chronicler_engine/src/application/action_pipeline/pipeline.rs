//! [DOC: docs/system/game_flow.md]
//! Action pipeline orchestration and execution

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
    #[allow(dead_code)]
    // Note: not returned after error-model unification; errors go to GenerationStatus::Error on state
    Error {
        message: String,
    },
    Cancelled,
}

type PipelineResult<T> = Result<T, ActionOutcome>;

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

        let (mut state, narration_text, backend_name, model_name) =
            self.phase_narrate(state, &input, &world, &map, &player, &all_npcs)?;

        // If narration recorded an error on state, skip remaining phases
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
            match self.phase_engine_commit(&state, &narration_text, &quantifier_result) {
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

        if let Err(e) = save_message_and_snapshot(self.ctx, &mut next_state) {
            tracing::error!("Failed to save post-engine snapshot: {e}");
            next_state.narrative.input_buffer.status =
                GenerationStatus::Error(format!("Failed to save post-engine snapshot: {e}"));
            if let Err(e2) = save_state(self.ctx, &next_state) {
                tracing::error!("Failed to persist error state: {e2}");
            }
            // Skip trigger phases, go straight to finalize
            self.phase_finalize(&mut next_state);
            return Ok(());
        }

        if let Some(target) = next_state.narrative.retry_target.take() {
            next_state.narrative.history.append(target);
        }

        if let Some(request) = trigger_request {
            match self.phase_trigger_continuation(next_state.clone(), &request) {
                Ok((updated_state, continuation_text)) => {
                    next_state = updated_state;

                    if !continuation_text.is_empty() {
                        match self.phase_post_trigger_reconcile(
                            next_state.clone(),
                            &input,
                            &continuation_text,
                        ) {
                            Ok(s) => next_state = s,
                            Err(e) => {
                                tracing::error!("Post-trigger reconcile failed: {e:?}");
                                next_state.narrative.input_buffer.status =
                                    GenerationStatus::Error(format!("Trigger reconcile: {e:?}"));
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Trigger continuation failed: {e:?}");
                    next_state.narrative.input_buffer.status =
                        GenerationStatus::Error(format!("Trigger continuation: {e:?}"));
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
        if let Err(e) = save_message_and_snapshot(self.ctx, &mut state) {
            tracing::error!("Failed to save pre-main snapshot: {e}");
            state.narrative.input_buffer.status =
                GenerationStatus::Error(format!("Failed to save pre-main snapshot: {e}"));
            if let Err(e2) = save_state(self.ctx, &state) {
                tracing::error!("Failed to persist error state: {e2}");
            }
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
            state.narrative.input_buffer.status =
                GenerationStatus::Error("Room not found".to_string());
            if let Err(e) = save_state(self.ctx, &state) {
                tracing::error!("Failed to persist error state: {e}");
            }
            return Ok((state, String::new(), String::new(), String::new()));
        };
        let history = state.narrative.history();

        let (preset, response_length) = match self.load_preset_and_response_length() {
            Ok(p) => p,
            Err(msg) => {
                state.narrative.input_buffer.status = GenerationStatus::Error(msg);
                if let Err(e) = save_state(self.ctx, &state) {
                    tracing::error!("Failed to persist error state: {e}");
                }
                return Ok((state, String::new(), String::new(), String::new()));
            }
        };

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
            Err(e) => {
                let msg = map_llm_error(&e);
                state.narrative.input_buffer.status = GenerationStatus::Error(msg.clone());
                if let Err(e2) = save_state(self.ctx, &state) {
                    tracing::error!("Failed to persist error state: {e2}");
                }
                return Ok((state, String::new(), String::new(), String::new()));
            }
        };

        tracing::info!("Pipeline ▶ Narration LLM call (agent=narrator)");
        let narration_result = match self.service.complete(
            crate::narrative::llm::backend::AGENT_NARRATOR,
            &assembled.system_prompt,
            &assembled.user_prompt,
            Some(assembled.max_tokens),
        ) {
            Ok(result) => result,
            Err(e) => {
                let msg = map_llm_error(&e);
                state.narrative.input_buffer.status = GenerationStatus::Error(msg.clone());
                if let Err(e2) = save_state(self.ctx, &state) {
                    tracing::error!("Failed to persist error state: {e2}");
                }
                return Ok((state, String::new(), String::new(), String::new()));
            }
        };
        tracing::info!("Pipeline ✓ Narration complete");
        let narration_text = narration_result.text;

        if self.ctx.cancel_token.is_cancelled() {
            return Err(self.handle_cancellation());
        }

        if narration_text.trim().is_empty() {
            state.narrative.input_buffer.status =
                GenerationStatus::Error("LLM Error: empty response".to_string());
            if let Err(e) = save_state(self.ctx, &state) {
                tracing::error!("Failed to persist error state: {e}");
            }
            return Ok((state, String::new(), String::new(), String::new()));
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
        tracing::info!("Pipeline ▶ Quantifying");
        state.narrative.input_buffer.phase = GenerationPhase::Quantifying;
        if let Err(e) = save_message_and_snapshot(self.ctx, state) {
            tracing::warn!("Failed to save pre-quantifier phase update: {e}");
        }

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
        state: &GameState,
        narration_text: &str,
        quantifier_result: &QuantifierResult,
    ) -> Result<crate::engine::action_processing::TurnResult, EngineError> {
        execute_freeaction_impl(
            state,
            &FreeActionContext {
                narration_text,
                quantifier_result,
            },
        )
    }

    fn phase_trigger_continuation(
        &self,
        mut state: GameState,
        trigger: &StoredTriggerContext,
    ) -> PipelineResult<(GameState, String)> {
        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::GeneratingEvent;
        state.narrative.last_trigger = Some(trigger.clone());
        tracing::info!(
            "Pipeline ▶ GeneratingEvent (trigger={})",
            trigger.trigger_name
        );

        if self.ctx.cancel_token.is_cancelled() {
            return Err(self.handle_cancellation());
        }

        if let Err(e) = save_message_and_snapshot(self.ctx, &mut state) {
            tracing::error!("Failed to save pre-event snapshot: {e}");
            state.narrative.input_buffer.status =
                GenerationStatus::Error(format!("Failed to save pre-event snapshot: {e}"));
            if let Err(e2) = save_state(self.ctx, &state) {
                tracing::error!("Failed to persist error state: {e2}");
            }
            return Ok((state, String::new()));
        }

        tracing::info!("Pipeline ▶ Trigger LLM call (agent=trigger)");
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
                if let Err(e2) = save_message_and_snapshot(self.ctx, &mut state) {
                    tracing::error!("Failed to persist trigger error state: {e2}");
                }
                return Ok((state, String::new()));
            }
        };
        tracing::info!("Pipeline ✓ Trigger complete");
        let continuation_text = continuation_result.text;

        if self.ctx.cancel_token.is_cancelled() {
            return Err(self.handle_cancellation());
        }

        if continuation_text.trim().is_empty() {
            state.narrative.input_buffer.status =
                GenerationStatus::Error("LLM Error: empty response".to_string());
            if let Err(e2) = save_state(self.ctx, &state) {
                tracing::error!("Failed to persist error state: {e2}");
            }
            return Ok((state, String::new()));
        }

        state = match commit_trigger_narration(state.clone(), trigger, &continuation_text) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Trigger commit failed: {e}");
                state.narrative.input_buffer.status =
                    GenerationStatus::Error(format!("Trigger error: {e}"));
                if let Err(e2) = save_state(self.ctx, &state) {
                    tracing::error!("Failed to persist error state: {e2}");
                }
                return Ok((state, String::new()));
            }
        };

        if let Err(e) = save_message_and_snapshot(self.ctx, &mut state) {
            tracing::error!("Failed to save post-trigger snapshot: {e}");
            state.narrative.input_buffer.status =
                GenerationStatus::Error(format!("Failed to save post-trigger snapshot: {e}"));
            if let Err(e2) = save_state(self.ctx, &state) {
                tracing::error!("Failed to persist error state: {e2}");
            }
            return Ok((state, String::new()));
        }

        Ok((state, continuation_text))
    }

    fn phase_post_trigger_reconcile(
        &self,
        mut state: GameState,
        input: &str,
        continuation_text: &str,
    ) -> PipelineResult<GameState> {
        tracing::info!("Pipeline ▶ Post-trigger reconcile");
        match self.reconcile_post_trigger_npcs(state.clone(), input, continuation_text) {
            Ok(updated) => Ok(updated),
            Err(e) => {
                tracing::error!("Failed to apply post-trigger NPC events: {e}");
                state.narrative.input_buffer.status =
                    GenerationStatus::Error(format!("NPC event error: {e}"));
                if let Err(e2) = save_state(self.ctx, &state) {
                    tracing::error!("Failed to persist error state: {e2}");
                }
                Ok(state)
            }
        }
    }

    fn phase_finalize(&self, state: &mut GameState) {
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
        if let Err(e) = save_state(self.ctx, state) {
            tracing::error!("Failed to persist finished action: {e}");
        }
    }

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
                if let Err(e2) = save_state(self.ctx, &state) {
                    tracing::error!("Failed to persist error state: {e2}");
                }
                return ActionOutcome::Completed;
            }
        };
        let continuation_text = continuation_result.text;

        if continuation_text.trim().is_empty() {
            state.narrative.input_buffer.status =
                GenerationStatus::Error("LLM Error: empty response".to_string());
            if let Err(e2) = save_state(self.ctx, &state) {
                tracing::error!("Failed to persist error state: {e2}");
            }
            return ActionOutcome::Completed;
        }

        let mut committed_state =
            match commit_trigger_narration(state.clone(), &trigger, &continuation_text) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Trigger commit failed on retry: {e}");
                    state.narrative.input_buffer.status =
                        GenerationStatus::Error(format!("Trigger error: {e}"));
                    if let Err(e2) = save_state(self.ctx, &state) {
                        tracing::error!("Failed to persist error state: {e2}");
                    }
                    return ActionOutcome::Completed;
                }
            };

        if let Err(e) = save_message_and_snapshot(self.ctx, &mut committed_state) {
            tracing::error!("Failed to save post-trigger retry snapshot: {e}");
            committed_state.narrative.input_buffer.status =
                GenerationStatus::Error(format!("Failed to save post-trigger retry snapshot: {e}"));
            if let Err(e2) = save_state(self.ctx, &committed_state) {
                tracing::error!("Failed to persist error state: {e2}");
            }
            return ActionOutcome::Completed;
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
                if let Err(e2) = save_state(self.ctx, &committed_state) {
                    tracing::error!("Failed to persist error state: {e2}");
                }
                return ActionOutcome::Completed;
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

    fn load_preset_and_response_length(&self) -> Result<(PromptPreset, String), String> {
        let settings = self.ctx.settings.read().unwrap_or_else(|e| e.into_inner());
        let preset_id = settings.active_system_prompt_preset_id.clone();
        let response_length = settings.response_length.clone();
        match self.ctx.preset_storage.get_preset(&preset_id) {
            Ok(Some(p)) => Ok((p, response_length)),
            Ok(None) => {
                tracing::error!(
                    "active system preset '{preset_id}' not found — defaults not seeded?"
                );
                Err("Active system preset not found".to_string())
            }
            Err(e) => {
                tracing::error!("preset storage inaccessible: {e}");
                Err("Preset storage inaccessible".to_string())
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
