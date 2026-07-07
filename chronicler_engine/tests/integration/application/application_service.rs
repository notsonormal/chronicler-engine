//! Integration tests for DefaultApplicationService
use std::sync::Arc;

use chronicler_engine::application::application_service::ProcessActionResult;
use chronicler_engine::application::game_service::GameService;
use chronicler_engine::domain::model::state::generation_status::GenerationPhase;
use chronicler_engine::domain::model::state::generation_status::GenerationStatus;
use chronicler_engine::domain::model::state::message_types::MessageType;

use crate::fixtures::{
    create_basic_test_state, create_test_storage_arc as create_storage,
    make_test_app_service_from_ctx as make_app, make_test_ctx, seed_test_world,
};

fn create_game_service() -> Arc<GameService> {
    Arc::new(crate::working_service())
}

#[test]
fn test_create_game_integration() {
    let storage = create_storage(1);
    seed_test_world(&storage);
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), state);

    let app_service = make_app(&ctx, game_service);

    let game_id = app_service
        .create_game(ctx.clone())
        .expect("create_game should succeed");
    assert!(game_id > 0, "Game ID should be positive");

    let game = storage.get_game(game_id).unwrap();
    assert!(game.is_some(), "Game should be persisted");
}

#[test]
fn test_switch_game_integration() {
    let storage = create_storage(1);
    seed_test_world(&storage);
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), state);

    let app_service = make_app(&ctx, game_service);

    let id1 = app_service.create_game(ctx.clone()).expect("create_game 1");
    let id2 = app_service.create_game(ctx.clone()).expect("create_game 2");

    app_service
        .switch_game(ctx.clone(), id1)
        .expect("switch_game");
    assert_eq!(storage.current_game_id(), id1);

    app_service
        .switch_game(ctx.clone(), id2)
        .expect("switch_game");
    assert_eq!(storage.current_game_id(), id2);
}

#[test]
fn test_delete_game_integration() {
    let storage = create_storage(1);
    seed_test_world(&storage);
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), state);

    let app_service = make_app(&ctx, game_service);

    let id1 = app_service.create_game(ctx.clone()).expect("create_game 1");
    app_service.create_game(ctx.clone()).expect("create_game 2");

    app_service
        .delete_game(ctx.clone(), id1)
        .expect("delete_game");

    let deleted = storage.get_game(id1).unwrap();
    assert!(deleted.is_none(), "Deleted game should not exist");
}

#[test]
fn test_list_games_integration() {
    let storage = create_storage(1);
    seed_test_world(&storage);
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), state);

    let app_service = make_app(&ctx, game_service);

    app_service.create_game(ctx.clone()).unwrap();
    app_service.create_game(ctx.clone()).unwrap();

    let games = app_service.list_games(ctx.clone()).unwrap();
    assert!(games.len() >= 2, "Should list all games");
}

#[test]
fn test_get_generating_status() {
    let storage = create_storage(1);
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), state);

    let (status, phase) =
        chronicler_engine::application::query_handlers::get_generating_status(ctx.clone()).unwrap();
    assert_eq!(status, GenerationStatus::Idle);
    assert_eq!(phase, GenerationPhase::default());
}

#[tokio::test]
async fn test_process_action_persists_input_message() {
    let game_service = create_game_service();
    let mut state = crate::fixtures::create_test_state();
    state.narrative.history.clear();
    let ctx = chronicler_engine::test_support::make_test_context_with_sqlite(state).unwrap();
    let app_service = make_app(&ctx, game_service);

    let result = app_service.process_action(ctx.clone(), "examine the room".to_string());
    assert!(
        matches!(result, Ok(ProcessActionResult::Started)),
        "process_action should return Started"
    );

    let completed = crate::pipeline_helpers::wait_for_generation_complete(&ctx, 5000);
    assert!(completed, "Timed out waiting for generation to complete");

    let guard = crate::pipeline_helpers::latest_state(&ctx);
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
    let mut state = crate::fixtures::create_test_state();
    state.narrative.history.clear();

    state.narrative.input_buffer.status = GenerationStatus::Generating;
    state.narrative.input_buffer.phase = GenerationPhase::Narrating;
    let ctx = chronicler_engine::test_support::make_test_context_with_sqlite(state).unwrap();
    let app_service = make_app(&ctx, game_service);

    assert!(!ctx.is_generating.load(std::sync::atomic::Ordering::SeqCst));

    let result = app_service.process_action(ctx.clone(), "look around".to_string());
    assert!(
        matches!(result, Ok(ProcessActionResult::Started)),
        "process_action should return Started"
    );

    let completed = crate::pipeline_helpers::wait_for_generation_complete(&ctx, 5000);
    assert!(completed, "Timed out waiting for generation to complete");

    let guard = crate::pipeline_helpers::latest_state(&ctx);
    assert!(
        !guard.narrative.input_buffer.status.is_generating(),
        "Status should not be Generating after completion, got {:?}",
        guard.narrative.input_buffer.status
    );
}
