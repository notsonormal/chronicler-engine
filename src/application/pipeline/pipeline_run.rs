//! [DOC: docs/diataxis/reference/game_flow.md]
//! PipelineRun and its phase implementations for the action pipeline.

use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::model::agent::{
    AgentSettings, EventRecord, HookInput, Kind, StoryEntry, WorldView, STORY_WINDOW,
};
use crate::domain::model::game::GameId;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::map::MapDef;
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::quantifier::{NpcEventList, QuantifierConfidence, QuantifierResult};
use crate::domain::model::state::trigger_context::StoredTriggerContext;
use crate::domain::model::utils::game_rules::TriggerMatch;
use crate::domain::model::state::generation_status::{
    GenerationFailure, GenerationFailureKind, GenerationPhase, GenerationStatus,
};
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::settings::{NarrativePerspective, NarrativeTense};
use crate::domain::model::world::WorldCard;
use crate::application::prompting::token_budget::TokenBudget;
use crate::application::prompting::{NpcContext, PromptContext};
use crate::application::ports::llm_provider::AGENT_TRIGGER;
use crate::error::EngineError;

use crate::adapters::driven::storage::worlds::WorldBundle;
use crate::adapters::driven::storage::Storage;
use crate::domain::model::map::Room;
use crate::domain::model::settings::AppSettings;

use super::phase_error::PhaseError;
use super::action_pipeline::core::ActionPipeline;

pub const UNCERTAIN_NPC_NOTICE: &str = "[System] NPC detection uncertain — using room defaults";

pub(crate) struct PipelineRun<'a> {
    pub(super) pipeline: &'a ActionPipeline,
    pub(super) started_for: GameId,
}

impl<'a> PipelineRun<'a> {
    pub(super) fn new(pipeline: &'a ActionPipeline, started_for: GameId) -> Self {
        Self {
            pipeline,
            started_for,
        }
    }

    pub(super) fn check_game_unchanged(&self, started_for: GameId) -> Result<(), PhaseError> {
        let current = self.pipeline.game_catalogue.active_game_id();
        if current != Some(started_for) {
            tracing::info!(
                started = %started_for,
                current = ?current,
                "Pipeline aborting: game changed — discarding in-flight generation"
            );
            return Err(PhaseError::Cancelled);
        }
        Ok(())
    }

    pub(super) fn persist(&self, state: &GameState) {
        if let Err(e) = self
            .pipeline
            .message_service
            .save_state(self.started_for, state)
        {
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
            .save_message_and_snapshot(self.started_for, state)
        {
            tracing::error!("Failed to save {label}: {source}");
            state.narrative.input_buffer.status =
                GenerationStatus::Error(Self::save_failure(label, &source));
            self.persist(state);
            Err(PhaseError::PersistFailed { label, source })
        } else {
            Ok(())
        }
    }

    pub(crate) fn save_failure(label: &'static str, source: &EngineError) -> GenerationFailure {
        GenerationFailure::from_engine_error(source).with_context(format!("Failed to save {label}"))
    }

    pub(super) fn set_error(
        &self,
        state: &mut GameState,
        failure: GenerationFailure,
    ) -> PhaseError {
        state.narrative.input_buffer.status = GenerationStatus::Error(failure.clone());
        PhaseError::NarratorFailed(failure)
    }

    /// Read per call, so a connection edit takes effect on the next call.
    pub(super) fn narration_budget(&self) -> Result<TokenBudget, GenerationFailure> {
        let settings = self
            .pipeline
            .storage
            .get_settings()
            .map_err(|e| GenerationFailure::from_engine_error(&e))?;
        let connection = settings
            .narration_connection()
            .map_err(|e| GenerationFailure::from_engine_error(&e))?;
        Ok(TokenBudget {
            max_context_tokens: connection.resolve_max_context_tokens(),
            max_tokens: connection.max_tokens,
        })
    }

    fn preset_text(&self, pick: fn(&Storage, GameId, &AppSettings) -> String) -> Option<String> {
        let settings = self.pipeline.storage.get_settings().ok()?;
        let preset_id = pick(&self.pipeline.storage, self.started_for, &settings);
        self.pipeline
            .storage
            .get_preset(&preset_id)
            .ok()
            .flatten()
            .map(|preset| preset.assemble_text(&[], None, None))
    }

    pub(super) fn analysis_settings(&self) -> AgentSettings {
        AgentSettings {
            preset: self.preset_text(Storage::active_quantifier_preset_id),
            max_tokens: None,
        }
    }

    pub(super) fn options_settings(&self) -> AgentSettings {
        AgentSettings {
            preset: self.preset_text(Storage::active_options_preset_id),
            max_tokens: None,
        }
    }

    /// The room the generation happens in, as the map and the state resolve it.
    pub(super) fn room_of<'s>(state: &'s GameState, map: &'s MapDef) -> Option<&'s Room> {
        let room_id = &state.swipe.movement.current_room_id;
        map.get_room_by_id(room_id)
            .or_else(|| state.swipe.movement.dynamic_rooms.get(room_id))
    }

    /// The kind of the generation that produced the scene on screen: an event
    /// Narration when the last Message is one, else a Narration. The Options run
    /// generates nothing itself, so it reports the generation whose scene it reads.
    pub(super) fn scene_kind(state: &GameState) -> Kind {
        match state.narrative.history().last() {
            Some(entry) if entry.event_header.is_some() => Kind::Event,
            _ => Kind::Narration,
        }
    }

    /// Builds the values one hook's Agents read from the state the Job holds,
    /// and hands them to `run`. The story window, the event record and the world
    /// view live here, so the `HookInput` can borrow them for the one call.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn with_hook_input<R>(
        state: &GameState,
        map: &MapDef,
        persona: &PersonaCard,
        npcs: &HashMap<String, NpcCard>,
        text: Option<&str>,
        player_text: &str,
        kind: Kind,
        run: impl FnOnce(&HookInput) -> R,
    ) -> R {
        let story: Vec<StoryEntry> = state
            .narrative
            .history()
            .iter()
            .rev()
            .take(STORY_WINDOW)
            .rev()
            .map(StoryEntry::from)
            .collect();
        let event = state.narrative.last_trigger.as_ref().map(EventRecord::from);
        let world = WorldView {
            room: Self::room_of(state, map),
            map,
            persona,
            characters: npcs,
            present: &state.swipe.scene.npcs_in_area,
        };
        let input = HookInput {
            kind,
            text,
            player_text,
            story: &story,
            world: &world,
            event: event.as_ref(),
        };
        run(&input)
    }

    /// Runs the analysis hook for one generation.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn run_analysis_hook(
        &self,
        state: &GameState,
        map: &MapDef,
        persona: &PersonaCard,
        npcs: &HashMap<String, NpcCard>,
        text: &str,
        player_text: &str,
        kind: Kind,
    ) -> QuantifierResult {
        Self::with_hook_input(
            state,
            map,
            persona,
            npcs,
            Some(text),
            player_text,
            kind,
            |input| {
                self.pipeline
                    .agent_runner
                    .analysis(input, &self.analysis_settings())
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn phase_post_generation(
        &self,
        state: &mut GameState,
        input: &str,
        narration_text: &str,
        map: &Arc<MapDef>,
        persona: &Arc<PersonaCard>,
        npcs: &HashMap<String, NpcCard>,
        kind: Kind,
    ) -> Result<QuantifierResult, PhaseError> {
        tracing::info!("Pipeline ▶ Quantifying");
        state.narrative.input_buffer.phase = GenerationPhase::Quantifying;
        self.persist_snapshot_or_err(state, "pre-quantifier phase update")?;

        let mut quantifier_result =
            self.run_analysis_hook(state, map, persona, npcs, narration_text, input, kind);

        state.swipe.scene.quantifier_confidence =
            Some(format!("{:?}", quantifier_result.npcs.confidence));

        if quantifier_result.npcs.npc_ids.is_empty() && !quantifier_result.npcs.confidence.is_high()
        {
            let room_default_npcs = state
                .swipe
                .scene
                .npcs_in_area
                .iter()
                .map(|n| n.id.clone())
                .collect();
            quantifier_result.npcs.npc_ids = room_default_npcs;
            quantifier_result.npcs.confidence = QuantifierConfidence::Low;
            state.add_message(UNCERTAIN_NPC_NOTICE.to_string(), MessageType::System);
        }

        // Best-effort: the turn commit does not depend on quantifier metadata (swipes).
        if let Err(e) = self
            .pipeline
            .message_service
            .save_message_and_snapshot(self.started_for, state)
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
                return Err(self.set_error(
                    state,
                    GenerationFailure::from_engine_error(&e)
                        .with_context("Trigger narration failed"),
                ));
            }
        };
        tracing::info!("Pipeline ✓ Trigger complete");
        let continuation_text = continuation_result.text;

        self.check_game_unchanged(self.started_for)?;

        if continuation_text.trim().is_empty() {
            return Err(self.set_error(
                state,
                GenerationFailure::new(
                    GenerationFailureKind::UnreadableAnswer,
                    "LLM Error: empty response",
                ),
            ));
        }

        if let Err(e) = state.commit_trigger_narration(trigger, &continuation_text, map, npcs) {
            tracing::error!("Trigger commit failed: {e}");
            return Err(self.set_error(
                state,
                GenerationFailure::from_engine_error(&e).with_context("Trigger error"),
            ));
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
            .swipe
            .scene
            .npcs_in_area
            .iter()
            .map(|n| n.id.clone())
            .collect();
        let post_trigger_result = self.run_analysis_hook(
            state,
            map,
            persona,
            npcs,
            continuation_text,
            player_input,
            Kind::Event,
        );

        state.swipe.scene.quantifier_confidence =
            Some(format!("{:?}", post_trigger_result.npcs.confidence));

        let npc_cards: Vec<NpcCard> = post_trigger_result
            .npcs
            .npc_ids
            .iter()
            .filter_map(|id| npcs.get(id).cloned())
            .collect();
        let new_ids: Vec<String> = npc_cards.iter().map(|n| n.id.clone()).collect();

        state.swipe.scene.npcs_in_area = npc_cards;

        let events = NpcEventList::from_diff(&previous_ids, &new_ids);
        if let Err(e) = state.apply_npc_events(&events.events, map, npcs) {
            tracing::error!("Post-trigger reconcile failed: {e}");
            return Err(self.set_error(
                state,
                GenerationFailure::from_engine_error(&e).with_context("Trigger reconcile"),
            ));
        }
        Ok(())
    }

    pub(super) fn build_trigger_request(
        &self,
        state: &GameState,
        narration_text: &str,
        bundle: &WorldBundle,
        trigger_match: &TriggerMatch,
    ) -> Option<StoredTriggerContext> {
        let continuation_user_msg = format!(
            "Previous narration:\n{}\n\nTrigger event: {}\n\n\
             Continue the scene naturally, incorporating the trigger event into the narrative. \
             Do NOT repeat or contradict what was already described. Build naturally on the existing scene.",
            narration_text, trigger_match.trigger_narration_prompt
        );

        let room_data = bundle
            .map
            .get_room_by_id(&state.swipe.movement.current_room_id)
            .or_else(|| {
                state
                    .swipe
                    .movement
                    .dynamic_rooms
                    .get(&state.swipe.movement.current_room_id)
            })?;
        let history = state.narrative.history();

        let preset_id = {
            let settings = self.pipeline.storage.get_settings().ok()?;
            self.pipeline
                .storage
                .active_system_preset_id(self.started_for, &settings)
        };
        let (preset, response_length) = self
            .load_preset_and_response_length(&preset_id, PresetKind::System)
            .ok()?;

        let budget = self.narration_budget().ok()?;
        let (narrative_perspective, narrative_tense) = self.resolve_posture(&bundle.world);
        let all_npcs: Vec<NpcCard> = bundle.npcs.values().cloned().collect();
        let trigger_ctx = PromptContext::new(
            &bundle.world,
            narrative_perspective,
            narrative_tense,
            room_data,
            NpcContext {
                all_npcs: &all_npcs,
                npcs_in_area: &state.swipe.scene.npcs_in_area,
            },
            &bundle.persona,
            &continuation_user_msg,
            &history,
        );

        let assembled = self
            .pipeline
            .narrator
            .assemble(&trigger_ctx, &preset, &response_length, budget)
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

    pub(super) fn resolve_posture(
        &self,
        world: &WorldCard,
    ) -> (NarrativePerspective, NarrativeTense) {
        match self.pipeline.storage.get_game(self.started_for.0) {
            Ok(Some(game)) => (game.narrative_perspective, game.narrative_tense),
            _ => {
                tracing::warn!("game row unreadable; falling back to world posture");
                (world.narrative_perspective, world.narrative_tense)
            }
        }
    }

    pub(super) fn resolve_options_always_on(&self, world: &WorldCard) -> bool {
        match self.pipeline.storage.get_game(self.started_for.0) {
            Ok(Some(game)) => game.options_always_on,
            _ => {
                tracing::warn!("game row unreadable; falling back to world options toggle");
                world.options_always_on
            }
        }
    }

    pub(super) fn load_preset_and_response_length(
        &self,
        preset_id: &str,
        kind: PresetKind,
    ) -> Result<(PromptPreset, String), GenerationFailure> {
        let response_length = self
            .pipeline
            .storage
            .get_settings()
            .map_err(|e| {
                GenerationFailure::new(
                    GenerationFailureKind::Other,
                    format!("Failed to read settings: {e}"),
                )
            })?
            .response_length
            .clone();
        match self.pipeline.storage.get_preset(preset_id) {
            Ok(Some(p)) => Ok((p, response_length)),
            Ok(None) => {
                tracing::error!(
                    "active {} preset '{preset_id}' not found — defaults not seeded?",
                    kind.label()
                );
                Err(GenerationFailure::from_engine_error(
                    &EngineError::PresetNotFound(preset_id.to_string()),
                ))
            }
            Err(e) => {
                tracing::error!("preset storage inaccessible: {e}");
                Err(GenerationFailure::new(
                    GenerationFailureKind::Other,
                    "Preset storage inaccessible",
                ))
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum PresetKind {
    /// The active system preset (narration and trigger generations).
    System,
    /// The active impersonate preset (impersonate generations).
    Impersonate,
}

impl PresetKind {
    fn label(self) -> &'static str {
        match self {
            PresetKind::System => "system",
            PresetKind::Impersonate => "impersonate",
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

        if state.narrative.input_buffer.status.failure().is_none() {
            state.narrative.input_buffer.status = GenerationStatus::Idle;
        }
        state.narrative.input_buffer.phase = GenerationPhase::default();
        self.persist(state);
    }

    pub(super) fn handle_cancellation(&self) -> PhaseError {
        tracing::warn!("Pipeline cancelled — aborting remaining stages");
        let mut state = self
            .pipeline
            .message_service
            .load_or_fresh(self.started_for);
        state.narrative.input_buffer.status = GenerationStatus::Idle;
        state.narrative.input_buffer.phase = GenerationPhase::default();
        self.persist(&state);
        PhaseError::Cancelled
    }
}
