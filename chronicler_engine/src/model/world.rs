use serde::{Deserialize, Serialize};

/// Represents the overarching rules and scenario for the game world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldCard {
    pub name: String,
    pub description: String,
    pub global_rules: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_card_serde() {
        let json = r#"{
            "name": "Test World",
            "description": "A test world.",
            "global_rules": ["Rule 1"]
        }"#;

        let card: WorldCard = serde_json::from_str(json).unwrap();
        assert_eq!(card.name, "Test World");
        assert_eq!(card.global_rules.len(), 1);
    }
}
