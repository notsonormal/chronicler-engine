use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StartingScenario {
    pub id: String,
    pub name: String,
    pub description: String,
    pub starting_room_id: String,
    #[serde(default)]
    pub text: String,
}
