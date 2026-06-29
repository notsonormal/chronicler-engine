use crate::domain::model::scenario::StartingScenario;
use crate::domain::model::world::{WorldCard, WorldManifest};

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
    assert!(card.default_scenario().is_none());
}

#[test]
fn test_world_manifest_serde() {
    let json = r#"{
        "id": "test",
        "name": "Test World",
        "description": "A test world.",
        "global_rules": ["Rule 1"],
        "map_file": "map.json"
    }"#;

    let manifest: WorldManifest = serde_json::from_str(json).unwrap();
    assert_eq!(manifest.id, "test");
    assert_eq!(manifest.map_file, "map.json");
    assert!(manifest.default_scenario().is_none());
}

#[test]
fn test_world_manifest_defaults() {
    let json = r#"{
        "id": "test",
        "name": "Test World",
        "description": "A test world.",
        "global_rules": []
    }"#;

    let manifest: WorldManifest = serde_json::from_str(json).unwrap();
    assert_eq!(manifest.map_file, "map.json");
}

#[test]
fn test_world_manifest_to_card() {
    let manifest = WorldManifest {
        id: "test".to_string(),
        name: "Test".to_string(),
        description: "Test desc".to_string(),
        global_rules: vec!["rule1".to_string()],
        map_file: "map.json".to_string(),
        characters_dir: String::new(),
        scenarios: vec![],
        default_scenario_id: None,
        default_room_image: None,
    };

    let card: WorldCard = manifest.into();
    assert_eq!(card.name, "Test");
    assert_eq!(card.description, "Test desc");
    assert_eq!(card.global_rules.len(), 1);
    assert!(card.default_scenario().is_none());
}

#[test]
fn test_starting_scenario_serde_roundtrip() {
    let scenario = StartingScenario {
        id: "intro".to_string(),
        name: "Intro".to_string(),
        description: "Intro scenario".to_string(),
        starting_room_id: "front_gates".to_string(),
        text: "".to_string(),
        npcs: vec![],
    };

    let json = serde_json::to_string(&scenario).unwrap();
    let deserialized: StartingScenario = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.starting_room_id, "front_gates");
}

#[test]
fn test_world_card_deserializes_missing_top_level_starting_room_id() {
    let json = r#"{
        "key": "test",
        "name": "Test World",
        "description": "A test world.",
        "global_rules": ["Rule 1"],
        "starting_room_id": "start"
    }"#;

    let card: WorldCard = serde_json::from_str(json).unwrap();
    assert_eq!(card.name, "Test World");
}

#[test]
fn test_world_manifest_deserializes_missing_top_level_starting_room_id() {
    let json = r#"{
        "id": "test",
        "name": "Test World",
        "description": "A test world.",
        "global_rules": ["Rule 1"],
        "starting_room_id": "start"
    }"#;

    let manifest: WorldManifest = serde_json::from_str(json).unwrap();
    assert_eq!(manifest.id, "test");
}
