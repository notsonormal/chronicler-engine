use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapDef {
    pub overworld: Overworld,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Overworld {
    pub id: String,
    pub name: String,
    pub regions: Vec<Region>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub id: String,
    pub name: String,
    pub rooms: Vec<Room>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub description: String,
    pub exits: HashMap<Direction, String>,
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default)]
    pub npcs: Vec<String>,
    #[serde(default)]
    pub image_path: Option<String>,
    #[serde(default)]
    pub navigation_description: Option<String>,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_serde() {
        let json = r#"{
            "overworld": {
                "id": "ow_1",
                "name": "World",
                "regions": [
                    {
                        "id": "reg_1",
                        "name": "Start",
                        "rooms": [
                            {
                                "id": "room_1",
                                "name": "Tavern",
                                "description": "A tavern.",
                                "exits": {
                                    "north": "room_2"
                                },
                                "image_path": "data/images/tavern.png"
                            }
                        ]
                    }
                ]
            }
        }"#;

        let map: MapDef = serde_json::from_str(json).unwrap();
        assert_eq!(map.overworld.id, "ow_1");
        assert_eq!(
            map.overworld.regions[0].rooms[0]
                .exits
                .get(&Direction::North)
                .unwrap(),
            "room_2"
        );
        assert_eq!(
            map.overworld.regions[0].rooms[0].image_path,
            Some("data/images/tavern.png".to_string())
        );
    }
}
