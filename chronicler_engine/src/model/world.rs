//! [DOC: docs/system/agent_system.md]
//! World model definitions

use serde::{Deserialize, Serialize};

use crate::model::scenario::StartingScenario;

/// Runtime world descriptor sourced from the DB. No filesystem pointers.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorldCard {
    #[serde(default)]
    pub key: String,
    pub name: String,
    pub description: String,
    pub global_rules: Vec<String>,
    #[serde(default)]
    pub scenarios: Vec<StartingScenario>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_scenario_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_room_image: Option<String>,
}

/// Bootstrap manifest deserialized from `worlds/<id>/world.json`.
/// Contains file pointer fields (`map_file`, `characters_dir`) used ONLY during
/// initial seeding in `bootstrap/load.rs`. Runtime data lives in the DB afterwards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub global_rules: Vec<String>,
    #[serde(default = "default_map_file")]
    pub map_file: String,
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

fn default_map_file() -> String {
    "map.json".to_string()
}

impl WorldCard {
    pub fn default_scenario(&self) -> Option<&StartingScenario> {
        self.scenarios.first()
    }

    /// Resolve starting room ID from default scenario, falling back to "start".
    pub fn starting_room_id(&self) -> String {
        self.default_scenario()
            .map(|s| s.starting_room_id.clone())
            .unwrap_or_else(|| "start".to_string())
    }
}

impl From<WorldManifest> for WorldCard {
    fn from(manifest: WorldManifest) -> Self {
        WorldCard {
            key: manifest.id.clone(),
            name: manifest.name,
            description: manifest.description,
            global_rules: manifest.global_rules,
            scenarios: manifest.scenarios,
            default_scenario_id: manifest.default_scenario_id,
            default_room_image: manifest.default_room_image,
        }
    }
}
