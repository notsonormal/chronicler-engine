use serde::{Deserialize, Serialize};

use crate::model::scenario::StartingScenario;

/// Runtime world descriptor from DB. No filesystem concerns.
/// Used by `list_worlds()` and `get_world()` to return world data without file pointers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldInfo {
    pub key: String,
    pub name: String,
    pub description: String,
    pub global_rules: Vec<String>,
    pub starting_room_id: String,
    pub scenarios: Vec<StartingScenario>,
    #[serde(default)]
    pub default_scenario_id: Option<String>,
    #[serde(default)]
    pub default_room_image: Option<String>,
}

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

/// Filesystem bootstrap descriptor. Deserialized from `worlds/<id>/world.json`.
/// Contains file pointer fields (`map_file`, `player_file`, `characters_dir`) used ONLY
/// during initial seeding in `bootstrap/load.rs::initialize_world_from_manifest()`.
/// After seeding, these fields are meaningless — runtime data lives in the DB.
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
