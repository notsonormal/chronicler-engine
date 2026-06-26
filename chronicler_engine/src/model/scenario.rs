//! [DOC: docs/system/agent_system.md]
//! Scenario definitions and world data

use serde::{Deserialize, Serialize};

fn default_starting_room() -> String {
    "start".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StartingScenario {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default = "default_starting_room")]
    pub starting_room_id: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub npcs: Vec<String>,
}
