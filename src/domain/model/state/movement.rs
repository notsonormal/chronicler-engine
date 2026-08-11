//! [DOC: docs/diataxis/reference/game_flow.md]
//! Player movement state

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::domain::model::map::Room;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MovementState {
    pub current_room_id: String,
    pub dynamic_rooms: HashMap<String, Room>,
}
