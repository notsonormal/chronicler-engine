use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::adapters::driving::http::AppState;
use crate::application::errors::ApplicationError;
use crate::application::games::view_query::GameViewQuery;
use crate::domain::model::agent::Role;
use crate::domain::model::game::GameId;
use crate::error::EngineError;
use crate::test_support::fixtures::{active_game_of, llm_message, TestMap, TestPersona, TestWorld};
use crate::test_support::{TestAppBuilder, TestData};

const NO_GAME: GameId = GameId(0);

/// The app's Game holds its opening Narration: `TestAppBuilder` seeds through the
/// design's create path.
fn minimal_data() -> TestData {
    TestData {
        world: Arc::new(TestWorld::minimal()),
        map: Arc::new(TestMap::single_room("start")),
        persona: Arc::new(TestPersona::standard()),
        npcs: Vec::new(),
        room_npcs: Vec::new(),
    }
}

fn minimal_app() -> AppState {
    TestAppBuilder::with_data(minimal_data()).build_service()
}

fn minimal_app_no_game() -> AppState {
    TestAppBuilder::with_data(minimal_data())
        .skip_seeding(true)
        .active_game(NO_GAME)
        .build_service()
}

#[test]
fn test_game_display_name_unknown_when_no_game() {
    let app = minimal_app_no_game();
    let name = app
        .game_view_query
        .game_display_name(active_game_of(&app))
        .unwrap();
    assert_eq!(name, "Unknown");
}

#[test]
fn test_list_latest_llm_messages_empty() {
    let app = minimal_app_no_game();
    let messages = app.game_view_query.list_latest_llm_messages(10).unwrap();
    assert!(messages.is_empty());
}

#[test]
fn test_story_window_empty() {
    let app = minimal_app_no_game();
    let window = app
        .game_view_query
        .story_window(active_game_of(&app))
        .unwrap();
    assert!(window.entries.is_empty());
    assert!(!window.retrigger_allowed);
}

#[test]
fn test_get_current_room_view_succeeds_with_valid_state() {
    let app = minimal_app();
    let result = app
        .game_view_query
        .get_current_room_view(active_game_of(&app));
    assert!(result.is_ok());
    let (room_name, _image_path) = result.unwrap();
    assert_eq!(room_name, "Room start");
}

#[test]
fn test_get_current_room_view_returns_typed_error_when_game_missing() {
    let app = minimal_app_no_game();
    let err = app
        .game_view_query
        .get_current_room_view(active_game_of(&app))
        .unwrap_err();
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
    let err = app
        .game_view_query
        .get_npc_headshots(active_game_of(&app), true)
        .unwrap_err();
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
    let headshots = app
        .game_view_query
        .get_npc_headshots(active_game_of(&app), true)
        .unwrap();
    assert!(headshots.is_empty());
}

#[test]
fn test_get_npc_headshots_all_empty() {
    let app = minimal_app();
    let headshots = app
        .game_view_query
        .get_npc_headshots(active_game_of(&app), false)
        .unwrap();
    assert!(headshots.is_empty());
}

#[test]
fn test_get_debug_state_populates_fields() {
    let app = minimal_app();
    let debug = app
        .game_view_query
        .get_debug_state(active_game_of(&app))
        .unwrap();
    assert_eq!(debug.narration_history_length, 1);
    assert!(debug.dynamic_rooms.is_empty());
    assert_eq!(debug.dynamic_room_count, 0);
    assert!(debug.last_error.is_none());
}

#[test]
fn test_role_health_reads_each_roles_true_newest_attempt() {
    let storage = Arc::new(Storage::new_in_memory());
    storage
        .save_llm_message(&llm_message("quantifier", "mock", Some("q failure"), -20))
        .unwrap();
    storage
        .save_llm_message(&llm_message("quantifier", "mock", None, -5))
        .unwrap();
    storage
        .save_llm_message(&llm_message("narrator", "mock", Some("n failure"), -15))
        .unwrap();

    let query = GameViewQuery::new(
        Arc::clone(&storage),
        crate::application::generation::gate::GameWriter::new(Arc::clone(&storage)),
    );
    let health = query.role_health().unwrap();

    let narrator = health.iter().find(|r| r.role == Role::Narrator).unwrap();
    assert_eq!(narrator.last_error.as_deref(), Some("n failure"));
    let quantifier = health.iter().find(|r| r.role == Role::Quantifier).unwrap();
    assert!(
        quantifier.last_error.is_none(),
        "the newer quantifier success must clear the older failure"
    );
    let options = health.iter().find(|r| r.role == Role::Options).unwrap();
    assert!(options.last_error.is_none());
    assert!(
        options.backend_model.is_none(),
        "a role with no recorded attempt reports no backend/model"
    );
}
