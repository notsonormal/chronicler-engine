/// Integration tests for GameLifecycleService
/// [DOC: docs/reference/testing.md]
mod test_data;

use std::collections::HashMap;
use std::sync::Arc;

use chronicler_engine::application::game_lifecycle::GameLifecycleService;
use chronicler_engine::application::game_service::DefaultGameService;
use chronicler_engine::model::character::{CharacterSheet, PlayerCard};
use chronicler_engine::model::map::{MapDef, Overworld, Region, Room};
use chronicler_engine::model::scenario::StartingScenario;
use chronicler_engine::model::state::{GameState, MessageType};
use chronicler_engine::model::world::WorldCard;
use chronicler_engine::narrative::agents::registry::AgentRegistry;
use chronicler_engine::narrative::llm::MockBackend;
use chronicler_engine::storage::Storage;

fn create_game_service() -> Arc<DefaultGameService> {
    Arc::new(DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    ))
}

fn create_test_world_with_scenario() -> WorldCard {
    WorldCard {
        name: "Test Realm".to_string(),
        description: "A small testing kingdom".to_string(),
        global_rules: vec![],
        starting_room_id: "room1".to_string(),
        scenarios: vec![StartingScenario {
            id: "test_scenario".to_string(),
            name: "Test Scenario".to_string(),
            description: "A test".to_string(),
            starting_room_id: "room1".to_string(),
            text: "You wake up in a cozy room.".to_string(),
            npcs: vec![],
        }],
        default_room_image: None,
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
    _game_service: Arc<DefaultGameService>,
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

#[test]
fn test_create_game_with_scenario() {
    let db_pool =
        chronicler_engine::storage::db::DbPool::new(":memory:").expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let lifecycle_service = GameLifecycleService::new(game_service.clone());

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
fn test_create_game_without_scenario() {
    let db_pool =
        chronicler_engine::storage::db::DbPool::new(":memory:").expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    let game_service = create_game_service();

    let mut world = create_test_world_with_scenario();
    world.scenarios = vec![];
    let world = Arc::new(world);
    let player = Arc::new(create_test_player());
    let map = Arc::new(create_test_map());
    let npcs = Vec::new();
    let state = GameState::new(world.clone(), map, player, npcs, "room1".to_string());

    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);
    let lifecycle_service = GameLifecycleService::new(game_service.clone());

    let result = lifecycle_service.create_game(ctx.clone());
    assert!(
        result.is_ok(),
        "create_game should succeed even without scenario: {:?}",
        result.err()
    );
}

#[test]
fn test_reset_preserves_world_settings() {
    let db_pool =
        chronicler_engine::storage::db::DbPool::new(":memory:").expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let lifecycle_service = GameLifecycleService::new(game_service.clone());

    lifecycle_service.create_game(ctx.clone()).unwrap();
    let first_game_id = ctx.storage.current_game_id();
    let world_name_before = ctx.world.name.clone();
    let world_desc_before = ctx.world.description.clone();

    let reset_result = lifecycle_service.reset(ctx.clone());
    assert!(
        reset_result.is_ok(),
        "reset should succeed: {:?}",
        reset_result.err()
    );

    assert_eq!(
        ctx.world.name, world_name_before,
        "World name should be preserved after reset"
    );
    assert_eq!(
        ctx.world.description, world_desc_before,
        "World description should be preserved after reset"
    );

    let new_game_id = ctx.storage.current_game_id();
    assert_ne!(
        new_game_id, first_game_id,
        "Reset should create a new game instance with different ID"
    );
}

#[test]
fn test_reset_creates_scenario_message() {
    let db_pool =
        chronicler_engine::storage::db::DbPool::new(":memory:").expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let lifecycle_service = GameLifecycleService::new(game_service.clone());

    lifecycle_service.create_game(ctx.clone()).unwrap();

    let _messages_before = storage.load_message_rows().unwrap();

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
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let lifecycle_service = GameLifecycleService::new(game_service.clone());

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
}

#[test]
fn test_list_games_returns_all() {
    let db_pool =
        chronicler_engine::storage::db::DbPool::new(":memory:").expect("DbPool creation failed");
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let lifecycle_service = GameLifecycleService::new(game_service.clone());

    lifecycle_service.create_game(ctx.clone()).unwrap();
    lifecycle_service.create_game(ctx.clone()).unwrap();
    lifecycle_service.create_game(ctx.clone()).unwrap();

    let games = lifecycle_service.list_games(ctx.clone()).unwrap();
    assert!(games.len() >= 3, "Should have created at least 3 games");
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

    let lifecycle_service = GameLifecycleService::new(game_service.clone());

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

    let lifecycle_service = GameLifecycleService::new(game_service.clone());

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

    let lifecycle_service = GameLifecycleService::new(game_service.clone());

    let result = lifecycle_service.reset(ctx.clone());
    assert!(
        result.is_ok(),
        "reset should succeed with default game: {:?}",
        result.err()
    );
}
