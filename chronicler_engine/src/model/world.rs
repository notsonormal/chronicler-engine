use serde::{Deserialize, Serialize};

use crate::model::scenario::StartingScenario;

/// Represents the overarching rules and scenario for the game world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldCard {
    pub name: String,
    pub description: String,
    pub global_rules: Vec<String>,
}

/// Extended world manifest with loading metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub global_rules: Vec<String>,
    #[serde(default = "default_starting_room")]
    pub starting_room_id: String,
    #[serde(default = "default_map_file")]
    pub map_file: String,
    #[serde(default = "default_player_file")]
    pub player_file: String,
    #[serde(default)]
    pub scenarios: Vec<StartingScenario>,
    #[serde(default)]
    pub default_scenario_id: Option<String>,
}

impl WorldManifest {
    /// Returns the first scenario in the scenarios list, if any.
    pub fn default_scenario(&self) -> Option<&StartingScenario> {
        self.scenarios.first()
    }
}

fn default_starting_room() -> String {
    "start".to_string()
}

fn default_map_file() -> String {
    "map.json".to_string()
}

fn default_player_file() -> String {
    "player.json".to_string()
}

impl From<WorldManifest> for WorldCard {
    fn from(manifest: WorldManifest) -> Self {
        WorldCard {
            name: manifest.name,
            description: manifest.description,
            global_rules: manifest.global_rules,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_card_serde() {
        let json = r#"{
            "name": "Test World",
            "description": "A test world.",
            "global_rules": ["Rule 1"]
        }"#;

        let card: WorldCard = serde_json::from_str(json).unwrap();
        assert_eq!(card.name, "Test World");
        assert_eq!(card.global_rules.len(), 1);
    }

    #[test]
    fn test_world_manifest_serde() {
        let json = r#"{
            "id": "test",
            "name": "Test World",
            "description": "A test world.",
            "global_rules": ["Rule 1"],
            "starting_room_id": "start",
            "map_file": "map.json",
            "player_file": "player.json"
        }"#;

        let manifest: WorldManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.id, "test");
        assert_eq!(manifest.starting_room_id, "start");
        assert_eq!(manifest.map_file, "map.json");
        assert_eq!(manifest.player_file, "player.json");
    }

    #[test]
    fn test_world_manifest_defaults() {
        let json = r#"{
            "id": "test",
            "name": "Test World",
            "description": "A test world.",
            "global_rules": []
        }"#;

        let manifest: WorldManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.starting_room_id, "start");
        assert_eq!(manifest.map_file, "map.json");
        assert_eq!(manifest.player_file, "player.json");
    }

    #[test]
    fn test_world_manifest_to_card() {
        let manifest = WorldManifest {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test desc".to_string(),
            global_rules: vec!["rule1".to_string()],
            starting_room_id: "start".to_string(),
            map_file: "map.json".to_string(),
            player_file: "player.json".to_string(),
            scenarios: vec![],
            default_scenario_id: None,
        };

        let card: WorldCard = manifest.into();
        assert_eq!(card.name, "Test");
        assert_eq!(card.description, "Test desc");
        assert_eq!(card.global_rules.len(), 1);
    }
}
