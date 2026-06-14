//! [DOC: docs/system/agent_system.md]
//! World model definitions

use serde::{Deserialize, Serialize};

use crate::model::scenario::StartingScenario;

/// Runtime world descriptor from DB. No filesystem concerns.
/// Used by `list_worlds()` and `get_world()` to return world data without file pointers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldCard {
    #[serde(default)]
    pub key: String,
    pub name: String,
    pub description: String,
    pub global_rules: Vec<String>,
    #[serde(default = "default_starting_room")]
    pub starting_room_id: String,
    #[serde(default)]
    pub scenarios: Vec<StartingScenario>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_scenario_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_room_image: Option<String>,
    #[serde(default)]
    pub player_key: String,
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

impl Default for WorldCard {
    fn default() -> Self {
        Self {
            key: String::default(),
            name: String::default(),
            description: String::default(),
            global_rules: Vec::default(),
            starting_room_id: default_starting_room(),
            scenarios: Vec::default(),
            default_scenario_id: None,
            default_room_image: None,
            player_key: "player".to_string(),
        }
    }
}

fn default_map_file() -> String {
    "map.json".to_string()
}

fn default_player_file() -> String {
    "player.json".to_string()
}

/// Derive the player key from a `player_file` path string.
/// Uses the file stem (without extension), falling back to `"player"`.
pub(crate) fn derive_player_key(player_file: &str) -> String {
    use std::path::Path;
    Path::new(player_file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("player")
        .to_string()
}

impl WorldCard {
    pub fn default_scenario(&self) -> Option<&StartingScenario> {
        self.scenarios.first()
    }
}

impl From<WorldManifest> for WorldCard {
    fn from(manifest: WorldManifest) -> Self {
        WorldCard {
            key: manifest.id.clone(),
            name: manifest.name,
            description: manifest.description,
            global_rules: manifest.global_rules,
            starting_room_id: manifest.starting_room_id,
            scenarios: manifest.scenarios,
            default_scenario_id: manifest.default_scenario_id,
            default_room_image: manifest.default_room_image,
            player_key: crate::model::world::derive_player_key(&manifest.player_file),
        }
    }
}
