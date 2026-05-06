use std::collections::HashMap;
use std::sync::Arc;

use crate::engine::logic::{
    find_room_in_world_map, get_available_exits, get_current_room, process_directional_movement,
};
use crate::model::character::{CharacterSheet, PlayerCard};
use crate::model::map::{Direction, MapDef, Overworld, Region, Room};
use crate::model::state::GameState;
use crate::model::world::WorldCard;

fn setup_test_state() -> GameState {
    let mut exits1 = HashMap::new();
    exits1.insert(Direction::North, "room2".to_string());
    exits1.insert(Direction::East, "room3".to_string());

    let mut exits2 = HashMap::new();
    exits2.insert(Direction::South, "room1".to_string());

    let mut exits3 = HashMap::new();
    exits3.insert(Direction::West, "room1".to_string());

    let room1 = Room {
        id: "room1".to_string(),
        name: "Grand Hall".to_string(),
        description: "A huge hall.".to_string(),
        exits: exits1,
        items: vec![],
        npcs: vec![],
        image_path: None,
        navigation_description: None,
    };

    let room2 = Room {
        id: "room2".to_string(),
        name: "Dusty Kitchen".to_string(),
        description: "Smells like mold.".to_string(),
        exits: exits2,
        items: vec![],
        npcs: vec![],
        image_path: None,
        navigation_description: None,
    };

    let room3 = Room {
        id: "room3".to_string(),
        name: "Library".to_string(),
        description: "Books everywhere.".to_string(),
        exits: exits3,
        items: vec![],
        npcs: vec![],
        image_path: None,
        navigation_description: None,
    };

    let region = Region {
        id: "reg1".to_string(),
        name: "Mansion".to_string(),
        rooms: vec![room1, room2, room3],
    };

    let overworld = Overworld {
        id: "ow1".to_string(),
        name: "World".to_string(),
        regions: vec![region],
    };

    let map = MapDef { overworld };
    let world = WorldCard {
        name: "W".into(),
        description: "D".into(),
        global_rules: vec![],
        ..Default::default()
    };
    let player = PlayerCard {
        sheet: CharacterSheet {
            name: "P".into(),
            description: "P".into(),
            personality: "P".into(),
            scenario: "S".into(),
            example_dialogue: "E".into(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    };

    GameState::new(
        Arc::new(world),
        Arc::new(map),
        Arc::new(player),
        vec![],
        "room1".to_string(),
    )
}

#[test]
fn test_attempt_walk_cardinal() {
    let mut state = setup_test_state();
    let res = process_directional_movement(&mut state, "north");
    assert!(res.is_ok());
    assert_eq!(state.current_room_id, "room2");
}

#[test]
fn test_attempt_walk_semantic() {
    let mut state = setup_test_state();
    let res = process_directional_movement(&mut state, "kitchen");
    assert!(res.is_ok());
    assert_eq!(state.current_room_id, "room2");
}

#[test]
fn test_attempt_walk_semantic_partial() {
    let mut state = setup_test_state();
    let res = process_directional_movement(&mut state, "library");
    assert!(res.is_ok());
    assert_eq!(state.current_room_id, "room3");
}

#[test]
fn test_attempt_walk_fail() {
    let mut state = setup_test_state();
    let res = process_directional_movement(&mut state, "bathroom");
    assert!(res.is_err());
    assert_eq!(state.current_room_id, "room1");

    let err = res.unwrap_err();
    assert!(err.to_string().contains("don't see a way"));
}

#[test]
fn test_get_room_by_id_missing() {
    let state = setup_test_state();
    let res = find_room_in_world_map(&state, "phantom_room");
    assert!(res.is_none());
}

#[test]
fn test_attempt_walk_dangling_exit() {
    let state = setup_test_state();
    let mut room = state.map.overworld.regions[0].rooms[0].clone();
    room.exits
        .insert(Direction::South, "non_existent_id".to_string());

    // [DOC: docs/architecture/system.md]
    // NOTE: Map is behind Arc, so we clone the reference for the guard.
    let res = find_room_in_world_map(&state, "non_existent_id");
    assert!(res.is_none());
}

#[test]
fn test_get_available_exits() {
    let state = setup_test_state();
    let exits = get_available_exits(&state);
    assert_eq!(exits.len(), 2);
    assert!(exits.contains(&"north".to_string()) || exits.contains(&"North".to_string()));
    assert!(exits.contains(&"east".to_string()) || exits.contains(&"East".to_string()));
}

#[test]
fn test_get_available_exits_no_exits() {
    let world = Arc::new(WorldCard {
        name: "W".into(),
        description: "D".into(),
        global_rules: vec![],
        ..Default::default()
    });

    let room_no_exits = Room {
        id: "empty".to_string(),
        name: "Empty Room".to_string(),
        description: "Nothing here.".to_string(),
        exits: HashMap::new(),
        items: vec![],
        npcs: vec![],
        image_path: None,
        navigation_description: None,
    };

    let map = MapDef {
        overworld: Overworld {
            id: "o".into(),
            name: "W".into(),
            regions: vec![Region {
                id: "r".into(),
                name: "R".into(),
                rooms: vec![room_no_exits],
            }],
        },
    };

    let state = GameState::new(
        world,
        Arc::new(map),
        Arc::new(PlayerCard {
            sheet: CharacterSheet {
                name: "P".into(),
                description: "P".into(),
                personality: "P".into(),
                scenario: "S".into(),
                example_dialogue: "".into(),
                summary: None,
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
        }),
        vec![],
        "empty".to_string(),
    );

    let exits = get_available_exits(&state);
    assert!(exits.is_empty());
}

#[test]
fn test_get_current_room_success() {
    let state = setup_test_state();
    let result = get_current_room(&state);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name, "Grand Hall");
}

#[test]
fn test_get_current_room_failure() {
    let world = Arc::new(WorldCard {
        name: "W".into(),
        description: "D".into(),
        global_rules: vec![],
        ..Default::default()
    });

    let map = MapDef {
        overworld: Overworld {
            id: "o".into(),
            name: "W".into(),
            regions: vec![Region {
                id: "r".into(),
                name: "R".into(),
                rooms: vec![Room {
                    id: "room1".into(),
                    name: "Room".to_string(),
                    description: "D".to_string(),
                    exits: HashMap::new(),
                    items: vec![],
                    npcs: vec![],
                    image_path: None,
                    navigation_description: None,
                }],
            }],
        },
    };

    let state = GameState::new(
        world,
        Arc::new(map),
        Arc::new(PlayerCard {
            sheet: CharacterSheet {
                name: "P".into(),
                description: "P".into(),
                personality: "P".into(),
                scenario: "S".into(),
                example_dialogue: "".into(),
                summary: None,
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
        }),
        vec![],
        "non_existent_room".to_string(),
    );

    let result = get_current_room(&state);
    assert!(result.is_err());
}

#[test]
fn test_get_room_by_id_existing() {
    let state = setup_test_state();
    let room = find_room_in_world_map(&state, "room1");
    assert!(room.is_some());
    assert_eq!(room.unwrap().name, "Grand Hall");
}

#[test]
fn test_attempt_walk_case_insensitive() {
    let mut state = setup_test_state();
    let res = process_directional_movement(&mut state, "NORTH");
    assert!(res.is_ok());
    assert_eq!(state.current_room_id, "room2");

    state.current_room_id = "room1".to_string();

    let res = process_directional_movement(&mut state, "North");
    assert!(res.is_ok());
    assert_eq!(state.current_room_id, "room2");
}
