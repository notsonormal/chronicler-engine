//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Game logic and rule evaluation

use std::collections::HashMap;

use crate::error::{EngineError, Result};
use crate::domain::model::map::{MapDef, Room};
use crate::domain::model::state::game_state::GameState;

pub fn attempt_semantic_walk(state: &mut GameState, map: &MapDef, room_id: &str) -> Result<String> {
    let room_name = if let Some(room) = map.get_room_by_id(room_id) {
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

/// Spawns a placeholder room when the quantifier detects movement to an unmapped destination. Persists for session in `state.movement.dynamic_rooms`.
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
