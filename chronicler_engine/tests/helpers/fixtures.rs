#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use chronicler_engine::application::context::GameServiceContext;
use chronicler_engine::domain::model::character::{CharacterSheet, NpcCard, PlayerCard};
use chronicler_engine::domain::model::map::{Direction, MapDef, Overworld, Region, Room};
use chronicler_engine::domain::model::scenario::StartingScenario;
use chronicler_engine::domain::model::settings::AppSettings;
use chronicler_engine::domain::model::state::game_state::GameState;
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driven::storage::db::DbPool;

pub fn create_test_world() -> WorldCard {
    WorldCard {
        key: "test".to_string(),
        name: "Test Realm".to_string(),
        description: "A small testing kingdom".to_string(),
        ..Default::default()
    }
}

pub fn create_test_player() -> PlayerCard {
    PlayerCard {
        key: "test_player".to_string(),
        sheet: CharacterSheet {
            name: "Test Player".to_string(),
            description: "A brave adventurer".to_string(),
            personality: "Brave and curious".to_string(),
            scenario: "Exploring the test realm".to_string(),
            example_dialogue: "Hello, world!".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    }
}

pub fn create_test_map() -> MapDef {
    let mut room1_exits = HashMap::new();
    room1_exits.insert(Direction::North, "room2".to_string());

    let mut room2_exits = HashMap::new();
    room2_exits.insert(Direction::South, "room1".to_string());
    room2_exits.insert(Direction::East, "room3".to_string());

    let room3_exits = HashMap::new();

    let room1 = Room {
        id: "room1".to_string(),
        name: "Test Tavern".to_string(),
        description: "A cozy tavern with wooden beams and warm fire.".to_string(),
        exits: room1_exits,
        items: vec![],
        image_path: None,
        navigation_description: None,
    };

    let room2 = Room {
        id: "room2".to_string(),
        name: "Village Square".to_string(),
        description: "A bustling village square with a fountain.".to_string(),
        exits: room2_exits,
        items: vec![],
        image_path: None,
        navigation_description: None,
    };

    let room3 = Room {
        id: "room3".to_string(),
        name: "Forest Path".to_string(),
        description: "A quiet path through the woods.".to_string(),
        exits: room3_exits,
        items: vec![],
        image_path: None,
        navigation_description: None,
    };

    let region = Region {
        id: "test_region".to_string(),
        name: "Test Region".to_string(),
        rooms: vec![room1, room2, room3],
    };

    let overworld = Overworld {
        id: "test_overworld".to_string(),
        name: "Test World".to_string(),
        regions: vec![region],
    };

    MapDef { overworld }
}

pub fn create_test_npcs() -> Vec<NpcCard> {
    vec![NpcCard {
        id: "test_npc".to_string(),
        sheet: CharacterSheet {
            name: "Innkeeper".to_string(),
            description: "A friendly innkeeper".to_string(),
            personality: "Helpful and cheerful".to_string(),
            scenario: "Runs the local tavern".to_string(),
            example_dialogue: "Welcome, traveler!".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![],
        relationships: vec![],
    }]
}

/// Create a GameState with configurable room NPCs and NPC cards.
pub fn create_test_state_with_npcs(room_npcs: Vec<String>, npcs: Vec<NpcCard>) -> GameState {
    let world = Arc::new(WorldCard {
        key: "test".into(),
        name: "Test World".into(),
        description: "A test world".into(),
        ..Default::default()
    });

    let room1 = Room {
        id: "room1".into(),
        name: "Test Tavern".into(),
        description: "A cozy tavern with wooden beams and warm fire.".into(),
        exits: HashMap::new(),
        items: vec![],
        image_path: None,
        navigation_description: None,
    };

    let region = Region {
        id: "test_region".into(),
        name: "Test Region".into(),
        rooms: vec![room1],
    };

    let map = Arc::new(MapDef {
        overworld: Overworld {
            id: "test_overworld".into(),
            name: "Test World".into(),
            regions: vec![region],
        },
    });

    let player = Arc::new(PlayerCard {
        key: "test_player".to_string(),
        sheet: CharacterSheet {
            name: "Test Player".into(),
            description: "A test player".into(),
            personality: "Brave".into(),
            scenario: "Test scenario".into(),
            example_dialogue: "Hello!".into(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    });

    let mut state = GameState::new(world, map, player, npcs, "room1".to_string());
    for id in room_npcs {
        if let Some(npc) = state.npcs.get(&id).cloned() {
            state.scene.npcs_in_area.push(npc);
        }
    }
    state
}

pub fn create_test_state() -> GameState {
    create_test_state_with_npcs(
        vec!["test_npc".to_string()],
        vec![NpcCard {
            id: "test_npc".into(),
            sheet: CharacterSheet {
                name: "Innkeeper".into(),
                description: "A friendly innkeeper".into(),
                personality: "Helpful".into(),
                scenario: "Runs the tavern".into(),
                example_dialogue: "Welcome!".into(),
                summary: None,
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
            triggers: vec![],
            relationships: vec![],
        }],
    )
}

pub fn create_test_game_state() -> Arc<std::sync::Mutex<GameState>> {
    let world = Arc::new(create_test_world());
    let map = Arc::new(create_test_map());
    let player = Arc::new(create_test_player());
    let npcs = create_test_npcs();

    Arc::new(std::sync::Mutex::new(GameState::new(
        world,
        map,
        player,
        npcs,
        "room1".to_string(),
    )))
}

pub fn create_navigation_test_map() -> MapDef {
    let mut entrance_exits = HashMap::new();
    entrance_exits.insert(Direction::North, "hall".to_string());

    let mut hall_exits = HashMap::new();
    hall_exits.insert(Direction::South, "entrance".to_string());
    hall_exits.insert(Direction::East, "kitchen".to_string());
    hall_exits.insert(Direction::West, "library".to_string());

    let kitchen_exits = HashMap::new();
    let library_exits = HashMap::new();

    let entrance = Room {
        id: "entrance".to_string(),
        name: "Mansion Entrance".to_string(),
        description: "A grand entrance to the mansion.".to_string(),
        exits: entrance_exits,
        items: vec![],
        image_path: None,
        navigation_description: None,
    };

    let hall = Room {
        id: "hall".to_string(),
        name: "Main Hall".to_string(),
        description: "A spacious hall with marble floors.".to_string(),
        exits: hall_exits,
        items: vec![],
        image_path: None,
        navigation_description: None,
    };

    let kitchen = Room {
        id: "kitchen".to_string(),
        name: "Kitchen".to_string(),
        description: "A busy kitchen with delicious smells.".to_string(),
        exits: kitchen_exits,
        items: vec![],
        image_path: None,
        navigation_description: None,
    };

    let library = Room {
        id: "library".to_string(),
        name: "Library".to_string(),
        description: "Rows of ancient books line the walls.".to_string(),
        exits: library_exits,
        items: vec![],
        image_path: None,
        navigation_description: None,
    };

    let region = Region {
        id: "mansion".to_string(),
        name: "Mansion".to_string(),
        rooms: vec![entrance, hall, kitchen, library],
    };

    let overworld = Overworld {
        id: "overworld".to_string(),
        name: "Mansion".to_string(),
        regions: vec![region],
    };

    MapDef { overworld }
}

pub fn create_simple_test_map() -> MapDef {
    let room = Room {
        id: "start".to_string(),
        name: "Start Room".to_string(),
        description: "A simple test room.".to_string(),
        exits: HashMap::new(),
        items: vec![],
        image_path: None,
        navigation_description: None,
    };

    let region = Region {
        id: "test".to_string(),
        name: "Test Area".to_string(),
        rooms: vec![room],
    };

    let overworld = Overworld {
        id: "test_world".to_string(),
        name: "Test World".to_string(),
        regions: vec![region],
    };

    MapDef { overworld }
}

/// A `WorldCard` with a `StartingScenario` block. Promoted from `lifecycle.rs`.
pub fn create_test_world_with_scenario() -> WorldCard {
    WorldCard {
        key: "test".to_string(),
        name: "Test Realm".to_string(),
        description: "A small testing kingdom".to_string(),
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

/// `GameState` using `create_test_world_with_scenario()`. Most integration tests
/// want the scenario variant for `inject_scenario_logs` coverage.
pub fn create_basic_test_state() -> GameState {
    let world = Arc::new(create_test_world_with_scenario());
    let map = Arc::new(create_test_map());
    let player = Arc::new(create_test_player());
    let npcs = Vec::new();
    GameState::new(world, map, player, npcs, "room1".to_string())
}

/// `GameState` using `create_test_world()` (no scenario). Promoted from
/// `application_service.rs` variant.
pub fn create_basic_test_state_no_scenario() -> GameState {
    let world = Arc::new(create_test_world());
    let map = Arc::new(create_test_map());
    let player = Arc::new(create_test_player());
    let npcs = Vec::new();
    GameState::new(world, map, player, npcs, "room1".to_string())
}

/// Seed a test world + persona into the storage. Promoted from `lifecycle.rs`
/// (identical copy existed in `application_service.rs`).
pub fn seed_test_world(storage: &Storage) {
    use chronicler_engine::test_support::{TestMap, TestPlayer, TestWorld};
    let world = TestWorld::minimal();
    let map = TestMap::single_room("start");
    storage.seed_world(&world, &map).expect("seed world");
    let player = TestPlayer::standard();
    storage
        .seed_persona(&player.key, &player)
        .expect("seed persona");
}

/// Construct a `GameServiceContext` from storage + state. Promoted from
/// `lifecycle.rs` (functionally identical copy existed in `application_service.rs`).
pub fn make_test_ctx(storage: Arc<Storage>, state: GameState) -> GameServiceContext {
    GameServiceContext {
        storage,
        world: state.world,
        map: state.map,
        player: state.player,
        npcs: Arc::new(state.npcs),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::new(std::sync::RwLock::new(AppSettings::default())),
        preset_storage: Arc::new(Storage::new_in_memory()),
    }
}

/// Build a sqlite-backed `Storage` with the games row for `game_id` pre-seeded.
/// Use this instead of `Storage::new_sqlite(DbPool::new(":memory:").unwrap(), 1)`
/// to satisfy `game_state_snapshots.game_id` / `messages.game_id` FK constraints.
pub fn create_test_storage(game_id: u64) -> Storage {
    let pool = DbPool::new(":memory:").expect("in-memory db should open");
    chronicler_engine::test_support::seed_default_game_row(&pool, game_id).unwrap();
    Storage::new_sqlite(pool, game_id)
}

/// `Arc`-wrapped variant for tests that share storage across services.
pub fn create_test_storage_arc(game_id: u64) -> Arc<Storage> {
    Arc::new(create_test_storage(game_id))
}
