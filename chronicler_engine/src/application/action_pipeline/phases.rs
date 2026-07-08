//! [DOC: docs/system/game_flow.md]
//! Phase implementations for the action pipeline

use std::sync::Arc;

use crate::application::context::map_llm_error;
use crate::domain::engine::action_processing::{
    FreeActionContext, TriggerMatch, apply_npc_events, commit_trigger_narration,
    execute_freeaction_impl,
};
use crate::error::EngineError;
use crate::domain::model::character::{NpcCard, PlayerCard};
use crate::domain::model::map::MapDef;
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::quantifier::{QuantifierConfidence, QuantifierResult, compute_npc_events};
use crate::domain::model::state::trigger_context::StoredTriggerContext;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::world::WorldCard;
use crate::application::narrative_prompt::{NpcContext, build_narration_prompt, make_prompt_context};
use crate::application::application_service::DefaultApplicationService;
use crate::application::ports::llm_provider::{AGENT_NARRATOR, AGENT_TRIGGER};

use super::pipeline::{ActionOutcome, ActionPipeline, PipelineResult};

pub struct PipelineInputs {
    pub input: String,
    pub world: Arc<WorldCard>,
    pub map: Arc<MapDef>,
    pub player: Arc<PlayerCard>,
    pub all_npcs: Vec<NpcCard>,
}

pub(super) struct PipelineRun<'a> {
    pub(super) pipeline: &'a ActionPipeline,
    pub(super) app: &'a DefaultApplicationService,
}

impl<'a> PipelineRun<'a> {
    pub(super) fn new(pipeline: &'a ActionPipeline, app: &'a DefaultApplicationService) -> Self {
        Self { pipeline, app }
    }

    pub(super) fn persist(&self, state: &GameState) {
        if let Err(e) = self.app.save_state(state) {
            tracing::error!("Failed to persist state: {e}");
        }
    }

    pub(super) fn persist_snapshot_failed(&self, state: &mut GameState, label: &str) -> bool {
        if let Err(e) = self.app.save_message_and_snapshot(state) {
            tracing::error!("Failed to save {label}: {e}");
            state.narrative.input_buffer.status =
                GenerationStatus::Error(format!("Failed to save {label}: {e}"));
            self.persist(state);
            true
        } else {
            false
        }
    }

    pub(super) fn error_return(
        &self,
        mut state: GameState,
        msg: String,
    ) -> PipelineResult<(GameState, String, String, String)> {
        state.narrative.input_buffer.status = GenerationStatus::Error(msg);
        self.persist(&state);
        Ok((state, String::new(), String::new(), String::new()))
    }

    pub(super) fn phase_narrate(
        &self,
        mut state: GameState,
        inputs: &PipelineInputs,
    ) -> PipelineResult<(GameState, String, String, String)> {
        let Some(room) = inputs.map.get_room_by_id(&state.movement.current_room_id) else {
            return self.error_return(state, "Room not found".to_string());
        };
        let history = state.narrative.history();

        let (preset, response_length) = match self.load_preset_and_response_length() {
            Ok(p) => p,
            Err(msg) => return self.error_return(state, msg),
        };

        let context = make_prompt_context(
            &inputs.world,
            room,
            NpcContext {
                all_npcs: &inputs.all_npcs,
                npcs_in_area: &state.scene.npcs_in_area,
            },
            &inputs.player,
            &inputs.input,
            &history,
        );

        let assembled = match build_narration_prompt(
            &context,
            &preset,
            &inputs.world.global_rules,
            Some(&response_length),
            self.pipeline.assembler.max_context_tokens,
            self.pipeline.assembler.max_tokens,
        ) {
            Ok(a) => a,
            Err(e) => return self.error_return(state, map_llm_error(&e)),
        };

        tracing::info!("Pipeline ▶ Narration LLM call (agent=narrator)");
        let narration_result = match self.pipeline.recorder.complete(
            AGENT_NARRATOR,
            &assembled.system_prompt,
            &assembled.user_prompt,
            Some(assembled.max_tokens),
        ) {
            Ok(result) => result,
            Err(e) => return self.error_return(state, map_llm_error(&e)),
        };
        tracing::info!("Pipeline ✓ Narration complete");
        let narration_text = narration_result.text;

        if self.app.cancel_token.is_cancelled() {
            return Err(ActionOutcome::Cancelled);
        }

        if narration_text.trim().is_empty() {
            return self.error_return(state, "LLM Error: empty response".to_string());
        }

        state.add_message(narration_text.clone(), None, MessageType::Narration);
        if let Err(e) = self.app.save_message_and_snapshot(&mut state) {
            tracing::warn!("Failed to save pre-quantifier narration: {e}");
        }

        Ok((
            state,
            narration_text,
            narration_result.backend_name,
            narration_result.model_name,
        ))
    }

    pub(super) fn phase_post_generation(
        &self,
        state: &mut GameState,
        input: &str,
        narration_text: &str,
    ) -> QuantifierResult {
        tracing::info!("Pipeline ▶ Quantifying");
        state.narrative.input_buffer.phase = GenerationPhase::Quantifying;
        if let Err(e) = self.app.save_message_and_snapshot(state) {
            tracing::warn!("Failed to save pre-quantifier phase update: {e}");
        }

        let mut quantifier_result =
            self.pipeline
                .run_post_generation_agents(state, input, narration_text);

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

    pub(super) fn phase_trigger_continuation_llm_call(
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

        if self.app.cancel_token.is_cancelled() {
            return Err(ActionOutcome::Cancelled);
        }

        if self.persist_snapshot_failed(&mut state, "pre-event snapshot") {
            return Ok((state, String::new()));
        }

        tracing::info!("Pipeline ▶ Trigger LLM call (agent=trigger)");
        let continuation_result = match self.pipeline.recorder.complete(
            AGENT_TRIGGER,
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
                if let Err(e2) = self.app.save_message_and_snapshot(&mut state) {
                    tracing::error!("Failed to persist trigger error state: {e2}");
                }
                return Ok((state, String::new()));
            }
        };
        tracing::info!("Pipeline ✓ Trigger complete");
        let continuation_text = continuation_result.text;

        if self.app.cancel_token.is_cancelled() {
            return Err(ActionOutcome::Cancelled);
        }

        if continuation_text.trim().is_empty() {
            state.narrative.input_buffer.status =
                GenerationStatus::Error("LLM Error: empty response".to_string());
            self.persist(&state);
            return Ok((state, String::new()));
        }

        state = match commit_trigger_narration(state.clone(), trigger, &continuation_text) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Trigger commit failed: {e}");
                state.narrative.input_buffer.status =
                    GenerationStatus::Error(format!("Trigger error: {e}"));
                self.persist(&state);
                return Ok((state, String::new()));
            }
        };

        if self.persist_snapshot_failed(&mut state, "post-trigger snapshot") {
            return Ok((state, String::new()));
        }

        Ok((state, continuation_text))
    }

    pub(super) fn reconcile_post_trigger_npcs(
        &self,
        mut state: GameState,
        player_input: &str,
        continuation_text: &str,
    ) -> GameState {
        tracing::info!("Pipeline ▶ Post-trigger reconcile");
        state.narrative.input_buffer.phase = GenerationPhase::Quantifying;

        let previous_ids: Vec<String> = state
            .scene
            .npcs_in_area
            .iter()
            .map(|n| n.id.clone())
            .collect();
        let post_trigger_result =
            self.pipeline
                .run_post_generation_agents(&state, player_input, continuation_text);

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
        match apply_npc_events(state.clone(), &events.events) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Post-trigger reconcile failed: {e}");
                state.narrative.input_buffer.status =
                    GenerationStatus::Error(format!("Trigger reconcile: {e}"));
                state
            }
        }
    }

    pub(super) fn build_trigger_request(
        &self,
        state: &GameState,
        narration_text: &str,
        inputs: &PipelineInputs,
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
            &inputs.world,
            room_data,
            NpcContext {
                all_npcs: &inputs.all_npcs,
                npcs_in_area: &state.scene.npcs_in_area,
            },
            &inputs.player,
            &continuation_user_msg,
            &history,
        );

        let assembled = self
            .pipeline
            .assembler
            .assemble(
                &trigger_ctx,
                &preset,
                &inputs.world.global_rules,
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

    pub(super) fn load_preset_and_response_length(&self) -> Result<(PromptPreset, String), String> {
        let settings = self.app.settings.read().unwrap_or_else(|e| e.into_inner());
        let preset_id = settings.active_system_prompt_preset_id.clone();
        let response_length = settings.response_length.clone();
        match self.app.preset_storage.get_preset(&preset_id) {
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
}

impl ActionPipeline {
    pub(super) fn phase_engine_commit(
        state: &GameState,
        narration_text: &str,
        quantifier_result: &QuantifierResult,
    ) -> Result<crate::domain::engine::action_processing::TurnResult, EngineError> {
        execute_freeaction_impl(
            state,
            &FreeActionContext {
                narration_text,
                quantifier_result,
            },
        )
    }
}
