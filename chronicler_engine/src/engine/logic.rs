use std::collections::HashMap;

use crate::error::{EngineError, Result};
use crate::model::map::{MapDef, Room};
use crate::model::state::GameState;

/// [DOC: docs/system/navigation.md]
pub fn find_room_in_map<'a>(map: &'a MapDef, target_id: &str) -> Option<&'a Room> {
    map.get_room_by_id(target_id)
}

pub fn find_room_in_world_map<'a>(state: &'a GameState, target_id: &str) -> Option<&'a Room> {
    // [DOC: docs/system/navigation.md]
    state.map.get_room_by_id(target_id)
}

pub fn get_current_room(state: &GameState) -> Result<&Room> {
    if let Some(room) = find_room_in_world_map(state, &state.movement.current_room_id) {
        return Ok(room);
    }
    // [DOC: docs/architecture/system.md]
    if let Some(room) = state
        .movement
        .dynamic_rooms
        .get(&state.movement.current_room_id)
    {
        return Ok(room);
    }
    Err(EngineError::RoomNotFound(
        state.movement.current_room_id.clone(),
    ))
}

pub fn get_available_exits(state: &GameState) -> Vec<String> {
    if let Ok(room) = get_current_room(state) {
        room.exits.keys().map(|d| format!("{d:?}")).collect()
    } else {
        vec![]
    }
}

/// [DOC: docs/system/navigation.md]
pub fn process_directional_movement(state: &mut GameState, target: &str) -> Result<String> {
    let current_room = get_current_room(state)?;
    let target_lower = target.to_lowercase();
    let mut found_dest: Option<String> = None;

    for (dir, room_id) in &current_room.exits {
        let dir_str = format!("{dir:?}").to_lowercase();
        if dir_str == target_lower {
            found_dest = Some(room_id.clone());
            break;
        }

        if let Some(dest_room) = find_room_in_world_map(state, room_id)
            && dest_room.name.to_lowercase().contains(&target_lower)
        {
            found_dest = Some(room_id.clone());
            break;
        }
    }

    if let Some(next_room_id) = found_dest {
        state.movement.current_room_id = next_room_id;
        Ok(format!("You walk to: {target}."))
    } else {
        Err(EngineError::Navigation(
            "You don't see a way to go there.".to_string(),
        ))
    }
}

/// [DOC: docs/system/navigation.md]
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

/// [DOC: docs/architecture/system.md]
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
