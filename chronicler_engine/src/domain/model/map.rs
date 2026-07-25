//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Map and location data structures

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MapDef {
    pub overworld: Overworld,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Overworld {
    pub id: String,
    pub name: String,
    pub regions: Vec<Region>,
}

impl MapDef {
    pub fn get_room_by_id(&self, room_id: &str) -> Option<&Room> {
        self.overworld
            .regions
            .iter()
            .flat_map(|region| &region.rooms)
            .find(|room| room.id == room_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub id: String,
    pub name: String,
    pub rooms: Vec<Room>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub description: String,
    pub exits: HashMap<Direction, String>,
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default)]
    pub image_path: Option<String>,
    #[serde(default)]
    pub navigation_description: Option<String>,
}

impl Room {
    pub fn new_dynamic(name: &str, description: &str) -> Self {
        use std::time::SystemTime;
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: format!("dynamic_{timestamp}"),
            name: name.to_string(),
            description: description.to_string(),
            exits: HashMap::new(),
            items: vec![],
            image_path: None,
            navigation_description: None,
        }
    }
}

/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}
