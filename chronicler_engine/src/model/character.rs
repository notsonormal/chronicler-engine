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
    #[serde(default)]
    pub headshot_image: Option<String>,
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

    #[test]
    fn test_npc_card_headshot_image() {
        let json = r#"{
            "id": "carla",
            "name": "Carla",
            "description": "Guard",
            "personality": "Strict",
            "scenario": "Guarding",
            "image_path": "data/images/carla.png",
            "headshot_image": "data/images/carla_headshot.png"
        }"#;
        let npc: NpcCard = serde_json::from_str(json).unwrap();
        assert_eq!(
            npc.sheet.headshot_image,
            Some("data/images/carla_headshot.png".to_string())
        );
    }

    #[test]
    fn test_npc_card_headshot_image_none() {
        // When headshot_image is not present, it should be None
        let json = r#"{
            "id": "carla",
            "name": "Carla",
            "description": "Guard",
            "personality": "Strict",
            "scenario": "Guarding",
            "image_path": "data/images/carla.png"
        }"#;
        let npc: NpcCard = serde_json::from_str(json).unwrap();
        assert_eq!(npc.sheet.headshot_image, None);
        assert_eq!(
            npc.sheet.image_path,
            Some("data/images/carla.png".to_string())
        );
    }

    #[test]
    fn test_player_card_headshot_image() {
        let json = r#"{
            "name": "Julian",
            "description": "Heir",
            "personality": "Determined",
            "scenario": "Estate",
            "headshot_image": "data/images/julian_headshot.png"
        }"#;
        let player: PlayerCard = serde_json::from_str(json).unwrap();
        assert_eq!(
            player.sheet.headshot_image,
            Some("data/images/julian_headshot.png".to_string())
        );
    }
}
