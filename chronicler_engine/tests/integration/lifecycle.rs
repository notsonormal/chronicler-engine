// Integration tests for GameLifecycleService
// [DOC: docs/reference/testing.md]
use std::collections::HashMap;
use std::sync::Arc;

use chronicler_engine::application::game_lifecycle::GameLifecycleService;
use chronicler_engine::application::game_service::GameService;
use chronicler_engine::model::character::{CharacterSheet, PlayerCard};
use chronicler_engine::model::map::{MapDef, Overworld, Region, Room};
use chronicler_engine::model::scenario::StartingScenario;
use chronicler_engine::model::state::{GameState, MessageType};
use chronicler_engine::model::world::WorldCard;
use chronicler_engine::narrative::agents::registry::AgentRegistry;
use chronicler_engine::narrative::llm::MockBackend;
use chronicler_engine::storage::Storage;

fn create_game_service() -> Arc<GameService> {
    Arc::new(GameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    ))
}

fn create_test_world_with_scenario() -> WorldCard {
    WorldCard {
        key: "test".to_string(),
        name: "Test Realm".to_string(),
        description: "A small testing kingdom".to_string(),
        player_key: "player".to_string(),
        starting_room_id: "room1".to_string(),
        scenarios: vec![StartingScenario {
            id: "test_scenario".to_string(),
            name: "Test Scenario".to_string(),
            description: "A test".to_string(),
            starting_room_id: "room1".to_string(),
            text: "You wake up in a cozy room.".to_string(),
            npcs: vec![],
        }],
        ..Default::default()
    }
}

fn create_test_map() -> MapDef {
    let room1_exits = HashMap::new();
    let room1 = Room {
        id: "room1".into(),
        name: "Test Room".into(),
        description: "A test room".into(),
        exits: room1_exits,
        items: vec![],
        image_path: None,
        navigation_description: None,
    };
    let region = Region {
        id: "test_region".into(),
        name: "Test Region".into(),
        rooms: vec![room1],
    };
    MapDef {
        overworld: Overworld {
            id: "test_overworld".into(),
            name: "Test World".into(),
            regions: vec![region],
        },
    }
}

fn create_test_player() -> PlayerCard {
    PlayerCard {
        key: "test_player".to_string(),
        sheet: CharacterSheet {
            name: "Test Player".to_string(),
            description: "A test player".to_string(),
            personality: "Brave".to_string(),
            scenario: "Test scenario".to_string(),
            example_dialogue: "Hello!".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    }
}

fn create_basic_test_state() -> GameState {
    let world = Arc::new(create_test_world_with_scenario());
    let map = Arc::new(create_test_map());
    let player = Arc::new(create_test_player());
    let npcs = Vec::new();
    GameState::new(world, map, player, npcs, "room1".to_string())
}

fn make_test_ctx(
    storage: Arc<Storage>,
    _game_service: Arc<GameService>,
    state: GameState,
) -> chronicler_engine::application::context::GameServiceContext {
    chronicler_engine::application::context::GameServiceContext {
        storage,
        world: state.world,
        map: state.map,
        player: state.player,
        npcs: Arc::new(state.npcs),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(std::sync::RwLock::new(
            chronicler_engine::model::settings::AppSettings::default(),
        )),
        preset_storage: Arc::new(Storage::new_in_memory()),
    }
}

/// Seed a test world for game-related tests
fn seed_test_world(storage: &Storage) {
    use chronicler_engine::test_support::{TestWorld, TestMap, TestPlayer};
    let world = TestWorld::minimal();
    let map = TestMap::single_room("start");
    storage.seed_world(&world, &map).expect("seed world");
    let player = TestPlayer::standard();
    storage
        .seed_persona(&world.player_key, &player)
        .expect("seed persona");
}

#[test]
fn test_create_game_with_scenario() {
    let db_pool =
        chronicler_engine::storage::db::DbPool::new(":memory:").expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    seed_test_world(&storage);
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let lifecycle_service = GameLifecycleService::new();

    let result = lifecycle_service.create_game(ctx.clone());
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
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let lifecycle_service = GameLifecycleService::new();

    lifecycle_service.create_game(ctx.clone()).unwrap();

    lifecycle_service.reset(ctx.clone()).unwrap();

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
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let lifecycle_service = GameLifecycleService::new();

    lifecycle_service.create_game(ctx.clone()).unwrap();
    let game1_id = ctx.storage.current_game_id();

    lifecycle_service.create_game(ctx.clone()).unwrap();
    let game2_id = ctx.storage.current_game_id();
    assert_ne!(game1_id, game2_id, "Should have different game IDs");

    let switch_result = lifecycle_service.switch_game(ctx.clone(), game1_id);
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

    let switch_result = lifecycle_service.switch_game(ctx.clone(), game2_id);
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
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    ctx.is_generating.store(true, Ordering::SeqCst);

    let lifecycle_service = GameLifecycleService::new();

    let result = lifecycle_service.create_game(ctx.clone());
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
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let lifecycle_service = GameLifecycleService::new();

    let result = lifecycle_service.switch_game(ctx.clone(), 99999);
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
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let lifecycle_service = GameLifecycleService::new();

    let result = lifecycle_service.reset(ctx.clone());
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
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let lifecycle_service = GameLifecycleService::new();

    let result1 = lifecycle_service.create_game(ctx.clone());
    assert!(result1.is_ok(), "First create_game should succeed");

    let result2 = lifecycle_service.create_game(ctx.clone());
    assert!(result2.is_ok(), "Second create_game should succeed");

    let games = lifecycle_service.list_games(ctx.clone()).unwrap();
    // Filter to only the games we created (exclude "default" game)
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
    let game_service = create_game_service();

    let mut world_a = create_test_world_with_scenario();
    world_a.name = "World A".to_string();
    let world_a = Arc::new(world_a);
    let player = Arc::new(create_test_player());
    let map = Arc::new(create_test_map());
    let npcs = Vec::new();
    let state_a = GameState::new(world_a.clone(), map, player, npcs, "room1".to_string());

    let ctx_a = make_test_ctx(storage.clone(), game_service.clone(), state_a);
    let lifecycle_service = GameLifecycleService::new();

    let create_result = lifecycle_service.create_game(ctx_a.clone());
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

    let ctx_b = make_test_ctx(storage.clone(), game_service.clone(), state_b);

    // Cross-world switching is now allowed - world context loads from DB
    let result = lifecycle_service.switch_game(ctx_b, game_id);
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
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let lifecycle_service = GameLifecycleService::new();

    lifecycle_service.create_game(ctx.clone()).unwrap();
    let game_id_1 = ctx.storage.current_game_id();

    lifecycle_service.create_game(ctx.clone()).unwrap();
    let game_id_2 = ctx.storage.current_game_id();

    assert_ne!(game_id_1, game_id_2, "Should have different game IDs");

    let delete_result = lifecycle_service.delete_game(ctx.clone(), game_id_1);
    assert!(delete_result.is_ok(), "delete_game should succeed");

    let games = lifecycle_service.list_games(ctx.clone()).unwrap();
    assert_eq!(games.len(), 2, "Should have exactly 2 games after deletion");
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
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let lifecycle_service = GameLifecycleService::new();

    lifecycle_service.create_game(ctx.clone()).unwrap();
    let active_game_id = ctx.storage.current_game_id();

    let result = lifecycle_service.delete_game(ctx.clone(), active_game_id);
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
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let lifecycle_service = GameLifecycleService::new();

    let result = lifecycle_service.delete_game(ctx.clone(), 99999);
    assert!(
        result.is_ok(),
        "delete_game should succeed silently for nonexistent game"
    );
}
