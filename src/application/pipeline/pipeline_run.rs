//! [DOC: docs/diataxis/reference/game_flow.md]
//! PipelineRun and its phase implementations for the action pipeline.

use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::model::state::game_state::{GameState, TriggerMatch};
use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::map::MapDef;
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::quantifier::{NpcEventList, QuantifierConfidence, QuantifierResult};
use crate::domain::model::state::trigger_context::StoredTriggerContext;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::world::WorldCard;
use crate::application::prompting::{NpcContext, PromptContext};
use crate::application::ports::llm_provider::{AGENT_NARRATOR, AGENT_TRIGGER};

use super::phase_error::PhaseError;
use super::action_pipeline::core::ActionPipeline;

pub struct PipelineInputs {
    pub input: String,
    pub world: Arc<WorldCard>,
    pub map: Arc<MapDef>,
    pub persona: Arc<PersonaCard>,
    pub all_npcs: Vec<NpcCard>,
    /// Transient guided-generation instruction for this turn (`None` on plain
    /// turns; on retry, sourced from `retry_target.replay().guide` instead).
    pub guide: Option<String>,
    /// Impersonate steering for this turn. When true, the narration runs as the
    /// player's persona: the impersonate preset replaces the system preset, the
    /// player-character layer is dropped, and the output is a player-voiced
    /// `Dialogue` message. Retry re-derives it from `retry_target.replay().impersonate`.
    pub impersonate: bool,
    /// Optional `/impersonate <direction>` text; fed to the prompt as the
    /// instruction for the impersonated turn. `None` for plain `/impersonate`.
    pub impersonate_direction: Option<String>,
    /// Impersonate preset id to load (staged by the seam from settings, or read
    /// from the retry-target replay blob). Falls back to
    /// `active_impersonate_prompt_preset_id` when `None`.
    pub impersonate_preset_id: Option<String>,
}

pub(super) struct PipelineRun<'a> {
    pub(super) pipeline: &'a ActionPipeline,
    pub(super) started_for: u64,
}

impl<'a> PipelineRun<'a> {
    pub(super) fn new(pipeline: &'a ActionPipeline, started_for: u64) -> Self {
        Self {
            pipeline,
            started_for,
        }
    }

    fn check_game_unchanged(&self, started_for: u64) -> Result<(), PhaseError> {
        let current = self.pipeline.storage.current_game_id();
        if current != started_for {
            tracing::info!(
                started = started_for,
                current = current,
                "Pipeline aborting: game changed — discarding in-flight generation"
            );
            return Err(PhaseError::Cancelled);
        }
        Ok(())
    }

    pub(super) fn persist(&self, state: &GameState) {
        if let Err(e) = self.pipeline.message_service.save_state(state) {
            tracing::error!("Failed to persist state: {e}");
        }
    }

    pub(super) fn persist_snapshot_or_err(
        &self,
        state: &mut GameState,
        label: &'static str,
    ) -> Result<(), PhaseError> {
        if let Err(source) = self
            .pipeline
            .message_service
            .save_message_and_snapshot(state)
        {
            tracing::error!("Failed to save {label}: {source}");
            state.narrative.input_buffer.status =
                GenerationStatus::Error(format!("Failed to save {label}: {source}"));
            self.persist(state);
            Err(PhaseError::PersistFailed { label, source })
        } else {
            Ok(())
        }
    }

    fn set_error(&self, state: &mut GameState, msg: String) -> PhaseError {
        state.narrative.input_buffer.status = GenerationStatus::Error(msg.clone());
        PhaseError::NarratorFailed(msg)
    }

    pub(super) fn phase_narrate(
        &self,
        state: &mut GameState,
        inputs: &PipelineInputs,
    ) -> Result<(String, String, String), PhaseError> {
        let Some(room) = inputs
            .map
            .get_room_by_id(&state.movement.current_room_id)
            .or_else(|| {
                state
                    .movement
                    .dynamic_rooms
                    .get(&state.movement.current_room_id)
            })
        else {
            return Err(self.set_error(state, "Room not found".to_string()));
        };
        let history = state.narrative.history();

        let impersonate = self.resolve_impersonate(state, inputs);

        let (preset, response_length) = if let Some((_direction, preset_id)) = &impersonate {
            match self.load_impersonate_preset_and_response_length(preset_id.as_deref()) {
                Ok(p) => p,
                Err(msg) => return Err(self.set_error(state, msg)),
            }
        } else {
            match self.load_preset_and_response_length() {
                Ok(p) => p,
                Err(msg) => return Err(self.set_error(state, msg)),
            }
        };

        let impersonate_direction = impersonate
            .as_ref()
            .and_then(|(direction, _)| direction.clone())
            .unwrap_or_default();
        let user_message: &str = if impersonate.is_some() {
            &impersonate_direction
        } else {
            &inputs.input
        };

        let context = PromptContext::new(
            &inputs.world,
            room,
            NpcContext {
                all_npcs: &inputs.all_npcs,
                npcs_in_area: &state.scene.npcs_in_area,
            },
            &inputs.persona,
            user_message,
            &history,
        );
        let context = if impersonate.is_some() {
            context.with_impersonate(true)
        } else {
            context.with_guide(self.resolve_guide(state, &inputs.guide))
        };

        let assembled = match self.pipeline.prompt_assembler.assemble(
            &context,
            &preset,
            &inputs.world.global_rules,
            Some(&response_length),
        ) {
            Ok(a) => a,
            Err(e) => return Err(self.set_error(state, e.llm_error_string())),
        };

        tracing::info!("Pipeline ▶ Narration LLM call (agent=narrator)");
        let narration_result = match self.pipeline.recorder.complete(
            AGENT_NARRATOR,
            &assembled.system_prompt,
            &assembled.user_prompt,
            Some(assembled.max_tokens),
        ) {
            Ok(result) => result,
            Err(e) => return Err(self.set_error(state, e.llm_error_string())),
        };
        tracing::info!("Pipeline ✓ Narration complete");
        let narration_text = narration_result.text;

        self.check_game_unchanged(self.started_for)?;

        if narration_text.trim().is_empty() {
            return Err(self.set_error(state, "LLM Error: empty response".to_string()));
        }

        let (sender, message_type) = if impersonate.is_some() {
            (
                Some(inputs.persona.sheet.name.clone()),
                MessageType::Dialogue,
            )
        } else {
            (None, MessageType::Narration)
        };
        state.add_message(narration_text.clone(), sender, message_type);
        self.persist_snapshot_or_err(state, "pre-quantifier narration")?;

        Ok((
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
        map: &Arc<MapDef>,
        persona: &Arc<PersonaCard>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<QuantifierResult, PhaseError> {
        tracing::info!("Pipeline ▶ Quantifying");
        state.narrative.input_buffer.phase = GenerationPhase::Quantifying;
        self.persist_snapshot_or_err(state, "pre-quantifier phase update")?;

        let mut quantifier_result = self.pipeline.run_post_generation_agents(
            state,
            input,
            narration_text,
            map,
            persona,
            npcs,
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

        // Best-effort: quantifier metadata (swipes) is not load-bearing for the turn commit.
        if let Err(e) = self
            .pipeline
            .message_service
            .save_message_and_snapshot(state)
        {
            tracing::warn!("Failed to save post-quantifier metadata: {e}");
        }

        Ok(quantifier_result)
    }

    pub(super) fn phase_trigger_continuation_llm_call(
        &self,
        state: &mut GameState,
        trigger: &StoredTriggerContext,
        map: &Arc<MapDef>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<String, PhaseError> {
        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::GeneratingEvent;
        state.narrative.last_trigger = Some(trigger.clone());
        tracing::info!(
            "Pipeline ▶ GeneratingEvent (trigger={})",
            trigger.trigger_name
        );

        self.check_game_unchanged(self.started_for)?;

        self.persist_snapshot_or_err(state, "pre-event snapshot")?;

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
                return Err(self.set_error(state, format!("Trigger narration failed: {e}")));
            }
        };
        tracing::info!("Pipeline ✓ Trigger complete");
        let continuation_text = continuation_result.text;

        self.check_game_unchanged(self.started_for)?;

        if continuation_text.trim().is_empty() {
            return Err(self.set_error(state, "LLM Error: empty response".to_string()));
        }

        if let Err(e) = state.commit_trigger_narration(trigger, &continuation_text, map, npcs) {
            tracing::error!("Trigger commit failed: {e}");
            return Err(self.set_error(state, format!("Trigger error: {e}")));
        }

        self.persist_snapshot_or_err(state, "post-trigger snapshot")?;

        Ok(continuation_text)
    }

    pub(super) fn reconcile_post_trigger_npcs(
        &self,
        state: &mut GameState,
        player_input: &str,
        continuation_text: &str,
        map: &Arc<MapDef>,
        persona: &Arc<PersonaCard>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<(), PhaseError> {
        tracing::info!("Pipeline ▶ Post-trigger reconcile");
        state.narrative.input_buffer.phase = GenerationPhase::Quantifying;

        let previous_ids: Vec<String> = state
            .scene
            .npcs_in_area
            .iter()
            .map(|n| n.id.clone())
            .collect();
        let post_trigger_result = self.pipeline.run_post_generation_agents(
            state,
            player_input,
            continuation_text,
            map,
            persona,
            npcs,
        );

        state.scene.quantifier_confidence =
            Some(format!("{:?}", post_trigger_result.npcs.confidence));

        let npc_cards: Vec<NpcCard> = post_trigger_result
            .npcs
            .npc_ids
            .iter()
            .filter_map(|id| npcs.get(id).cloned())
            .collect();
        let new_ids: Vec<String> = npc_cards.iter().map(|n| n.id.clone()).collect();

        state.scene.npcs_in_area = npc_cards;

        let events = NpcEventList::from_diff(&previous_ids, &new_ids);
        if let Err(e) = state.apply_npc_events(&events.events, map, npcs) {
            tracing::error!("Post-trigger reconcile failed: {e}");
            return Err(self.set_error(state, format!("Trigger reconcile: {e}")));
        }
        Ok(())
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

        let room_data = inputs
            .map
            .get_room_by_id(&state.movement.current_room_id)
            .or_else(|| {
                state
                    .movement
                    .dynamic_rooms
                    .get(&state.movement.current_room_id)
            })?;
        let history = state.narrative.history();

        let (preset, response_length) = self.load_preset_and_response_length().ok()?;

        let trigger_ctx = PromptContext::new(
            &inputs.world,
            room_data,
            NpcContext {
                all_npcs: &inputs.all_npcs,
                npcs_in_area: &state.scene.npcs_in_area,
            },
            &inputs.persona,
            &continuation_user_msg,
            &history,
        );

        let assembled = self
            .pipeline
            .prompt_assembler
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

    /// Resolve the guided-generation instruction for the in-flight turn.
    /// New guide generations carry it via `inputs.guide` (staged from the
    /// `/guide` slash command); retry re-applies it from the replay blob on the
    /// retry-target swipe.
    fn resolve_guide(&self, state: &GameState, inputs_guide: &Option<String>) -> Option<String> {
        if let Some(g) = inputs_guide {
            return Some(g.clone());
        }
        state
            .narrative
            .retry_target
            .as_ref()
            .and_then(|m| m.replay())
            .and_then(|r| r.guide.clone())
    }

    /// Resolve the impersonate steering for the in-flight turn, if any. New
    /// impersonate generations carry direction/preset id via `inputs` (staged
    /// from `/impersonate`); retry re-applies them from the retry-target replay
    /// blob. Returns `(direction, preset_id)`.
    fn resolve_impersonate(
        &self,
        state: &GameState,
        inputs: &PipelineInputs,
    ) -> Option<(Option<String>, Option<String>)> {
        if inputs.impersonate {
            return Some((
                inputs.impersonate_direction.clone(),
                inputs.impersonate_preset_id.clone(),
            ));
        }
        state
            .narrative
            .retry_target
            .as_ref()
            .and_then(|m| m.replay())
            .filter(|r| r.impersonate)
            .map(|r| {
                (
                    r.impersonate_direction.clone(),
                    r.impersonate_preset_id.clone(),
                )
            })
    }

    pub(super) fn load_preset_and_response_length(&self) -> Result<(PromptPreset, String), String> {
        let settings = self
            .pipeline
            .settings
            .read()
            .unwrap_or_else(|e| e.into_inner());
        let preset_id = settings.active_system_prompt_preset_id.clone();
        let response_length = settings.response_length.clone();
        match self.pipeline.storage.get_preset(&preset_id) {
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

    /// Load the impersonate preset for an impersonated turn. `preset_id` is the
    /// blob's recorded id (staged from `active_impersonate_prompt_preset_id` by
    /// the seam, or carried by the retry-target replay); when `None`/empty it
    /// falls back to the current setting.
    fn load_impersonate_preset_and_response_length(
        &self,
        preset_id: Option<&str>,
    ) -> Result<(PromptPreset, String), String> {
        let settings = self
            .pipeline
            .settings
            .read()
            .unwrap_or_else(|e| e.into_inner());
        let response_length = settings.response_length.clone();
        let preset_id = preset_id
            .filter(|id| !id.is_empty())
            .unwrap_or(&settings.active_impersonate_prompt_preset_id);
        match self.pipeline.storage.get_preset(preset_id) {
            Ok(Some(p)) => Ok((p, response_length)),
            Ok(None) => {
                tracing::error!(
                    "active impersonate preset '{preset_id}' not found — defaults not seeded?"
                );
                Err("Active impersonate preset not found".to_string())
            }
            Err(e) => {
                tracing::error!("preset storage inaccessible: {e}");
                Err("Preset storage inaccessible".to_string())
            }
        }
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
