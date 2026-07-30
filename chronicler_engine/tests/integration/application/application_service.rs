//! Integration tests for DefaultApplicationService
use std::sync::Arc;

use chronicler_engine::application::application_service::ProcessActionResult;
use chronicler_engine::application::game_service::GameService;
use chronicler_engine::domain::model::state::generation_status::GenerationPhase;
use chronicler_engine::domain::model::state::generation_status::GenerationStatus;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::{TestAppBuilder, TestDataBuilder};

use crate::fixtures::{create_test_storage_arc as create_storage};
use crate::sqlite_test_app_builder::SqliteTestAppBuilder;
use crate::application_ext::PipelineHelpers;
use crate::storage_ext::TestWorldFixture;

fn create_game_service() -> Arc<GameService> {
    Arc::new(crate::working_service())
}

#[test]
fn test_create_game_integration() {
    let storage = create_storage(1);
    storage.seed_test_world_fixture();
    let game_service = create_game_service();
    let data = TestDataBuilder::default_test().build();
    let world_key = data.world_key();
    let app_service = TestAppBuilder::with_data(data)
        .storage(storage.clone())
        .game_service(game_service)
        .skip_seeding(true)
        .build_service();

    let game_id = app_service
        .create_game(&world_key, "hero")
        .expect("create_game should succeed");
    assert!(game_id > 0, "Game ID should be positive");

    let game = storage.get_game(game_id).unwrap();
    assert!(game.is_some(), "Game should be persisted");
}

#[test]
fn test_switch_game_integration() {
    let storage = create_storage(1);
    storage.seed_test_world_fixture();
    let game_service = create_game_service();
    let data = TestDataBuilder::default_test().build();
    let world_key = data.world_key();
    let app_service = TestAppBuilder::with_data(data)
        .storage(storage.clone())
        .game_service(game_service)
        .skip_seeding(true)
        .build_service();

    let id1 = app_service
        .create_game(&world_key, "hero")
        .expect("create_game 1");
    let id2 = app_service
        .create_game(&world_key, "hero")
        .expect("create_game 2");

    app_service.switch_game(id1).expect("switch_game");
    assert_eq!(storage.current_game_id(), id1);

    app_service.switch_game(id2).expect("switch_game");
    assert_eq!(storage.current_game_id(), id2);
}

#[test]
fn test_delete_game_integration() {
    let storage = create_storage(1);
    storage.seed_test_world_fixture();
    let game_service = create_game_service();
    let data = TestDataBuilder::default_test().build();
    let world_key = data.world_key();
    let app_service = TestAppBuilder::with_data(data)
        .storage(storage.clone())
        .game_service(game_service)
        .skip_seeding(true)
        .build_service();

    let id1 = app_service
        .create_game(&world_key, "hero")
        .expect("create_game 1");
    app_service
        .create_game(&world_key, "hero")
        .expect("create_game 2");

    app_service.delete_game(id1).expect("delete_game");

    let deleted = storage.get_game(id1).unwrap();
    assert!(deleted.is_none(), "Deleted game should not exist");
}

#[test]
fn test_list_games_integration() {
    let storage = create_storage(1);
    storage.seed_test_world_fixture();
    let game_service = create_game_service();
    let data = TestDataBuilder::default_test().build();
    let world_key = data.world_key();
    let app_service = TestAppBuilder::with_data(data)
        .storage(storage.clone())
        .game_service(game_service)
        .skip_seeding(true)
        .build_service();

    app_service.create_game(&world_key, "hero").unwrap();
    app_service.create_game(&world_key, "hero").unwrap();

    let games = app_service.list_games().unwrap();
    assert!(games.len() >= 2, "Should list all games");
}

#[test]
fn test_get_generating_status() {
    let storage = create_storage(1);
    let app_service = TestAppBuilder::default_test()
        .storage(storage)
        .game_service(create_game_service())
        .skip_seeding(true)
        .build_service();

    let (status, phase) = app_service.get_generating_status().unwrap();
    assert_eq!(status, GenerationStatus::Idle);
    assert_eq!(phase, GenerationPhase::default());
}

#[tokio::test]
async fn test_process_action_persists_input_message() {
    let game_service = create_game_service();
    let data = TestDataBuilder::default_test().build();
    let (app_service, pg_app_service) = SqliteTestAppBuilder::with_data(data)
        .game_service_fn(move |_storage| Arc::clone(&game_service))
        .build_with_state()
        .unwrap();

    let result = app_service.process_action("examine the room".to_string());
    assert!(
        matches!(result, Ok(ProcessActionResult::Started)),
        "process_action should return Started"
    );

    let completed = app_service.wait_for_generation_complete(&pg_app_service, 5000);
    assert!(completed, "Timed out waiting for generation to complete");

    let guard = app_service.latest_state(&pg_app_service);
    let entries = guard.narrative.history();
    let input_idx = entries
        .iter()
        .position(|e| e.message_type == MessageType::Input && e.text == "examine the room");
    let narration_idx = entries
        .iter()
        .position(|e| e.message_type == MessageType::Narration);
    assert!(input_idx.is_some(), "Input message should be persisted");
    assert!(narration_idx.is_some(), "Narration should be produced");
    assert!(
        input_idx.unwrap() < narration_idx.unwrap(),
        "Input should appear before Narration in history"
    );
}

#[tokio::test]
async fn test_process_action_self_heals_stale_generating_status() {
    let game_service = create_game_service();
    let data = TestDataBuilder::default_test().build();
    let (app_service, pg_app_service) = SqliteTestAppBuilder::with_data(data)
        .generation_status(GenerationStatus::Generating, GenerationPhase::Narrating)
        .game_service_fn(move |_storage| Arc::clone(&game_service))
        .build_with_state()
        .unwrap();

    let (status, _phase) = app_service.get_generating_status().unwrap();
    assert!(!status.is_generating());

    let result = app_service.process_action("look around".to_string());
    assert!(
        matches!(result, Ok(ProcessActionResult::Started)),
        "process_action should return Started"
    );

    let completed = app_service.wait_for_generation_complete(&pg_app_service, 5000);
    assert!(completed, "Timed out waiting for generation to complete");

    let guard = app_service.latest_state(&pg_app_service);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should not be Generating after completion, got {:?}",
        guard.narrative.input_buffer.status
    );
}
