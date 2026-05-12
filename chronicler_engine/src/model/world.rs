use serde::{Deserialize, Serialize};

use crate::model::scenario::StartingScenario;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorldCard {
    pub name: String,
    pub description: String,
    pub global_rules: Vec<String>,
    #[serde(default = "default_starting_room")]
    pub starting_room_id: String,
    #[serde(default)]
    pub scenarios: Vec<StartingScenario>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_room_image: Option<String>,
}

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
    pub characters_dir: String,
    #[serde(default)]
    pub scenarios: Vec<StartingScenario>,
    #[serde(default)]
    pub default_scenario_id: Option<String>,
    #[serde(default)]
    pub default_room_image: Option<String>,
}

impl WorldManifest {
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

impl WorldCard {
    pub fn default_scenario(&self) -> Option<&StartingScenario> {
        self.scenarios.first()
    }
}

impl From<WorldManifest> for WorldCard {
    fn from(manifest: WorldManifest) -> Self {
        WorldCard {
            name: manifest.name,
            description: manifest.description,
            global_rules: manifest.global_rules,
            starting_room_id: manifest.starting_room_id,
            scenarios: manifest.scenarios,
            default_room_image: manifest.default_room_image,
        }
    }
}
