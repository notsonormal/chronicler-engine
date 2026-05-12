use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StartingScenario {
    pub id: String,
    pub name: String,
    pub description: String,
    pub starting_room_id: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub npcs: Vec<String>,
}
