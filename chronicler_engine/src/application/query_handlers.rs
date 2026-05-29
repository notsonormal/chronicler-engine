//! [DOC: docs/architecture/system.md]
//! Read-only query handlers: generation status, game info, room view, NPCs, debug state.

use crate::application::ApplicationError;
use crate::application::DebugStateView;
use crate::application::context::GameServiceContext;
use crate::error::EngineError;
use crate::model::llm_message::LlmMessage;
use crate::model::state::LogEntry;

/// Read-only query operations for DefaultApplicationService.
pub struct QueryHandlers;

impl Default for QueryHandlers {
    fn default() -> Self {
        Self
    }
}

impl QueryHandlers {
    pub fn new() -> Self {
        Self
    }

    fn load_state(
        &self,
        ctx: &GameServiceContext,
    ) -> Result<crate::model::state::GameState, ApplicationError> {
        crate::application::context::try_load_state(ctx).map_err(Into::into)
    }

    /// Returns the current generation status and phase.
    pub fn get_generating_status(
        &self,
        ctx: GameServiceContext,
    ) -> Result<
        (
            crate::model::state::GenerationStatus,
            crate::model::state::GenerationPhase,
        ),
        ApplicationError,
    > {
        let game_state = self.load_state(&ctx)?;
        Ok((
            game_state.narrative.input_buffer.status.clone(),
            game_state.narrative.input_buffer.phase.clone(),
        ))
    }

    /// Resets generation status to idle and persists the snapshot.
    pub fn reset_generating_status(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        let mut game_state = self.load_state(&ctx)?;
        game_state.narrative.input_buffer.status = crate::model::state::GenerationStatus::Idle;
        let snapshot =
            crate::model::state_snapshot::GameStateSnapshot::from_game_state(&game_state);
        ctx.storage.save_snapshot(&snapshot)?;
        Ok(())
    }

    /// Returns the name of the current game.
    pub fn get_current_game_name(
        &self,
        ctx: GameServiceContext,
    ) -> Result<String, ApplicationError> {
        match ctx.storage.get_game(ctx.storage.current_game_id())? {
            Some(g) => Ok(g.name),
            None => Ok("Unknown".to_string()),
        }
    }

    /// Returns the latest LLM messages up to the given limit.
    pub fn list_latest_llm_messages(
        &self,
        ctx: GameServiceContext,
        limit: usize,
    ) -> Result<Vec<LlmMessage>, ApplicationError> {
        ctx.storage
            .list_latest_llm_messages(limit)
            .map_err(Into::into)
    }

    /// Returns all story log entries and whether there's a pending trigger.
    pub fn get_story_log_entries(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(Vec<LogEntry>, bool), ApplicationError> {
        let game_state = self.load_state(&ctx)?;
        let entries: Vec<_> = game_state.narrative.history().to_vec();
        let has_last_trigger = game_state.narrative.last_trigger.is_some();
        Ok((entries, has_last_trigger))
    }

    /// Returns the current input/generation status (alias for get_generating_status).
    pub fn get_input_status(
        &self,
        ctx: GameServiceContext,
    ) -> Result<
        (
            crate::model::state::GenerationStatus,
            crate::model::state::GenerationPhase,
        ),
        ApplicationError,
    > {
        self.get_generating_status(ctx)
    }

    /// Returns the current room name and optional image path.
    pub fn get_current_room_view(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(String, Option<String>), ApplicationError> {
        let game_state = self.load_state(&ctx)?;
        let room = game_state
            .current_room()
            .ok_or_else(|| EngineError::RoomNotFound("current room not found".to_string()))?;

        let image_path = room
            .image_path
            .clone()
            .or_else(|| game_state.world.default_room_image.clone());

        Ok((room.name.clone(), image_path))
    }

    /// Returns NPC headshots: (image_path, name) tuples.
    /// If scene_only is true, only NPCs in the current area are returned.
    pub fn get_npc_headshots(
        &self,
        ctx: GameServiceContext,
        scene_only: bool,
    ) -> Result<Vec<(String, String)>, ApplicationError> {
        let game_state = self.load_state(&ctx)?;

        let npc_ids: Vec<String> = if scene_only {
            game_state
                .scene
                .npcs_in_area
                .iter()
                .map(|npc| npc.id.clone())
                .collect()
        } else {
            game_state.npcs.keys().cloned().collect()
        };

        let npc_data: Vec<(String, String)> = npc_ids
            .iter()
            .filter_map(|id| {
                let npc = game_state.npcs.get(id)?;
                let image_path = npc.sheet.preferred_image()?.to_string();
                let name = npc.sheet.name.clone();
                Some((image_path, name))
            })
            .collect();

        Ok(npc_data)
    }

    /// Returns debug state for diagnostics.
    pub fn get_debug_state(
        &self,
        ctx: GameServiceContext,
    ) -> Result<DebugStateView, ApplicationError> {
        let game_state = self.load_state(&ctx)?;

        let history_tail: Vec<LogEntry> = game_state
            .narrative
            .history()
            .iter()
            .rev()
            .take(5)
            .rev()
            .cloned()
            .collect();

        let npcs_in_area: Vec<String> = game_state
            .scene
            .npcs_in_area
            .iter()
            .map(|npc| npc.id.clone())
            .collect();

        let dynamic_rooms: Vec<String> =
            game_state.movement.dynamic_rooms.keys().cloned().collect();

        let last_error = match &game_state.narrative.input_buffer.status {
            crate::model::state::GenerationStatus::Error(msg) => Some(msg.clone()),
            _ => None,
        };

        Ok(DebugStateView {
            current_room_id: game_state.movement.current_room_id.clone(),
            npcs_in_area,
            generation_status: game_state.narrative.input_buffer.status.clone(),
            generation_phase: game_state.narrative.input_buffer.phase.clone(),
            npc_encounter_log: game_state.npc_encounter_log.npcs.clone(),
            narration_history_tail: history_tail,
            narration_history_length: game_state.narrative.history().len(),
            dynamic_rooms,
            dynamic_room_count: game_state.movement.dynamic_rooms.len(),
            last_error,
            quantifier_confidence: game_state.scene.quantifier_confidence.clone(),
            backend_name: game_state.narrative.last_backend_name.clone(),
            model_name: game_state.narrative.last_model_name.clone(),
        })
    }
}
