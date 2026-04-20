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
    pub profile_image: Option<String>,
    #[serde(default)]
    pub headshot_image: Option<String>,
}

impl CharacterSheet {
    pub fn preferred_image(&self) -> Option<&str> {
        self.headshot_image
            .as_deref()
            .or(self.profile_image.as_deref())
    }
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
    #[serde(default)]
    pub triggers: Vec<crate::model::trigger::Trigger>,
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
            "profile_image": "data/images/carla.png"
        }"#;
        let npc: NpcCard = serde_json::from_str(json).unwrap();
        assert_eq!(npc.id, "carla");
        assert_eq!(npc.sheet.name, "Carla");
        assert_eq!(
            npc.sheet.profile_image,
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
        assert_eq!(player.sheet.profile_image, None);
    }

    #[test]
    fn test_npc_card_headshot_image() {
        let json = r#"{
            "id": "carla",
            "name": "Carla",
            "description": "Guard",
            "personality": "Strict",
            "scenario": "Guarding",
            "profile_image": "data/images/carla.png",
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
        let json = r#"{
            "id": "carla",
            "name": "Carla",
            "description": "Guard",
            "personality": "Strict",
            "scenario": "Guarding",
            "profile_image": "data/images/carla.png"
        }"#;
        let npc: NpcCard = serde_json::from_str(json).unwrap();
        assert_eq!(npc.sheet.headshot_image, None);
        assert_eq!(
            npc.sheet.profile_image,
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
            "profile_image": "data/images/julian_profile.png",
            "headshot_image": "data/images/julian_headshot.png"
        }"#;
        let player: PlayerCard = serde_json::from_str(json).unwrap();
        assert_eq!(
            player.sheet.headshot_image,
            Some("data/images/julian_headshot.png".to_string())
        );
    }

    #[test]
    fn test_preferred_image_headshot_first() {
        let sheet = CharacterSheet {
            name: "Test".into(),
            description: "Desc".into(),
            personality: "Personality".into(),
            scenario: "Scenario".into(),
            example_dialogue: "Dialogue".into(),
            profile_image: Some("profile.png".into()),
            headshot_image: Some("headshot.png".into()),
        };
        assert_eq!(sheet.preferred_image(), Some("headshot.png"));
    }

    #[test]
    fn test_preferred_image_fallback_to_profile() {
        let sheet = CharacterSheet {
            name: "Test".into(),
            description: "Desc".into(),
            personality: "Personality".into(),
            scenario: "Scenario".into(),
            example_dialogue: "Dialogue".into(),
            profile_image: Some("profile.png".into()),
            headshot_image: None,
        };
        assert_eq!(sheet.preferred_image(), Some("profile.png"));
    }

    #[test]
    fn test_preferred_image_none_when_both_absent() {
        let sheet = CharacterSheet {
            name: "Test".into(),
            description: "Desc".into(),
            personality: "Personality".into(),
            scenario: "Scenario".into(),
            example_dialogue: "Dialogue".into(),
            profile_image: None,
            headshot_image: None,
        };
        assert_eq!(sheet.preferred_image(), None);
    }
}
