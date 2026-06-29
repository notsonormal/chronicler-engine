//! [DOC: docs/system/navigation.md]
//! Game logic and rule evaluation

use std::collections::HashMap;

use crate::error::{EngineError, Result};
use crate::domain::model::map::{MapDef, Room};
use crate::domain::model::state::game_state::GameState;

pub fn find_room_in_map<'a>(map: &'a MapDef, target_id: &str) -> Option<&'a Room> {
    map.get_room_by_id(target_id)
}

pub fn find_room_in_world_map<'a>(state: &'a GameState, target_id: &str) -> Option<&'a Room> {
    state.map.get_room_by_id(target_id)
}

pub fn attempt_semantic_walk(state: &mut GameState, room_id: &str) -> Result<String> {
    let room_name = if let Some(room) = find_room_in_world_map(state, room_id) {
        room.name.clone()
    } else if let Some(room) = state.movement.dynamic_rooms.get(room_id) {
        room.name.clone()
    } else {
        return Err(EngineError::Navigation(
            "You don't see a way to go there.".to_string(),
        ));
    };

    state.movement.current_room_id = room_id.to_string();
    Ok(format!("You go to: {room_name}."))
}

/// Creates a dynamic (pseudo) room for invalid destinations.
///
/// When the quantifier detects movement intent but the destination doesn't exist in the
/// static map, the engine creates a placeholder room so the player can still proceed.
/// Dynamic rooms are stored in `state.movement.dynamic_rooms` and persist for the session.
///
pub fn create_dynamic_room(name: &str, description: &str) -> Room {
    use std::time::SystemTime;
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let room_id = format!("dynamic_{timestamp}");

    Room {
        id: room_id,
        name: name.to_string(),
        description: description.to_string(),
        exits: HashMap::new(),
        items: vec![],
        image_path: None,
        navigation_description: None,
    }
}
