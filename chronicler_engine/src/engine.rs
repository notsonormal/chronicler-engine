use crate::map::Room;
use crate::state::GameState;

pub fn get_room_by_id<'a>(state: &'a GameState, target_id: &str) -> Option<&'a Room> {
    for region in &state.map.overworld.regions {
        for room in &region.rooms {
            if room.id == target_id {
                return Some(room);
            }
        }
    }
    None
}

pub fn get_current_room(state: &GameState) -> &Room {
    get_room_by_id(state, &state.current_room_id).expect("Current room not found in map!")
}

pub fn attempt_walk(state: &mut GameState, target: &str) -> Result<String, String> {
    let current_room = get_current_room(&state);
    let target_lower = target.to_lowercase();
    let mut found_dest: Option<String> = None;

    for (dir, room_id) in &current_room.exits {
        // 1. Check if they typed the direction literal (e.g. "north")
        let dir_str = format!("{:?}", dir).to_lowercase();
        if dir_str == target_lower {
            found_dest = Some(room_id.clone());
            break;
        }

        // 2. Check if they typed the semantic room name (e.g. "kitchen")
        if let Some(dest_room) = get_room_by_id(state, room_id) {
            if dest_room.name.to_lowercase().contains(&target_lower) {
                found_dest = Some(room_id.clone());
                break;
            }
        }
    }

    if let Some(next_room_id) = found_dest {
        state.current_room_id = next_room_id.clone();
        Ok(format!("You walk to: {}.", target))
    } else {
        Err("You don't see a way to go there.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::PlayerCard;
    use crate::map::{Direction, MapDef, Overworld, Region};
    use crate::world::WorldCard;
    use std::collections::HashMap;

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
        };

        let room2 = Room {
            id: "room2".to_string(),
            name: "Dusty Kitchen".to_string(),
            description: "Smells like mold.".to_string(),
            exits: exits2,
            items: vec![],
            npcs: vec![],
        };

        let room3 = Room {
            id: "room3".to_string(),
            name: "Library".to_string(),
            description: "Books everywhere.".to_string(),
            exits: exits3,
            items: vec![],
            npcs: vec![],
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
        };
        let player = PlayerCard {
            name: "P".into(),
            description: "P".into(),
            inventory: vec![],
        };

        GameState::new(world, map, player, vec![], "room1".to_string())
    }

    #[test]
    fn test_attempt_walk_cardinal() {
        let mut state = setup_test_state();
        let res = attempt_walk(&mut state, "north");
        assert!(res.is_ok());
        assert_eq!(state.current_room_id, "room2");
    }

    #[test]
    fn test_attempt_walk_semantic() {
        let mut state = setup_test_state();
        let res = attempt_walk(&mut state, "kitchen");
        assert!(res.is_ok());
        assert_eq!(state.current_room_id, "room2");
    }

    #[test]
    fn test_attempt_walk_semantic_partial() {
        let mut state = setup_test_state();
        let res = attempt_walk(&mut state, "library");
        assert!(res.is_ok());
        assert_eq!(state.current_room_id, "room3");
    }

    #[test]
    fn test_attempt_walk_fail() {
        let mut state = setup_test_state();
        let res = attempt_walk(&mut state, "bathroom");
        assert!(res.is_err());
        assert_eq!(state.current_room_id, "room1"); // did not move
    }
}
