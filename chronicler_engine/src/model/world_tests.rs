use crate::model::world::{self, WorldCard, WorldManifest};

#[test]
fn derive_player_key_fallback() {
    assert_eq!(world::derive_player_key(""), "player");
    assert_eq!(
        world::derive_player_key("custom_player.json"),
        "custom_player"
    );
    assert_eq!(world::derive_player_key("player.json"), "player");
}

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
    assert_eq!(card.starting_room_id, "start");
}

#[test]
fn test_world_manifest_serde() {
    let json = r#"{
        "id": "test",
        "name": "Test World",
        "description": "A test world.",
        "global_rules": ["Rule 1"],
        "starting_room_id": "start",
        "map_file": "map.json",
        "player_file": "player.json"
    }"#;

    let manifest: WorldManifest = serde_json::from_str(json).unwrap();
    assert_eq!(manifest.id, "test");
    assert_eq!(manifest.starting_room_id, "start");
    assert_eq!(manifest.map_file, "map.json");
    assert_eq!(manifest.player_file, "player.json");
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
    assert_eq!(manifest.starting_room_id, "start");
    assert_eq!(manifest.map_file, "map.json");
    assert_eq!(manifest.player_file, "player.json");
}

#[test]
fn test_world_manifest_to_card() {
    let manifest = WorldManifest {
        id: "test".to_string(),
        name: "Test".to_string(),
        description: "Test desc".to_string(),
        global_rules: vec!["rule1".to_string()],
        starting_room_id: "start".to_string(),
        map_file: "map.json".to_string(),
        player_file: "player.json".to_string(),
        characters_dir: String::new(),
        scenarios: vec![],
        default_scenario_id: None,
        default_room_image: None,
    };

    let card: WorldCard = manifest.into();
    assert_eq!(card.name, "Test");
    assert_eq!(card.description, "Test desc");
    assert_eq!(card.global_rules.len(), 1);
    assert_eq!(card.starting_room_id, "start");
    assert_eq!(card.player_key, "player");
    assert!(card.default_scenario().is_none());
}
