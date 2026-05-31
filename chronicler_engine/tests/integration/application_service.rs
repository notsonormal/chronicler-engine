/// Integration tests for DefaultApplicationService
mod test_data;

use std::sync::Arc;

use chronicler_engine::application::application_service::{DefaultApplicationService};
use chronicler_engine::application::game_service::DefaultGameService;
use chronicler_engine::model::character::{CharacterSheet, PlayerCard};
use chronicler_engine::model::map::{MapDef, Overworld, Region, Room};
use chronicler_engine::model::state::GameState;
use chronicler_engine::model::world::WorldCard;
use chronicler_engine::narrative::agents::registry::AgentRegistry;
use chronicler_engine::narrative::llm::MockBackend;
use chronicler_engine::storage::{Storage, db::DbPool};
use chronicler_engine::application::context::GameServiceContext;
use chronicler_engine::model::state::{GenerationStatus, GenerationPhase};

fn create_game_service() -> Arc<DefaultGameService> {
    Arc::new(DefaultGameService::with_backends(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
    ))
}

fn create_test_world() -> WorldCard {
    WorldCard {
        name: "Test Realm".to_string(),
        description: "A testing kingdom".to_string(),
        global_rules: vec![],
        starting_room_id: "room1".to_string(),
        scenarios: vec![],
        default_room_image: None,
    }
}

fn create_test_map() -> MapDef {
    let room1_exits = std::collections::HashMap::new();
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
            scenario: "Test".to_string(),
            example_dialogue: "Hello!".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    }
}

fn create_basic_test_state() -> GameState {
    let world = Arc::new(create_test_world());
    let map = Arc::new(create_test_map());
    let player = Arc::new(create_test_player());
    let npcs = Vec::new();
    GameState::new(world, map, player, npcs, "room1".to_string())
}

fn make_test_ctx(
    storage: Arc<Storage>,
    _game_service: Arc<DefaultGameService>,
    state: GameState,
) -> GameServiceContext {
    GameServiceContext {
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

fn create_storage() -> Arc<Storage> {
    let pool = DbPool::new(":memory:").unwrap();
    Arc::new(Storage::new_sqlite(pool, 1))
}

#[test]
fn test_create_game_integration() {
    let storage = create_storage();
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let app_service = DefaultApplicationService::new(game_service);

    let game_id = app_service
        .create_game(ctx.clone())
        .expect("create_game should succeed");
    assert!(game_id > 0, "Game ID should be positive");

    let game = storage.get_game(game_id).unwrap();
    assert!(game.is_some(), "Game should be persisted");
}

#[test]
fn test_switch_game_integration() {
    let storage = create_storage();
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let app_service = DefaultApplicationService::new(game_service);

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
    let storage = create_storage();
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let app_service = DefaultApplicationService::new(game_service);

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
    let storage = create_storage();
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let app_service = DefaultApplicationService::new(game_service);

    app_service.create_game(ctx.clone()).unwrap();
    app_service.create_game(ctx.clone()).unwrap();

    let games = app_service.list_games(ctx.clone()).unwrap();
    assert!(games.len() >= 2, "Should list all games");
}

#[test]
fn test_get_generating_status() {
    let storage = create_storage();
    let game_service = create_game_service();
    let state = create_basic_test_state();
    let ctx = make_test_ctx(storage.clone(), game_service.clone(), state);

    let app_service = DefaultApplicationService::new(game_service);

    let (status, phase) = app_service.get_generating_status(ctx.clone()).unwrap();
    assert_eq!(status, GenerationStatus::Idle);
    assert_eq!(phase, GenerationPhase::default());
}
