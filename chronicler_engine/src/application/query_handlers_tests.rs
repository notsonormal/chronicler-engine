use std::sync::Arc;

use super::*;
use crate::application::GameServiceContext;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::GenerationStatus;
use crate::adapters::driven::storage::Storage;
use crate::test_support::fixtures::{TestWorld, TestMap, TestPlayer};

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
        &crate::domain::model::state_snapshot::GameStateSnapshot::from_game_state(&state),
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
            crate::domain::model::settings::AppSettings::default(),
        )),
        preset_storage: Arc::new(Storage::new_in_memory()),
    }
}

#[test]
fn test_get_generating_status_returns_current_state() {
    let ctx = minimal_ctx();
    let (status, _phase) = get_generating_status(ctx).unwrap();
    assert_eq!(status, GenerationStatus::Idle);
}

#[test]
fn test_get_current_game_name_unknown_when_no_game() {
    let ctx = minimal_ctx();
    let name = get_current_game_name(ctx).unwrap();
    assert_eq!(name, "Unknown"); // No default game anymore
}

#[test]
fn test_list_latest_llm_messages_empty() {
    let ctx = minimal_ctx();
    let messages = list_latest_llm_messages(ctx, 10).unwrap();
    assert!(messages.is_empty());
}

#[test]
fn test_get_story_log_entries_empty() {
    let ctx = minimal_ctx();
    let (entries, has_trigger) = get_story_log_entries(ctx).unwrap();
    assert!(entries.is_empty());
    assert!(!has_trigger);
}

#[test]
fn test_get_input_status_delegates_to_generating_status() {
    let ctx = minimal_ctx();
    let (status1, phase1) = get_generating_status(ctx.clone()).unwrap();
    let (status2, phase2) = get_input_status(ctx).unwrap();
    assert_eq!(status1, status2);
    assert_eq!(phase1, phase2);
}

#[test]
fn test_get_current_room_view_succeeds_with_valid_state() {
    let ctx = minimal_ctx();
    let result = get_current_room_view(ctx);
    assert!(result.is_ok());
    let (room_name, _image_path) = result.unwrap();
    assert_eq!(room_name, "Room start");
}

#[test]
fn test_get_npc_headshots_scene_only_empty() {
    let ctx = minimal_ctx();
    let headshots = get_npc_headshots(ctx, true).unwrap();
    assert!(headshots.is_empty());
}

#[test]
fn test_get_npc_headshots_all_empty() {
    let ctx = minimal_ctx();
    let headshots = get_npc_headshots(ctx, false).unwrap();
    assert!(headshots.is_empty());
}

#[test]
fn test_get_debug_state_populates_fields() {
    let ctx = minimal_ctx();
    let debug = get_debug_state(ctx).unwrap();
    assert_eq!(debug.narration_history_length, 0);
    assert!(debug.dynamic_rooms.is_empty());
    assert_eq!(debug.dynamic_room_count, 0);
    assert!(debug.last_error.is_none());
}

#[test]
fn test_reset_generating_status_sets_idle() {
    let ctx = minimal_ctx();
    let result = reset_generating_status(ctx.clone());
    assert!(result.is_ok());
    let (status, _) = get_generating_status(ctx).unwrap();
    assert_eq!(status, GenerationStatus::Idle);
}
