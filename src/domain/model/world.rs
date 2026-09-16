//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! World model definitions

use serde::{Deserialize, Serialize};

use crate::domain::model::scenario::StartingScenario;
use crate::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use crate::domain::model::utils::world_defaults;

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
    #[serde(default = "world_defaults::default_world_narrator_mode")]
    pub narrator_mode: NarratorMode,
    #[serde(default = "world_defaults::default_world_narrative_perspective")]
    pub narrative_perspective: NarrativePerspective,
    #[serde(default = "world_defaults::default_world_narrative_tense")]
    pub narrative_tense: NarrativeTense,
    /// Author default: generate pickable options after every narration turn.
    /// Games inherit this at creation (per-game override on `Game`).
    #[serde(default = "world_defaults::default_options_always_on")]
    pub options_always_on: bool,
}

/// Bootstrap manifest deserialized from `worlds/<id>/world.json`; file-pointer fields are used only during seeding in `bootstrap/load.rs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub global_rules: Vec<String>,
    #[serde(default = "world_defaults::default_map_file")]
    pub map_file: String,
    #[serde(default)]
    pub characters_dir: String,
    #[serde(default)]
    pub scenarios: Vec<StartingScenario>,
    #[serde(default)]
    pub default_scenario_id: Option<String>,
    #[serde(default)]
    pub default_room_image: Option<String>,
    #[serde(default = "world_defaults::default_world_narrator_mode")]
    pub narrator_mode: NarratorMode,
    #[serde(default = "world_defaults::default_world_narrative_perspective")]
    pub narrative_perspective: NarrativePerspective,
    #[serde(default = "world_defaults::default_world_narrative_tense")]
    pub narrative_tense: NarrativeTense,
    #[serde(default = "world_defaults::default_options_always_on")]
    pub options_always_on: bool,
}

impl WorldManifest {
    pub fn default_scenario(&self) -> Option<&StartingScenario> {
        self.scenarios.first()
    }
}

impl WorldCard {
    pub fn default_scenario(&self) -> Option<&StartingScenario> {
        self.scenarios.first()
    }

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
            narrator_mode: manifest.narrator_mode,
            narrative_perspective: manifest.narrative_perspective,
            narrative_tense: manifest.narrative_tense,
            options_always_on: manifest.options_always_on,
        }
    }
}
