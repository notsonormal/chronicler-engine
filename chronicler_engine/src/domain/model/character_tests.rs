use crate::domain::model::character::{CharacterSheet, NpcCard, PlayerCard};

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
        summary: None,
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
        summary: None,
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
        summary: None,
        profile_image: None,
        headshot_image: None,
    };
    assert_eq!(sheet.preferred_image(), None);
}

#[test]
fn test_npc_card_relationships_deserialize() {
    let json = r#"{
        "id": "carla",
        "name": "Carla",
        "description": "Guard",
        "personality": "Strict",
        "scenario": "Guarding",
        "relationships": [
            {"with": "gabriella", "static": "Carla distrusts Gabriella.", "dynamic": "open hostility"}
        ]
    }"#;
    let npc: NpcCard = serde_json::from_str(json).unwrap();
    assert_eq!(npc.relationships.len(), 1);
    assert_eq!(npc.relationships[0].with, "gabriella");
    assert_eq!(
        npc.relationships[0].static_text,
        "Carla distrusts Gabriella."
    );
    assert_eq!(npc.relationships[0].dynamic, "open hostility");
}

#[test]
fn test_npc_card_relationships_default_empty() {
    let json = r#"{
        "id": "carla",
        "name": "Carla",
        "description": "Guard",
        "personality": "Strict",
        "scenario": "Guarding"
    }"#;
    let npc: NpcCard = serde_json::from_str(json).unwrap();
    assert!(npc.relationships.is_empty());
}
#[test]
fn test_relationship_display_text_uses_dynamic_when_present() {
    use crate::domain::model::character::Relationship;
    let rel = Relationship {
        with: "NPC1".to_string(),
        static_text: "Static description".to_string(),
        dynamic: "Dynamic update".to_string(),
    };
    assert_eq!(rel.display_text(), "Dynamic update");
}
#[test]
fn test_relationship_display_text_fallback_to_static() {
    use crate::domain::model::character::Relationship;
    let rel = Relationship {
        with: "NPC2".to_string(),
        static_text: "Static description".to_string(),
        dynamic: String::new(),
    };
    assert_eq!(rel.display_text(), "Static description");
}
#[test]
fn test_relationship_display_text_both_empty() {
    use crate::domain::model::character::Relationship;
    let rel = Relationship {
        with: "NPC3".to_string(),
        static_text: String::new(),
        dynamic: String::new(),
    };
    assert_eq!(rel.display_text(), "");
}
