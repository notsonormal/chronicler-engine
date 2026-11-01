use crate::domain::model::character::{CharacterSheet, CharacterCard, PersonaCard};
use crate::test_support::fixtures::TestCharacter;

fn card_with_id(id: &str) -> CharacterCard {
    TestCharacter::named(id, id)
}

#[test]
fn test_in_id_order_sorts_the_cards_by_id() {
    let cards = vec![card_with_id("c"), card_with_id("a"), card_with_id("b")];

    let ids: Vec<&str> = CharacterCard::in_id_order(&cards)
        .iter()
        .map(|card| card.id.as_str())
        .collect();

    assert_eq!(ids, vec!["a", "b", "c"]);
}

#[test]
fn test_in_id_order_keeps_the_input_order_of_equal_ids() {
    let cards = vec![card_with_id("a"), card_with_id("b"), card_with_id("a")];

    let sorted = CharacterCard::in_id_order(&cards);

    assert_eq!(sorted.len(), 3);
    assert!(
        std::ptr::eq(sorted[0], &cards[0]) && std::ptr::eq(sorted[1], &cards[2]),
        "the sort is stable, so two cards with one id keep their order"
    );
}

#[test]
fn test_character_card_serde() {
    let json = r#"{
        "id": "carla",
        "name": "Carla",
        "description": "Guard",
        "personality": "Strict",
        "scenario": "Guarding",
        "profile_image": "data/images/carla.png"
    }"#;
    let character: CharacterCard = serde_json::from_str(json).unwrap();
    assert_eq!(character.id, "carla");
    assert_eq!(character.sheet.name, "Carla");
    assert_eq!(
        character.sheet.profile_image,
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
    let player: PersonaCard = serde_json::from_str(json).unwrap();
    assert_eq!(player.sheet.name, "Julian");
    assert_eq!(player.inventory.len(), 1);
    assert_eq!(player.sheet.profile_image, None);
}

#[test]
fn test_character_card_headshot_image() {
    let json = r#"{
        "id": "carla",
        "name": "Carla",
        "description": "Guard",
        "personality": "Strict",
        "scenario": "Guarding",
        "profile_image": "data/images/carla.png",
        "headshot_image": "data/images/carla_headshot.png"
    }"#;
    let character: CharacterCard = serde_json::from_str(json).unwrap();
    assert_eq!(
        character.sheet.headshot_image,
        Some("data/images/carla_headshot.png".to_string())
    );
}

#[test]
fn test_character_card_headshot_image_none() {
    let json = r#"{
        "id": "carla",
        "name": "Carla",
        "description": "Guard",
        "personality": "Strict",
        "scenario": "Guarding",
        "profile_image": "data/images/carla.png"
    }"#;
    let character: CharacterCard = serde_json::from_str(json).unwrap();
    assert_eq!(character.sheet.headshot_image, None);
    assert_eq!(
        character.sheet.profile_image,
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
    let player: PersonaCard = serde_json::from_str(json).unwrap();
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
fn test_character_card_relationships_deserialize() {
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
    let character: CharacterCard = serde_json::from_str(json).unwrap();
    assert_eq!(character.relationships.len(), 1);
    assert_eq!(character.relationships[0].with, "gabriella");
    assert_eq!(
        character.relationships[0].static_text,
        "Carla distrusts Gabriella."
    );
    assert_eq!(character.relationships[0].dynamic, "open hostility");
}

#[test]
fn test_character_card_relationships_default_empty() {
    let json = r#"{
        "id": "carla",
        "name": "Carla",
        "description": "Guard",
        "personality": "Strict",
        "scenario": "Guarding"
    }"#;
    let character: CharacterCard = serde_json::from_str(json).unwrap();
    assert!(character.relationships.is_empty());
}
#[test]
fn test_relationship_display_text_uses_dynamic_when_present() {
    use crate::domain::model::character::Relationship;
    let rel = Relationship {
        with: "Character1".to_string(),
        static_text: "Static description".to_string(),
        dynamic: "Dynamic update".to_string(),
    };
    assert_eq!(rel.display_text(), "Dynamic update");
}
#[test]
fn test_relationship_display_text_fallback_to_static() {
    use crate::domain::model::character::Relationship;
    let rel = Relationship {
        with: "Character2".to_string(),
        static_text: "Static description".to_string(),
        dynamic: String::new(),
    };
    assert_eq!(rel.display_text(), "Static description");
}
#[test]
fn test_relationship_display_text_both_empty() {
    use crate::domain::model::character::Relationship;
    let rel = Relationship {
        with: "Character3".to_string(),
        static_text: String::new(),
        dynamic: String::new(),
    };
    assert_eq!(rel.display_text(), "");
}
