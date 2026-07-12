use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::engine::logic::{attempt_semantic_walk, create_dynamic_room};
use crate::domain::model::map::{Direction, MapDef, Overworld, Region, Room};
use crate::domain::model::state::game_state::GameState;

fn setup_test_state() -> (GameState, Arc<MapDef>) {
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
        image_path: None,
        navigation_description: None,
    };

    let room2 = Room {
        id: "room2".to_string(),
        name: "Dusty Kitchen".to_string(),
        description: "Smells like mold.".to_string(),
        exits: exits2,
        items: vec![],
        image_path: None,
        navigation_description: None,
    };

    let room3 = Room {
        id: "room3".to_string(),
        name: "Library".to_string(),
        description: "Books everywhere.".to_string(),
        exits: exits3,
        items: vec![],
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

    let state = GameState::new("room1".to_string());
    (state, Arc::new(map))
}

#[test]
fn test_current_room_success() {
    let (state, map) = setup_test_state();
    let result = map
        .get_room_by_id(&state.movement.current_room_id)
        .or_else(|| {
            state
                .movement
                .dynamic_rooms
                .get(&state.movement.current_room_id)
        });
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "Grand Hall");
}

#[test]
fn test_get_current_room_failure() {
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
                    image_path: None,
                    navigation_description: None,
                }],
            }],
        },
    };

    let state = GameState::new("non_existent_room");

    let result = map
        .get_room_by_id(&state.movement.current_room_id)
        .or_else(|| {
            state
                .movement
                .dynamic_rooms
                .get(&state.movement.current_room_id)
        });
    assert!(result.is_none());
}

#[test]
fn test_attempt_semantic_walk_valid() {
    let (mut state, map) = setup_test_state();
    let result = attempt_semantic_walk(&mut state, &map, "room2");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Dusty Kitchen"));
    assert_eq!(state.movement.current_room_id, "room2");
}

#[test]
fn test_attempt_semantic_walk_invalid() {
    let (mut state, map) = setup_test_state();
    let result = attempt_semantic_walk(&mut state, &map, "nonexistent_room");
    assert!(result.is_err());
    assert_eq!(state.movement.current_room_id, "room1");
}

#[test]
fn test_attempt_semantic_walk_empty() {
    let (mut state, map) = setup_test_state();
    let result = attempt_semantic_walk(&mut state, &map, "");
    assert!(result.is_err(), "Empty room id should return error");
}

#[test]
fn test_attempt_semantic_walk_dynamic_room() {
    let (mut state, map) = setup_test_state();
    let dynamic = create_dynamic_room("Secret Cave", "Dark and damp.");
    let dynamic_id = dynamic.id.clone();
    state
        .movement
        .dynamic_rooms
        .insert(dynamic_id.clone(), dynamic);

    let result = attempt_semantic_walk(&mut state, &map, &dynamic_id);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Secret Cave"));
    assert_eq!(state.movement.current_room_id, dynamic_id);
}

#[test]
fn test_create_dynamic_room() {
    let room = create_dynamic_room("Test Room", "A test room.");
    assert_eq!(room.name, "Test Room");
    assert_eq!(room.description, "A test room.");
    assert!(room.id.starts_with("dynamic_"));
    assert!(room.exits.is_empty());
    assert!(room.items.is_empty());
}
