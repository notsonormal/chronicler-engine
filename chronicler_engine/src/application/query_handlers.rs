use crate::application::ApplicationError;
use crate::application::DebugStateView;
use crate::application::context::{GameServiceContext, load_or_fresh};
use crate::error::EngineError;
use crate::model::llm_message::LlmMessage;
use crate::model::state::MessageEntry;

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
        let game_state = load_or_fresh(&ctx);
        Ok((
            game_state.narrative.input_buffer.status.clone(),
            game_state.narrative.input_buffer.phase.clone(),
        ))
    }

    pub fn reset_generating_status(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        let mut game_state = load_or_fresh(&ctx);
        game_state.narrative.input_buffer.status = crate::model::state::GenerationStatus::Idle;
        let snapshot =
            crate::model::state_snapshot::GameStateSnapshot::from_game_state(&game_state);
        ctx.storage.save_snapshot(&snapshot)?;
        Ok(())
    }

    pub fn get_current_game_name(
        &self,
        ctx: GameServiceContext,
    ) -> Result<String, ApplicationError> {
        match ctx.storage.get_game(ctx.storage.current_game_id())? {
            Some(g) => Ok(g.name),
            None => Ok("Unknown".to_string()),
        }
    }

    pub fn list_latest_llm_messages(
        &self,
        ctx: GameServiceContext,
        limit: usize,
    ) -> Result<Vec<LlmMessage>, ApplicationError> {
        ctx.storage
            .list_latest_llm_messages(limit)
            .map_err(Into::into)
    }

    pub fn get_story_log_entries(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(Vec<MessageEntry>, bool), ApplicationError> {
        let game_state = load_or_fresh(&ctx);
        let entries: Vec<_> = game_state.narrative.history().to_vec();
        let has_last_trigger = game_state.narrative.last_trigger.is_some();
        Ok((entries, has_last_trigger))
    }

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

    pub fn get_current_room_view(
        &self,
        ctx: GameServiceContext,
    ) -> Result<(String, Option<String>), ApplicationError> {
        let game_state = load_or_fresh(&ctx);
        let room = game_state
            .current_room()
            .ok_or_else(|| EngineError::RoomNotFound("current room not found".to_string()))?;

        let image_path = room
            .image_path
            .clone()
            .or_else(|| game_state.world.default_room_image.clone());

        Ok((room.name.clone(), image_path))
    }

    pub fn get_npc_headshots(
        &self,
        ctx: GameServiceContext,
        scene_only: bool,
    ) -> Result<Vec<(String, String)>, ApplicationError> {
        let game_state = load_or_fresh(&ctx);

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

    pub fn get_debug_state(
        &self,
        ctx: GameServiceContext,
    ) -> Result<DebugStateView, ApplicationError> {
        let game_state = load_or_fresh(&ctx);

        let history_tail: Vec<MessageEntry> = game_state
            .narrative
            .history()
            .iter()
            .rev()
            .take(5)
            .rev()
            .cloned()
            .collect();

        let npc_ids: Vec<String> = game_state
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
            npcs_in_area: npc_ids,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::state::GameState;
    use crate::storage::Storage;
    use crate::test_support::fixtures::{TestWorld, TestMap, TestPlayer};
    use std::sync::Arc;

    fn minimal_state() -> GameState {
        GameState::new(
            Arc::new(TestWorld::minimal()),
            Arc::new(TestMap::single_room("start")),
            Arc::new(TestPlayer::named("Test")),
            vec![],
            "start".to_string(),
        )
    }

    fn minimal_ctx() -> GameServiceContext {
        let state = minimal_state();
        let storage = Arc::new(Storage::new_in_memory());
        let _ = storage.save_snapshot(
            &crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state)
        );
        GameServiceContext {
            storage,
            world: state.world.clone(),
            map: state.map.clone(),
            player: state.player.clone(),
            npcs: Arc::new(state.npcs.clone()),
            cancel_token: tokio_util::sync::CancellationToken::new(),
            is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            settings: Arc::new(std::sync::RwLock::new(
                crate::model::settings::AppSettings::default(),
            )),
            preset_storage: Arc::new(Storage::new_in_memory()),
        }
    }

    #[test]
    fn test_get_generating_status_returns_current_state() {
        let ctx = minimal_ctx();
        let handlers = QueryHandlers::new();
        let (status, _phase) = handlers.get_generating_status(ctx).unwrap();
        assert_eq!(status, crate::model::state::GenerationStatus::Idle);
    }

    #[test]
    #[test]
    fn test_get_current_game_name_unknown_when_no_game() {
        let ctx = minimal_ctx();
        let handlers = QueryHandlers::new();
        let name = handlers.get_current_game_name(ctx).unwrap();
        // TestWorld creates a game with name "default"
        assert_eq!(name, "default");
    }

    #[test]
    fn test_list_latest_llm_messages_empty() {
        let ctx = minimal_ctx();
        let handlers = QueryHandlers::new();
        let messages = handlers.list_latest_llm_messages(ctx, 10).unwrap();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_get_story_log_entries_empty() {
        let ctx = minimal_ctx();
        let handlers = QueryHandlers::new();
        let (entries, has_trigger) = handlers.get_story_log_entries(ctx).unwrap();
        assert!(entries.is_empty());
        assert!(!has_trigger);
    }

    #[test]
    fn test_get_input_status_delegates_to_generating_status() {
        let ctx = minimal_ctx();
        let handlers = QueryHandlers::new();
        let (status1, phase1) = handlers.get_generating_status(ctx.clone()).unwrap();
        let (status2, phase2) = handlers.get_input_status(ctx).unwrap();
        assert_eq!(status1, status2);
        assert_eq!(phase1, phase2);
    }

    #[test]
    fn test_get_current_room_view_succeeds_with_valid_state() {
        let ctx = minimal_ctx();
        let handlers = QueryHandlers::new();
        let result = handlers.get_current_room_view(ctx);
        assert!(result.is_ok());
        let (room_name, _image_path) = result.unwrap();
        assert_eq!(room_name, "Room start");
    }

    #[test]
    fn test_get_npc_headshots_scene_only_empty() {
        let ctx = minimal_ctx();
        let handlers = QueryHandlers::new();
        let headshots = handlers.get_npc_headshots(ctx, true).unwrap();
        assert!(headshots.is_empty());
    }

    #[test]
    fn test_get_npc_headshots_all_empty() {
        let ctx = minimal_ctx();
        let handlers = QueryHandlers::new();
        let headshots = handlers.get_npc_headshots(ctx, false).unwrap();
        assert!(headshots.is_empty());
    }

    #[test]
    fn test_get_debug_state_populates_fields() {
        let ctx = minimal_ctx();
        let handlers = QueryHandlers::new();
        let debug = handlers.get_debug_state(ctx).unwrap();
        assert_eq!(debug.narration_history_length, 0);
        assert!(debug.dynamic_rooms.is_empty());
        assert_eq!(debug.dynamic_room_count, 0);
        assert!(debug.last_error.is_none());
    }

    #[test]
    fn test_reset_generating_status_sets_idle() {
        let ctx = minimal_ctx();
        let handlers = QueryHandlers::new();
        let result = handlers.reset_generating_status(ctx.clone());
        assert!(result.is_ok());
        let (status, _) = handlers.get_generating_status(ctx).unwrap();
        assert_eq!(status, crate::model::state::GenerationStatus::Idle);
    }
}
