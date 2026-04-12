use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CharacterSheet {
    pub name: String,
    pub description: String,
    pub personality: String,
    pub scenario: String,
    #[serde(default)]
    pub example_dialogue: String,
    #[serde(default)]
    pub image_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerCard {
    #[serde(flatten)]
    pub sheet: CharacterSheet,
    #[serde(default)]
    pub inventory: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NpcCard {
    pub id: String,
    #[serde(flatten)]
    pub sheet: CharacterSheet,
    #[serde(default)]
    pub inventory: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npc_card_serde() {
        let json = r#"{
            "id": "carla",
            "name": "Carla",
            "description": "Guard",
            "personality": "Strict",
            "scenario": "Guarding",
            "image_path": "data/images/carla.png"
        }"#;
        let npc: NpcCard = serde_json::from_str(json).unwrap();
        assert_eq!(npc.id, "carla");
        assert_eq!(npc.sheet.name, "Carla");
        assert_eq!(
            npc.sheet.image_path,
            Some("data/images/carla.png".to_string())
        );
    }

    #[test]
    fn test_player_card_serde() {
        let json = r#"{
            "name": "Julian",
            "description": "Heir",
            "personality": "Determined",
            "scenario": "Estate",
            "inventory": ["key"]
        }"#;
        let player: PlayerCard = serde_json::from_str(json).unwrap();
        assert_eq!(player.sheet.name, "Julian");
        assert_eq!(player.inventory.len(), 1);
        assert_eq!(player.sheet.image_path, None);
    }
}
