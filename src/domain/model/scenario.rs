//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Scenario definitions and world data

use serde::{Deserialize, Serialize};

use crate::domain::model::utils::scenario_defaults;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StartingScenario {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default = "scenario_defaults::default_starting_room")]
    pub starting_room_id: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub npcs: Vec<String>,
}
