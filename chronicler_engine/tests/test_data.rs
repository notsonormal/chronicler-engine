use chronicler_engine::model::character::{CharacterSheet, NpcCard, PlayerCard};
use chronicler_engine::model::map::{Direction, MapDef, Overworld, Region, Room};
use chronicler_engine::model::state::GameState;
use chronicler_engine::model::world::WorldCard;
use std::collections::HashMap;
use std::sync::Arc;

pub fn create_test_world() -> WorldCard {
    WorldCard {
        name: "Test Realm".to_string(),
        description: "A small testing kingdom".to_string(),
        global_rules: vec![],
        default_room_image: None,
    }
}

pub fn create_test_player() -> PlayerCard {
    PlayerCard {
        sheet: CharacterSheet {
            name: "Test Player".to_string(),
            description: "A brave adventurer".to_string(),
            personality: "Brave and curious".to_string(),
            scenario: "Exploring the test realm".to_string(),
            example_dialogue: "Hello, world!".to_string(),
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
        npcs: vec!["test_npc".to_string()],
        image_path: None,
        navigation_description: None,
    };

    let room2 = Room {
        id: "room2".to_string(),
        name: "Village Square".to_string(),
        description: "A bustling village square with a fountain.".to_string(),
        exits: room2_exits,
        items: vec![],
        npcs: vec![],
        image_path: None,
        navigation_description: None,
    };

    let room3 = Room {
        id: "room3".to_string(),
        name: "Forest Path".to_string(),
        description: "A quiet path through the woods.".to_string(),
        exits: room3_exits,
        items: vec![],
        npcs: vec![],
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
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    }]
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
        npcs: vec![],
        image_path: None,
        navigation_description: None,
    };

    let hall = Room {
        id: "hall".to_string(),
        name: "Main Hall".to_string(),
        description: "A spacious hall with marble floors.".to_string(),
        exits: hall_exits,
        items: vec![],
        npcs: vec![],
        image_path: None,
        navigation_description: None,
    };

    let kitchen = Room {
        id: "kitchen".to_string(),
        name: "Kitchen".to_string(),
        description: "A busy kitchen with delicious smells.".to_string(),
        exits: kitchen_exits,
        items: vec![],
        npcs: vec!["chef".to_string()],
        image_path: None,
        navigation_description: None,
    };

    let library = Room {
        id: "library".to_string(),
        name: "Library".to_string(),
        description: "Rows of ancient books line the walls.".to_string(),
        exits: library_exits,
        items: vec![],
        npcs: vec![],
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

/// Single room map for basic UI tests
pub fn create_simple_test_map() -> MapDef {
    let room = Room {
        id: "start".to_string(),
        name: "Start Room".to_string(),
        description: "A simple test room.".to_string(),
        exits: HashMap::new(),
        items: vec![],
        npcs: vec![],
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
