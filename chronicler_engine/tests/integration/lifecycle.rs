use std::sync::Arc;

use chronicler_engine::application::game_service::GameService;
use chronicler_engine::application::DefaultApplicationService;
use chronicler_engine::model::state::game_state::GameState;
use chronicler_engine::model::state::message_types::MessageType;
use chronicler_engine::narrative::agents::registry::AgentRegistry;
use chronicler_engine::narrative::llm::MockBackend;
use chronicler_engine::storage::Storage;

use crate::fixtures::{
    create_basic_test_state, create_test_map, create_test_player, create_test_world_with_scenario,
    make_test_ctx, seed_test_world,
};

fn create_app_service() -> Arc<DefaultApplicationService> {
    Arc::new(DefaultApplicationService::new(Arc::new(
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default()),
    )))
}

#[test]
fn test_create_game_with_scenario() {
    let db_pool =
        chronicler_engine::storage::db::DbPool::new(":memory:").expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    seed_test_world(&storage);
    let app_service = create_app_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), state);

    let result = app_service.create_game(ctx.clone());
    assert!(
        result.is_ok(),
        "create_game should succeed: {:?}",
        result.err()
    );
    let new_id = result.unwrap();

    let current_id = ctx.storage.current_game_id();
    assert_eq!(current_id, new_id, "Should have switched to new game");

    let latest = storage.load_latest_snapshot().unwrap();
    assert!(latest.is_some(), "New game should have an initial snapshot");

    let messages = storage.load_message_rows().unwrap();
    assert!(
        !messages.is_empty(),
        "New game should have at least one message (scenario introduction)"
    );
    let scenario_msg = &messages[0];
    assert_eq!(
        scenario_msg.message_type,
        MessageType::Narration,
        "First message should be Narration type"
    );

    let swipe_count = storage.count_swipes_for_message(scenario_msg.id).unwrap();
    assert!(
        swipe_count > 0,
        "Scenario message should have at least one swipe (text content)"
    );
}

#[test]
fn test_reset_creates_scenario_message() {
    let db_pool =
        chronicler_engine::storage::db::DbPool::new(":memory:").expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    seed_test_world(&storage);
    let app_service = create_app_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), state);

    app_service.create_game(ctx.clone()).unwrap();

    app_service.reset(ctx.clone()).unwrap();

    let messages_after = storage.load_message_rows().unwrap();
    assert!(
        !messages_after.is_empty(),
        "Reset should create a scenario message"
    );

    let scenario_msg = &messages_after[0];
    assert_eq!(
        scenario_msg.message_type,
        MessageType::Narration,
        "Scenario message should be Narration type"
    );

    let swipe_count = storage.count_swipes_for_message(scenario_msg.id).unwrap();
    assert!(
        swipe_count > 0,
        "Scenario message should have at least one swipe"
    );
}

#[test]
fn test_switch_game_loads_correct_state() {
    let db_pool =
        chronicler_engine::storage::db::DbPool::new(":memory:").expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    seed_test_world(&storage);
    let app_service = create_app_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), state);

    app_service.create_game(ctx.clone()).unwrap();
    let game1_id = ctx.storage.current_game_id();

    app_service.create_game(ctx.clone()).unwrap();
    let game2_id = ctx.storage.current_game_id();
    assert_ne!(game1_id, game2_id, "Should have different game IDs");

    let switch_result = app_service.switch_game(ctx.clone(), game1_id);
    assert!(
        switch_result.is_ok(),
        "switch_game should succeed: {:?}",
        switch_result.err()
    );
    assert_eq!(
        ctx.storage.current_game_id(),
        game1_id,
        "Should have switched to game 1"
    );
    let snapshot1 = storage.load_latest_snapshot().unwrap();
    assert!(
        snapshot1.is_some(),
        "Game 1 should have a snapshot after switching"
    );

    let switch_result = app_service.switch_game(ctx.clone(), game2_id);
    assert!(
        switch_result.is_ok(),
        "switch_game should succeed: {:?}",
        switch_result.err()
    );
    assert_eq!(
        ctx.storage.current_game_id(),
        game2_id,
        "Should have switched to game 2"
    );
    let snapshot2 = storage.load_latest_snapshot().unwrap();
    assert!(
        snapshot2.is_some(),
        "Game 2 should have a snapshot after switching"
    );
}

#[tokio::test]
async fn test_create_game_concurrent_generation_rejected() {
    use std::sync::atomic::Ordering;

    let db_pool =
        chronicler_engine::storage::db::DbPool::new(":memory:").expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    let app_service = create_app_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), state);

    ctx.is_generating.store(true, Ordering::SeqCst);

    let result = app_service.create_game(ctx.clone());
    assert!(
        result.is_err(),
        "create_game should fail during concurrent generation"
    );
    assert!(
        matches!(
            result.unwrap_err(),
            chronicler_engine::application::ApplicationError::ConcurrentGeneration
        ),
        "Error should be ConcurrentGeneration"
    );
}

#[test]
fn test_switch_to_nonexistent_game() {
    let db_pool =
        chronicler_engine::storage::db::DbPool::new(":memory:").expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    let app_service = create_app_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), state);

    let result = app_service.switch_game(ctx.clone(), 99999);
    assert!(
        result.is_err(),
        "switch_game should fail for nonexistent game"
    );
}

#[test]
fn test_reset_without_existing_game() {
    let db_pool =
        chronicler_engine::storage::db::DbPool::new(":memory:").expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    let app_service = create_app_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), state);

    let result = app_service.reset(ctx.clone());
    assert!(
        result.is_ok(),
        "reset should succeed with default game: {:?}",
        result.err()
    );
}

#[test]
fn test_create_game_name_uniqueness() {
    let db_pool =
        chronicler_engine::storage::db::DbPool::new(":memory:").expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    seed_test_world(&storage);
    let app_service = create_app_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), state);

    let result1 = app_service.create_game(ctx.clone());
    assert!(result1.is_ok(), "First create_game should succeed");

    let result2 = app_service.create_game(ctx.clone());
    assert!(result2.is_ok(), "Second create_game should succeed");

    let games = app_service.list_games(ctx.clone()).unwrap();
    let non_default_games: Vec<_> = games.iter().filter(|g| g.name != "default").collect();

    assert_eq!(
        non_default_games.len(),
        2,
        "Should have exactly 2 non-default games"
    );

    let world_date = chrono::Utc::now().format("%Y-%m-%d");
    let name_pattern = format!("Test World_{world_date}");

    let has_suffix_1 = non_default_games
        .iter()
        .any(|g| g.name == format!("{name_pattern}_1"));
    let has_suffix_2 = non_default_games
        .iter()
        .any(|g| g.name == format!("{name_pattern}_2"));
    assert!(has_suffix_1, "Should have game with _1 suffix");
    assert!(has_suffix_2, "Should have game with _2 suffix");
}

#[test]
fn test_switch_game_world_mismatch() {
    let db_pool =
        chronicler_engine::storage::db::DbPool::new(":memory:").expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    seed_test_world(&storage);
    let app_service = create_app_service();

    let mut world_a = create_test_world_with_scenario();
    world_a.name = "World A".to_string();
    let world_a = Arc::new(world_a);
    let player = Arc::new(create_test_player());
    let map = Arc::new(create_test_map());
    let npcs = Vec::new();
    let state_a = GameState::new(world_a.clone(), map, player, npcs, "room1".to_string());

    let ctx_a = make_test_ctx(storage.clone(), state_a);

    let create_result = app_service.create_game(ctx_a.clone());
    assert!(create_result.is_ok(), "create_game should succeed");
    let game_id = ctx_a.storage.current_game_id();

    let mut world_b = create_test_world_with_scenario();
    world_b.name = "World B".to_string();
    let world_b = Arc::new(world_b);
    let player_b = Arc::new(create_test_player());
    let map_b = Arc::new(create_test_map());
    let npcs_b = Vec::new();
    let state_b = GameState::new(
        world_b.clone(),
        map_b,
        player_b,
        npcs_b,
        "room1".to_string(),
    );

    let ctx_b = make_test_ctx(storage.clone(), state_b);

    let result = app_service.switch_game(ctx_b, game_id);
    assert!(
        result.is_ok(),
        "switch_game should succeed even for different worlds"
    );
}

#[test]
fn test_delete_game_removes() {
    let db_pool =
        chronicler_engine::storage::db::DbPool::new(":memory:").expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    seed_test_world(&storage);
    let app_service = create_app_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), state);

    app_service.create_game(ctx.clone()).unwrap();
    let game_id_1 = ctx.storage.current_game_id();

    app_service.create_game(ctx.clone()).unwrap();
    let game_id_2 = ctx.storage.current_game_id();

    assert_ne!(game_id_1, game_id_2, "Should have different game IDs");

    let delete_result = app_service.delete_game(ctx.clone(), game_id_1);
    assert!(delete_result.is_ok(), "delete_game should succeed");

    let games = app_service.list_games(ctx.clone()).unwrap();
    assert_eq!(games.len(), 1, "Should have exactly 1 game after deletion");
    assert!(
        games.iter().any(|g| g.id == game_id_2),
        "game_id_2 should still exist"
    );
    assert!(
        !games.iter().any(|g| g.id == game_id_1),
        "game_id_1 should be deleted"
    );
}

#[test]
fn test_delete_game_active_rejected() {
    let db_pool =
        chronicler_engine::storage::db::DbPool::new(":memory:").expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    seed_test_world(&storage);
    let app_service = create_app_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), state);

    app_service.create_game(ctx.clone()).unwrap();
    let active_game_id = ctx.storage.current_game_id();

    let result = app_service.delete_game(ctx.clone(), active_game_id);
    assert!(
        result.is_err(),
        "delete_game should fail for the active game"
    );
    assert!(
        matches!(
            result.unwrap_err(),
            chronicler_engine::application::ApplicationError::Validation(_)
        ),
        "Error should be Validation error"
    );
}

#[test]
fn test_delete_game_nonexistent() {
    let db_pool =
        chronicler_engine::storage::db::DbPool::new(":memory:").expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    let app_service = create_app_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), state);

    let result = app_service.delete_game(ctx.clone(), 99999);
    assert!(
        result.is_ok(),
        "delete_game should succeed silently for nonexistent game"
    );
}
