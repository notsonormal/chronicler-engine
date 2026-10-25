//! [DOC: docs/diataxis/reference/storage.md]
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
    /// The Characters present at the start. The World files key them `characters`; older ones use `npcs`.
    #[serde(default, rename = "characters", alias = "npcs")]
    pub characters_present: Vec<String>,
}
