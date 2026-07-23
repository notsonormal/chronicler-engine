//! Integration tests for game lifecycle operations — cross-cutting over `src/application/` rather than a mirror of a single src file; kept here for simplicity until the suite grows enough to split per-module.

use std::sync::Arc;

use chronicler_engine::application::game_service::GameService;
use chronicler_engine::application::DefaultApplicationService;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::application::agents::registry::AgentRegistry;
use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::{TestAppBuilder, TestDataBuilder};

use crate::fixtures::{create_test_world_with_scenario};
use crate::storage_ext::TestWorldFixture;

#[allow(dead_code)]
fn create_app_service() -> Arc<DefaultApplicationService> {
    create_app_service_with_storage(Arc::new(Storage::new_in_memory()))
}

fn create_app_service_with_storage(storage: Arc<Storage>) -> Arc<DefaultApplicationService> {
    let game_service = Arc::new(GameService::with_backends(
        crate::make_test_recorder(Arc::new(MockBackend::default())),
        AgentRegistry::default(),
    ));
    TestAppBuilder::default_test()
        .storage(storage)
        .game_service(game_service)
        .skip_seeding(true)
        .build_service()
}

#[test]
fn test_create_game_with_scenario() {
    let db_pool = chronicler_engine::adapters::driven::storage::db::DbPool::new(":memory:")
        .expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    storage.seed_test_world_with_scenario_fixture();
    let data = TestDataBuilder::default_test().build();
    let world_key = data.world_key();
    let app_service = TestAppBuilder::with_data(data)
        .storage(storage.clone())
        .game_service(Arc::new(GameService::with_backends(
            crate::make_test_recorder(Arc::new(MockBackend::default())),
            AgentRegistry::default(),
        )))
        .skip_seeding(true)
        .build_service();

    let result = app_service.create_game(&world_key, "hero");
    assert!(
        result.is_ok(),
        "create_game should succeed: {:?}",
        result.err()
    );
    let new_id = result.unwrap();

    let current_id = app_service.storage().current_game_id();
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
    let db_pool = chronicler_engine::adapters::driven::storage::db::DbPool::new(":memory:")
        .expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    storage.seed_test_world_with_scenario_fixture();
    let data = TestDataBuilder::default_test().build();
    let world_key = data.world_key();
    let app_service = TestAppBuilder::with_data(data)
        .storage(storage.clone())
        .game_service(Arc::new(GameService::with_backends(
            crate::make_test_recorder(Arc::new(MockBackend::default())),
            AgentRegistry::default(),
        )))
        .skip_seeding(true)
        .build_service();

    app_service.create_game(&world_key, "hero").unwrap();

    app_service.reset().unwrap();

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
    let db_pool = chronicler_engine::adapters::driven::storage::db::DbPool::new(":memory:")
        .expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    storage.seed_test_world_fixture();
    let data = TestDataBuilder::default_test().build();
    let world_key = data.world_key();
    let app_service = TestAppBuilder::with_data(data)
        .storage(storage.clone())
        .game_service(Arc::new(GameService::with_backends(
            crate::make_test_recorder(Arc::new(MockBackend::default())),
            AgentRegistry::default(),
        )))
        .skip_seeding(true)
        .build_service();

    app_service.create_game(&world_key, "hero").unwrap();
    let game1_id = app_service.storage().current_game_id();

    app_service.create_game(&world_key, "hero").unwrap();
    let game2_id = app_service.storage().current_game_id();
    assert_ne!(game1_id, game2_id, "Should have different game IDs");

    let switch_result = app_service.switch_game(game1_id);
    assert!(
        switch_result.is_ok(),
        "switch_game should succeed: {:?}",
        switch_result.err()
    );
    assert_eq!(
        app_service.storage().current_game_id(),
        game1_id,
        "Should have switched to game 1"
    );
    let snapshot1 = storage.load_latest_snapshot().unwrap();
    assert!(
        snapshot1.is_some(),
        "Game 1 should have a snapshot after switching"
    );

    let switch_result = app_service.switch_game(game2_id);
    assert!(
        switch_result.is_ok(),
        "switch_game should succeed: {:?}",
        switch_result.err()
    );
    assert_eq!(
        app_service.storage().current_game_id(),
        game2_id,
        "Should have switched to game 2"
    );
    let snapshot2 = storage.load_latest_snapshot().unwrap();
    assert!(
        snapshot2.is_some(),
        "Game 2 should have a snapshot after switching"
    );
}

#[test]
fn test_switch_to_nonexistent_game() {
    let db_pool = chronicler_engine::adapters::driven::storage::db::DbPool::new(":memory:")
        .expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    storage.seed_test_world_fixture();
    let app_service = TestAppBuilder::default_test()
        .storage(storage.clone())
        .game_service(Arc::new(GameService::with_backends(
            crate::make_test_recorder(Arc::new(MockBackend::default())),
            AgentRegistry::default(),
        )))
        .skip_seeding(true)
        .build_service();

    let result = app_service.switch_game(99999);
    assert!(
        result.is_err(),
        "switch_game should fail for nonexistent game"
    );
}

#[test]
fn test_reset_without_existing_game() {
    let db_pool = chronicler_engine::adapters::driven::storage::db::DbPool::new(":memory:")
        .expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    storage.seed_test_world_with_scenario_fixture();
    let data = TestDataBuilder::default_test().build();
    let world_key = data.world_key();
    let app_service = TestAppBuilder::with_data(data)
        .storage(storage.clone())
        .game_service(Arc::new(GameService::with_backends(
            crate::make_test_recorder(Arc::new(MockBackend::default())),
            AgentRegistry::default(),
        )))
        .skip_seeding(true)
        .build_service();

    let _ = app_service.create_game(&world_key, "hero").unwrap();

    let result = app_service.reset();
    assert!(
        result.is_ok(),
        "reset should succeed with default game: {:?}",
        result.err()
    );
}

#[test]
fn test_create_game_name_uniqueness() {
    let db_pool = chronicler_engine::adapters::driven::storage::db::DbPool::new(":memory:")
        .expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    storage.seed_test_world_fixture();
    let data = TestDataBuilder::default_test().build();
    let world_key = data.world_key();
    let app_service = TestAppBuilder::with_data(data)
        .storage(storage.clone())
        .game_service(Arc::new(GameService::with_backends(
            crate::make_test_recorder(Arc::new(MockBackend::default())),
            AgentRegistry::default(),
        )))
        .skip_seeding(true)
        .build_service();

    let result1 = app_service.create_game(&world_key, "hero");
    assert!(result1.is_ok(), "First create_game should succeed");

    let result2 = app_service.create_game(&world_key, "hero");
    assert!(result2.is_ok(), "Second create_game should succeed");

    let games = app_service.list_games().unwrap();
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
    let db_pool = chronicler_engine::adapters::driven::storage::db::DbPool::new(":memory:")
        .expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    storage.seed_test_world_fixture();
    let data = TestDataBuilder::default_test().build();
    let world_key = data.world_key();
    let app_service = TestAppBuilder::with_data(data)
        .storage(storage.clone())
        .game_service(Arc::new(GameService::with_backends(
            crate::make_test_recorder(Arc::new(MockBackend::default())),
            AgentRegistry::default(),
        )))
        .skip_seeding(true)
        .build_service();

    let create_result = app_service.create_game(&world_key, "hero");
    assert!(create_result.is_ok(), "create_game should succeed");
    let game_id = app_service.storage().current_game_id();

    let mut world_b = create_test_world_with_scenario();
    world_b.name = "World B".to_string();
    let data_b = TestDataBuilder::default_test().world(world_b).build();
    let _app2 = TestAppBuilder::with_data(data_b)
        .storage(storage.clone())
        .game_service(Arc::new(GameService::with_backends(
            crate::make_test_recorder(Arc::new(MockBackend::default())),
            AgentRegistry::default(),
        )))
        .skip_seeding(true)
        .build_service();

    let result = app_service.switch_game(game_id);
    assert!(
        result.is_ok(),
        "switch_game should succeed even for different worlds"
    );
}

#[test]
fn test_delete_game_removes() {
    let db_pool = chronicler_engine::adapters::driven::storage::db::DbPool::new(":memory:")
        .expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    storage.seed_test_world_fixture();
    let data = TestDataBuilder::default_test().build();
    let world_key = data.world_key();
    let app_service = TestAppBuilder::with_data(data)
        .storage(storage.clone())
        .game_service(Arc::new(GameService::with_backends(
            crate::make_test_recorder(Arc::new(MockBackend::default())),
            AgentRegistry::default(),
        )))
        .skip_seeding(true)
        .build_service();

    app_service.create_game(&world_key, "hero").unwrap();
    let game_id_1 = app_service.storage().current_game_id();

    app_service.create_game(&world_key, "hero").unwrap();
    let game_id_2 = app_service.storage().current_game_id();

    assert_ne!(game_id_1, game_id_2, "Should have different game IDs");

    let delete_result = app_service.delete_game(game_id_1);
    assert!(delete_result.is_ok(), "delete_game should succeed");

    let games = app_service.list_games().unwrap();
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
    let db_pool = chronicler_engine::adapters::driven::storage::db::DbPool::new(":memory:")
        .expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    storage.seed_test_world_fixture();
    let data = TestDataBuilder::default_test().build();
    let world_key = data.world_key();
    let app_service = TestAppBuilder::with_data(data)
        .storage(storage.clone())
        .game_service(Arc::new(GameService::with_backends(
            crate::make_test_recorder(Arc::new(MockBackend::default())),
            AgentRegistry::default(),
        )))
        .skip_seeding(true)
        .build_service();

    app_service.create_game(&world_key, "hero").unwrap();
    let active_game_id = app_service.storage().current_game_id();

    let result = app_service.delete_game(active_game_id);
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
    let db_pool = chronicler_engine::adapters::driven::storage::db::DbPool::new(":memory:")
        .expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    let app_service = TestAppBuilder::default_test()
        .storage(storage.clone())
        .game_service(Arc::new(GameService::with_backends(
            crate::make_test_recorder(Arc::new(MockBackend::default())),
            AgentRegistry::default(),
        )))
        .skip_seeding(true)
        .build_service();

    let result = app_service.delete_game(99999);
    assert!(
        result.is_ok(),
        "delete_game should succeed silently for nonexistent game"
    );
}
