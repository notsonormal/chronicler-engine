use std::sync::Arc;

use super::*;
use crate::application::application_service::DefaultApplicationService;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::GenerationStatus;
use crate::test_support::fixtures::{TestWorld, TestMap};
use crate::test_support::make_test_app;

fn minimal_state() -> GameState {
    GameState::new("start")
}

fn minimal_app() -> Arc<DefaultApplicationService> {
    make_test_app(minimal_state()).expect("minimal_app: make_test_app should succeed")
}

fn minimal_app_no_game() -> Arc<DefaultApplicationService> {
    use crate::adapters::driven::storage::Storage;
    use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
    use crate::test_support::make_test_recorder;
    use crate::application::game_service::GameService;
    use crate::application::agents::registry::AgentRegistry;
    use crate::adapters::driven::llm::providers::MockBackend;

    let state = minimal_state();
    let world_arc = Arc::new(TestWorld::minimal());
    let map_arc = Arc::new(TestMap::single_room("start"));
    let storage = Arc::new(Storage::new_in_memory());
    storage.seed_world(&world_arc, &map_arc).unwrap();
    let _ = storage.save_snapshot(&GameStateSnapshot::from_game_state(&state));
    let mock: Arc<dyn crate::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default());
    let backend = GameService::with_backends(make_test_recorder(mock), AgentRegistry::default());
    Arc::new(DefaultApplicationService::new(
        storage,
        Arc::new(Storage::new_in_memory()),
        Arc::new(std::sync::RwLock::new(
            crate::domain::model::settings::AppSettings::default(),
        )),
        tokio_util::sync::CancellationToken::new(),
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        Arc::new(backend),
    ))
}

#[test]
fn test_get_generating_status_returns_current_state() {
    let app = minimal_app();
    let (status, _phase) = get_generating_status(&app).unwrap();
    assert_eq!(status, GenerationStatus::Idle);
}

#[test]
fn test_get_current_game_name_unknown_when_no_game() {
    let app = minimal_app_no_game();
    let name = get_current_game_name(&app).unwrap();
    assert_eq!(name, "Unknown"); // No default game anymore
}

#[test]
fn test_list_latest_llm_messages_empty() {
    let app = minimal_app_no_game();
    let messages = list_latest_llm_messages(&app, 10).unwrap();
    assert!(messages.is_empty());
}

#[test]
fn test_get_story_log_entries_empty() {
    let app = minimal_app_no_game();
    let (entries, has_trigger) = get_story_log_entries(&app).unwrap();
    assert!(entries.is_empty());
    assert!(!has_trigger);
}

#[test]
fn test_get_input_status_delegates_to_generating_status() {
    let app = minimal_app();
    let (status1, phase1) = get_generating_status(&app).unwrap();
    let (status2, phase2) = get_input_status(&app).unwrap();
    assert_eq!(status1, status2);
    assert_eq!(phase1, phase2);
}

#[test]
fn test_get_current_room_view_succeeds_with_valid_state() {
    let app = minimal_app();
    let result = get_current_room_view(&app);
    assert!(result.is_ok());
    let (room_name, _image_path) = result.unwrap();
    assert_eq!(room_name, "Room start");
}

#[test]
fn test_get_current_room_view_returns_typed_error_when_game_missing() {
    let app = minimal_app_no_game();
    let err = get_current_room_view(&app).unwrap_err();
    assert!(
        matches!(
            &err,
            crate::application::errors::ApplicationError::Engine(
                crate::error::EngineError::GameNotFound(id)
            ) if *id == 0
        ),
        "expected GameNotFound(0), got: {err:?}"
    );
}

#[test]
fn test_get_npc_headshots_returns_typed_error_when_game_missing() {
    let app = minimal_app_no_game();
    let err = get_npc_headshots(&app, true).unwrap_err();
    assert!(
        matches!(
            &err,
            crate::application::errors::ApplicationError::Engine(
                crate::error::EngineError::GameNotFound(id)
            ) if *id == 0
        ),
        "expected GameNotFound(0), got: {err:?}"
    );
}

#[test]
fn test_get_npc_headshots_scene_only_empty() {
    let app = minimal_app();
    let headshots = get_npc_headshots(&app, true).unwrap();
    assert!(headshots.is_empty());
}

#[test]
fn test_get_npc_headshots_all_empty() {
    let app = minimal_app();
    let headshots = get_npc_headshots(&app, false).unwrap();
    assert!(headshots.is_empty());
}

#[test]
fn test_get_debug_state_populates_fields() {
    let app = minimal_app_no_game();
    let debug = get_debug_state(&app).unwrap();
    assert_eq!(debug.narration_history_length, 0);
    assert!(debug.dynamic_rooms.is_empty());
    assert_eq!(debug.dynamic_room_count, 0);
    assert!(debug.last_error.is_none());
}

#[test]
fn test_reset_generating_status_sets_idle() {
    let app = minimal_app_no_game();
    let result = reset_generating_status(&app);
    assert!(result.is_ok());
    let (status, _) = get_generating_status(&app).unwrap();
    assert_eq!(status, GenerationStatus::Idle);
}
