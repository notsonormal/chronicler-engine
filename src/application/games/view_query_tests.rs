//! Unit tests for GameViewQuery read-side orchestration.

use std::sync::Arc;

use crate::adapters::driven::llm::providers::MockBackend;
use crate::adapters::driven::storage::Storage;
use crate::adapters::driving::http::AppState;
use crate::application::errors::ApplicationError;
use crate::application::ports::llm_provider::LlmProvider;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::state::generation_status::GenerationStatus;
use crate::error::EngineError;
use crate::test_support::fixtures::{TestMap, TestWorld};
use crate::test_support::{make_test_pipeline_with_backends, make_test_recorder};

fn minimal_state() -> GameState {
    GameState::new("start")
}

fn minimal_app() -> AppState {
    AppState::from_wired(
        crate::test_support::make_test_app(minimal_state()).expect("minimal_app should build"),
    )
}

fn minimal_app_no_game() -> AppState {
    let state = minimal_state();
    let world_arc = Arc::new(TestWorld::minimal());
    let map_arc = Arc::new(TestMap::single_room("start"));
    let storage = Arc::new(Storage::new_in_memory());
    storage.seed_world(&world_arc, &map_arc).unwrap();
    let _ = storage.save_snapshot(&GameStateSnapshot::from_game_state(&state));
    let mock: Arc<dyn LlmProvider> = Arc::new(MockBackend::default());
    let narrator_recorder = make_test_recorder(Arc::clone(&mock));
    let registry = crate::application::agents::registry::AgentRegistry::default();
    let pipeline =
        make_test_pipeline_with_backends(Arc::clone(&storage), narrator_recorder, registry);
    let wired = crate::test_support::build_test_wired_app(Arc::clone(&storage), pipeline)
        .expect("build_test_wired_app: build_app_graph_for_tests should succeed");
    AppState::from_wired(wired)
}

#[test]
fn test_get_generating_status_returns_current_state() {
    let app = minimal_app();
    let (status, _phase) = app.game_view_query.get_generating_status().unwrap();
    assert_eq!(status, GenerationStatus::Idle);
}

#[test]
fn test_get_current_game_name_unknown_when_no_game() {
    let app = minimal_app_no_game();
    let name = app.game_view_query.get_current_game_name().unwrap();
    assert_eq!(name, "Unknown");
}

#[test]
fn test_list_latest_llm_messages_empty() {
    let app = minimal_app_no_game();
    let messages = app.game_view_query.list_latest_llm_messages(10).unwrap();
    assert!(messages.is_empty());
}

#[test]
fn test_get_story_log_entries_empty() {
    let app = minimal_app_no_game();
    let (entries, has_trigger) = app.game_view_query.get_story_log_entries().unwrap();
    assert!(entries.is_empty());
    assert!(!has_trigger);
}

#[test]
fn test_get_current_room_view_succeeds_with_valid_state() {
    let app = minimal_app();
    let result = app.game_view_query.get_current_room_view();
    assert!(result.is_ok());
    let (room_name, _image_path) = result.unwrap();
    assert_eq!(room_name, "Room start");
}

#[test]
fn test_get_current_room_view_returns_typed_error_when_game_missing() {
    let app = minimal_app_no_game();
    let err = app.game_view_query.get_current_room_view().unwrap_err();
    assert!(
        matches!(
            &err,
            ApplicationError::Engine(EngineError::GameNotFound(id)) if *id == 0
        ),
        "expected GameNotFound(0), got: {err:?}"
    );
}

#[test]
fn test_get_npc_headshots_returns_typed_error_when_game_missing() {
    let app = minimal_app_no_game();
    let err = app.game_view_query.get_npc_headshots(true).unwrap_err();
    assert!(
        matches!(
            &err,
            ApplicationError::Engine(EngineError::GameNotFound(id)) if *id == 0
        ),
        "expected GameNotFound(0), got: {err:?}"
    );
}

#[test]
fn test_get_npc_headshots_scene_only_empty() {
    let app = minimal_app();
    let headshots = app.game_view_query.get_npc_headshots(true).unwrap();
    assert!(headshots.is_empty());
}

#[test]
fn test_get_npc_headshots_all_empty() {
    let app = minimal_app();
    let headshots = app.game_view_query.get_npc_headshots(false).unwrap();
    assert!(headshots.is_empty());
}

#[test]
fn test_get_debug_state_populates_fields() {
    let app = minimal_app_no_game();
    let debug = app.game_view_query.get_debug_state().unwrap();
    assert_eq!(debug.narration_history_length, 0);
    assert!(debug.dynamic_rooms.is_empty());
    assert_eq!(debug.dynamic_room_count, 0);
    assert!(debug.last_error.is_none());
}

#[test]
fn test_active_quantifier_prompt_does_not_panic() {
    let app = minimal_app();
    let prompt = app.game_view_query.active_quantifier_prompt();
    let _ = prompt.len();
}
