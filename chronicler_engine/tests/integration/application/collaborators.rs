//! Integration tests for application collaborators

use chronicler_engine::application::errors::ProcessActionResult;
use chronicler_engine::application::pipeline::pipeline::ActionPipeline;
use chronicler_engine::domain::model::state::generation_status::GenerationPhase;
use chronicler_engine::domain::model::state::generation_status::GenerationStatus;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::{TestAppBuilder, TestDataBuilder};

use crate::fixtures::{create_test_storage_arc as create_storage};
use crate::sqlite_test_app_builder::SqliteTestAppBuilder;
use crate::application_ext::PipelineHelpers;
use crate::storage_ext::TestWorldFixture;

// TODO: I'm not sure what these integration tests are really
//  about. We had a `collaborators` method in the bootstrap folder
//  before but that was removed. So I don't know if we should really
//  have a tested class around a similar concept.

// TODO: If we need these tests, then we should move them into
//  more targeted tests

fn create_pipeline() -> ActionPipeline {
    crate::working_pipeline()
}

#[test]
fn test_create_game_integration() {
    let storage = create_storage(1);
    storage.seed_test_world_fixture();
    let pipeline = create_pipeline();
    let data = TestDataBuilder::default_test().build();
    let world_key = data.world_key();
    let app = TestAppBuilder::with_data(data)
        .storage(storage.clone())
        .pipeline(pipeline)
        .skip_seeding(true)
        .build_service();

    let game_id = app
        .game_catalogue
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
    let pipeline = create_pipeline();
    let data = TestDataBuilder::default_test().build();
    let world_key = data.world_key();
    let app = TestAppBuilder::with_data(data)
        .storage(storage.clone())
        .pipeline(pipeline)
        .skip_seeding(true)
        .build_service();

    let id1 = app
        .game_catalogue
        .create_game(&world_key, "hero")
        .expect("create_game 1");
    let id2 = app
        .game_catalogue
        .create_game(&world_key, "hero")
        .expect("create_game 2");

    app.game_catalogue.switch_game(id1).expect("switch_game");
    assert_eq!(storage.current_game_id(), id1);

    app.game_catalogue.switch_game(id2).expect("switch_game");
    assert_eq!(storage.current_game_id(), id2);
}

#[test]
fn test_delete_game_integration() {
    let storage = create_storage(1);
    storage.seed_test_world_fixture();
    let pipeline = create_pipeline();
    let data = TestDataBuilder::default_test().build();
    let world_key = data.world_key();
    let app = TestAppBuilder::with_data(data)
        .storage(storage.clone())
        .pipeline(pipeline)
        .skip_seeding(true)
        .build_service();

    let id1 = app
        .game_catalogue
        .create_game(&world_key, "hero")
        .expect("create_game 1");
    app.game_catalogue
        .create_game(&world_key, "hero")
        .expect("create_game 2");

    app.game_catalogue.delete_game(id1).expect("delete_game");

    let deleted = storage.get_game(id1).unwrap();
    assert!(deleted.is_none(), "Deleted game should not exist");
}

#[test]
fn test_list_games_integration() {
    let storage = create_storage(1);
    storage.seed_test_world_fixture();
    let pipeline = create_pipeline();
    let data = TestDataBuilder::default_test().build();
    let world_key = data.world_key();
    let app = TestAppBuilder::with_data(data)
        .storage(storage.clone())
        .pipeline(pipeline)
        .skip_seeding(true)
        .build_service();

    app.game_catalogue.create_game(&world_key, "hero").unwrap();
    app.game_catalogue.create_game(&world_key, "hero").unwrap();

    let games = app.game_catalogue.list_games().unwrap();
    assert!(games.len() >= 2, "Should list all games");
}

#[test]
fn test_get_generating_status() {
    let storage = create_storage(1);
    let app = TestAppBuilder::default_test()
        .storage(storage)
        .pipeline(create_pipeline())
        .skip_seeding(true)
        .build_service();

    let (status, phase) = app.game_view_query.get_generating_status().unwrap();
    assert_eq!(status, GenerationStatus::Idle);
    assert_eq!(phase, GenerationPhase::default());
}

#[tokio::test]
async fn test_process_action_persists_input_message() {
    let pipeline = create_pipeline();
    let data = TestDataBuilder::default_test().build();
    let app = SqliteTestAppBuilder::with_data(data)
        .pipeline_fn(move |_storage, _pg, _settings, _token| pipeline.clone())
        .build_with_state()
        .unwrap();

    let result = app
        .pipeline
        .process_action(&app.generation_gate, "examine the room".to_string());
    assert!(
        matches!(result, Ok(ProcessActionResult::Started)),
        "process_action should return Started"
    );

    let completed = app.wait_for_generation_complete(5000);
    assert!(completed, "Timed out waiting for generation to complete");

    let guard = app.latest_state();
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
    let pipeline = create_pipeline();
    let data = TestDataBuilder::default_test().build();
    let app = SqliteTestAppBuilder::with_data(data)
        .generation_status(GenerationStatus::Generating, GenerationPhase::Narrating)
        .pipeline_fn(move |_storage, _pg, _settings, _token| pipeline.clone())
        .build_with_state()
        .unwrap();

    let (status, _phase) = app.game_view_query.get_generating_status().unwrap();
    assert!(!status.is_generating());

    let result = app
        .pipeline
        .process_action(&app.generation_gate, "look around".to_string());
    assert!(
        matches!(result, Ok(ProcessActionResult::Started)),
        "process_action should return Started"
    );

    let completed = app.wait_for_generation_complete(5000);
    assert!(completed, "Timed out waiting for generation to complete");

    let guard = app.latest_state();
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should not be Generating after completion, got {:?}",
        guard.narrative.input_buffer.status
    );
}
