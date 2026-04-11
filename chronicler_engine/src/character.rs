use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCard {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub inventory: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcCard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub personality: String,
    pub scenario: String,
    pub example_dialogue: String,
    #[serde(default)]
    pub inventory: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npc_card_serde() {
        let json = r#"{
            "id": "npc_1",
            "name": "Gary",
            "description": "Desc",
            "personality": "Angry",
            "scenario": "Tavern",
            "example_dialogue": "Hello."
        }"#;

        let npc: NpcCard = serde_json::from_str(json).unwrap();
        assert_eq!(npc.name, "Gary");
        assert_eq!(npc.inventory.len(), 0);
    }
}
