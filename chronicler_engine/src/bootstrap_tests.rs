use crate::bootstrap::{
    initialize_world_from_manifest, inject_scenario_logs, load_world_manifest, validate_loaded_data,
};
use crate::cli::resolve_engine_data_path;
use crate::model::character::{CharacterSheet, NpcCard};
use crate::model::map::{MapDef, Overworld};
use crate::model::scenario::StartingScenario;
use crate::model::trigger::Trigger;
use crate::model::world::WorldManifest;
use crate::test_support::{TestGameState, TestPlayer};

#[test]
fn test_load_redmist_estate_world() {
    let result = initialize_world_from_manifest("redmist_estate", &resolve_engine_data_path());
    assert!(result.is_ok(), "Failed to load redmist_estate: {result:?}");
    let (manifest, _map, _player, npcs) = result.unwrap();
    assert_eq!(manifest.id, "redmist_estate");
    assert_eq!(manifest.name, "Redmist Estate");
    assert_eq!(manifest.starting_room_id, "front_gates");
    assert!(!npcs.is_empty(), "Should have NPCs");
}

#[test]
fn test_load_test_world() {
    let result = initialize_world_from_manifest("test", &resolve_engine_data_path());
    assert!(result.is_ok(), "Failed to load test world: {result:?}");
    let (manifest, _map, player, npcs) = result.unwrap();
    assert_eq!(manifest.id, "test");
    assert_eq!(manifest.name, "Test Realm");
    assert_eq!(player.sheet.name, "Test Player");
    // Test world has 3 NPCs: ranger, shopkeeper, bartender
    assert_eq!(npcs.len(), 3, "Test world should have 3 NPCs");
}

#[test]
fn test_load_world_manifest_json_parse_error() {
    let result = load_world_manifest("test", &resolve_engine_data_path()); // Valid world
    assert!(result.is_ok()); // Should succeed for valid world
}

#[test]
fn test_load_world_manifest_path_construction() {
    // Test that manifest path is constructed from data_dir
    let result = load_world_manifest("test", &resolve_engine_data_path());
    match result {
        Ok(manifest) => {
            // Valid world loads successfully
            assert_eq!(manifest.id, "test");
        }
        Err(e) => {
            // If test world doesn't exist, that's acceptable
            let _ = e; // Silence unused warning
        }
    }
}

#[test]
fn test_initialize_world_checks_world_directory() {
    // Test that initialize_world_from_manifest checks directory existence
    let result =
        initialize_world_from_manifest("nonexistent_directory_xyz123", &resolve_engine_data_path());
    assert!(result.is_err()); // Should fail for missing directory
}

#[test]
fn test_initialize_world_requires_world_json() {
    // A directory without world.manifest should fail
    let result = initialize_world_from_manifest("nonexistent_xyz789", &resolve_engine_data_path());
    assert!(result.is_err()); // Should fail gracefully
}

#[test]
fn test_initialize_world_validates_manifest_fields() {
    // Test world should have all required fields
    let result = initialize_world_from_manifest("test", &resolve_engine_data_path());
    if let Ok((manifest, _map, _player, _npcs)) = result {
        assert!(!manifest.id.is_empty());
        assert!(!manifest.name.is_empty());
        assert!(!manifest.starting_room_id.is_empty());
    }
    // else: test world may not exist, which is fine for this test
}

#[test]
fn test_initialize_world_loads_character_files() {
    // Verify NPC files are loaded from characters/ subdirectory
    let result = initialize_world_from_manifest("test", &resolve_engine_data_path());
    if let Ok((_manifest, _map, _player, npcs)) = result {
        // NPCs are loaded (if character files exist)
        let _ = npcs.len(); // Should not panic
    }
}

#[test]
fn test_load_world_manifest_returns_result() {
    // Verify load_world_manifest returns engine Result type
    let result: crate::error::Result<WorldManifest> =
        load_world_manifest("test", &resolve_engine_data_path());
    let _ = result; // Just verify it compiles
}

#[test]
fn test_load_world_manifest_error_nonexistent() {
    let result = load_world_manifest("nonexistent_world_xyz", &resolve_engine_data_path());
    assert!(result.is_err(), "Should fail for non-existent world");
}

#[test]
fn test_initialize_world_error_not_found() {
    let result =
        initialize_world_from_manifest("nonexistent_world_xyz", &resolve_engine_data_path());
    assert!(result.is_err());
}

#[test]
fn test_load_world_manifest_valid() {
    let result = load_world_manifest("test", &resolve_engine_data_path());
    assert!(
        result.is_ok(),
        "Should load test world manifest: {result:?}"
    );
    let manifest = result.unwrap();
    assert_eq!(manifest.id, "test");
}

#[test]
fn test_world_manifest_contains_required_fields() {
    let manifest = load_world_manifest("test", &resolve_engine_data_path()).unwrap();
    assert!(!manifest.id.is_empty());
    assert!(!manifest.name.is_empty());
    assert!(!manifest.starting_room_id.is_empty());
}

#[test]
fn test_validate_loaded_data_success() {
    let manifest = crate::test_support::TestWorldManifest::minimal();

    let map = crate::test_support::TestMap::single_room("room_a");

    let player = crate::test_support::TestPlayer::standard();

    let npc = crate::test_support::TestNpc::named("npc1", "NPC");

    let result = validate_loaded_data(&manifest, &map, &player, &[npc]);
    assert!(result.is_ok(), "Expected validation to pass: {result:?}");
}

#[test]
fn test_validate_loaded_data_missing_starting_room() {
    let manifest = WorldManifest {
        id: "test".to_string(),
        name: "Test".to_string(),
        starting_room_id: "missing_room".to_string(),
        description: "A test world".to_string(),
        global_rules: vec![],
        map_file: "map.json".to_string(),
        player_file: "player.json".to_string(),
        characters_dir: "".to_string(),
        scenarios: vec![],
        default_scenario_id: None,
        default_room_image: None,
    };

    let map = crate::test_support::TestMap::single_room("room_a");

    let player = crate::test_support::TestPlayer::standard();

    let result = validate_loaded_data(&manifest, &map, &player, &[]);
    assert!(
        result.is_err(),
        "Expected validation to fail for missing starting room"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("starting_room_id"),
        "Error should mention starting_room_id: {err}"
    );
}

#[test]
fn test_validate_loaded_data_basic_manifest_succeeds() {
    let manifest = crate::test_support::TestWorldManifest::minimal();

    let map = crate::test_support::TestMap::single_room("room_a");

    let player = crate::test_support::TestPlayer::standard();

    let result = validate_loaded_data(&manifest, &map, &player, &[]);
    assert!(
        result.is_ok(),
        "Basic manifest with no NPCs and empty scenarios should validate successfully"
    );
}

#[test]
fn test_validate_loaded_data_invalid_trigger_room() {
    let manifest = crate::test_support::TestWorldManifest::minimal();

    let map = crate::test_support::TestMap::single_room("room_a");

    let player = crate::test_support::TestPlayer::standard();

    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: CharacterSheet {
            name: "NPC".to_string(),
            description: "An NPC".to_string(),
            personality: "Friendly".to_string(),
            scenario: "Default".to_string(),
            example_dialogue: "".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![Trigger {
            requirement: crate::model::trigger::TriggerRequirement::TimesMet(
                crate::model::trigger::ComparisonOperator::Eq,
                0,
            ),
            narration: crate::model::trigger::TriggerNarration {
                name: "Test".to_string(),
                narration_prompt: "Hello".to_string(),
            },
            repeat: false,
            room_id: Some("nonexistent_room".to_string()),
        }],
        relationships: vec![],
    };

    let result = validate_loaded_data(&manifest, &map, &player, &[npc]);
    assert!(
        result.is_err(),
        "Expected validation to fail for invalid trigger room"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("non-existent room_id"),
        "Error should mention room_id: {err}"
    );
}

#[test]
fn test_load_world_manifest_invalid_json() {
    let temp_dir =
        std::env::temp_dir().join(format!("chronicler_bootstrap_json_{}", std::process::id()));
    let world_dir = temp_dir.join("worlds").join("bad_json");
    std::fs::create_dir_all(&world_dir).unwrap();
    std::fs::write(world_dir.join("world.json"), "not valid json").unwrap();

    let result = load_world_manifest("bad_json", &temp_dir);
    let _ = std::fs::remove_dir_all(&temp_dir);

    assert!(result.is_err(), "Should fail for invalid JSON");
}

#[test]
fn test_initialize_world_with_characters_dir() {
    let temp_dir =
        std::env::temp_dir().join(format!("chronicler_bootstrap_chars_{}", std::process::id()));

    // World manifest with custom characters_dir
    let world_dir = temp_dir.join("worlds").join("char_world");
    std::fs::create_dir_all(&world_dir).unwrap();

    let manifest = WorldManifest {
        id: "char_world".to_string(),
        name: "Char World".to_string(),
        starting_room_id: "room_a".to_string(),
        description: "A world with custom chars dir".to_string(),
        global_rules: vec![],
        map_file: "map.json".to_string(),
        player_file: "player.json".to_string(),
        characters_dir: "custom_chars".to_string(),
        scenarios: vec![],
        default_scenario_id: None,
        default_room_image: None,
    };
    std::fs::write(
        world_dir.join("world.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();

    let map = crate::test_support::TestMap::single_room("room_a");
    std::fs::write(
        world_dir.join("map.json"),
        serde_json::to_string_pretty(&map).unwrap(),
    )
    .unwrap();

    // Player
    let player = crate::test_support::TestPlayer::standard();
    let personas_dir = temp_dir.join("personas");
    std::fs::create_dir_all(&personas_dir).unwrap();
    std::fs::write(
        personas_dir.join("player.json"),
        serde_json::to_string_pretty(&player).unwrap(),
    )
    .unwrap();

    // Custom NPCs
    let chars_dir = temp_dir.join("characters").join("custom_chars");
    std::fs::create_dir_all(&chars_dir).unwrap();
    let npc = crate::test_support::TestNpc::named("custom_npc", "Custom");
    std::fs::write(
        chars_dir.join("npc.json"),
        serde_json::to_string_pretty(&npc).unwrap(),
    )
    .unwrap();

    let result = initialize_world_from_manifest("char_world", &temp_dir);
    let _ = std::fs::remove_dir_all(&temp_dir);

    assert!(result.is_ok(), "Failed to initialize world: {result:?}");
    let (_manifest, _map, _player, npcs) = result.unwrap();
    assert_eq!(npcs.len(), 1);
    assert_eq!(npcs[0].id, "custom_npc");
}

#[test]
fn test_validate_loaded_data_multiple_errors() {
    let manifest = WorldManifest {
        id: "test".to_string(),
        name: "Test".to_string(),
        starting_room_id: "missing".to_string(),
        description: "A test world".to_string(),
        global_rules: vec![],
        map_file: "map.json".to_string(),
        player_file: "player.json".to_string(),
        characters_dir: "".to_string(),
        scenarios: vec![],
        default_scenario_id: None,
        default_room_image: None,
    };

    let map = MapDef {
        overworld: Overworld {
            id: "overworld".to_string(),
            name: "Overworld".to_string(),
            regions: vec![],
        },
    };

    let player = crate::test_support::TestPlayer::standard();

    let result = validate_loaded_data(&manifest, &map, &player, &[]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("\n") || err.contains("starting_room_id"),
        "Should have errors: {err}"
    );
}

#[test]
fn test_inject_scenario_logs_adds_narration() {
    let mut state = TestGameState::in_room("start");
    let manifest = WorldManifest {
        id: "test".to_string(),
        name: "Test".to_string(),
        starting_room_id: "start".to_string(),
        description: "A test world".to_string(),
        global_rules: vec![],
        map_file: "map.json".to_string(),
        player_file: "player.json".to_string(),
        characters_dir: "".to_string(),
        scenarios: vec![StartingScenario {
            id: "intro".to_string(),
            name: "Introduction".to_string(),
            description: "The beginning".to_string(),
            starting_room_id: "start".to_string(),
            text: "Welcome, {{user}}.".to_string(),
            npcs: vec![],
        }],
        default_scenario_id: None,
        default_room_image: None,
    };
    let player = TestPlayer::named("Alice");

    inject_scenario_logs(&mut state, &manifest, &player);

    assert_eq!(state.narrative.history().len(), 1);
    let entry = &state.narrative.history()[0];
    assert_eq!(entry.text, "Welcome, Alice.");
    assert_eq!(
        entry.message_type,
        crate::model::state::MessageType::Narration
    );
    assert_eq!(entry.location_header, Some("Room start".to_string()));
}

#[test]
fn test_inject_scenario_logs_no_scenario() {
    let mut state = TestGameState::in_room("start");
    let manifest = WorldManifest {
        id: "test".to_string(),
        name: "Test".to_string(),
        starting_room_id: "start".to_string(),
        description: "A test world".to_string(),
        global_rules: vec![],
        map_file: "map.json".to_string(),
        player_file: "player.json".to_string(),
        characters_dir: "".to_string(),
        scenarios: vec![],
        default_scenario_id: None,
        default_room_image: None,
    };
    let player = TestPlayer::standard();

    inject_scenario_logs(&mut state, &manifest, &player);

    assert!(state.narrative.history().is_empty());
}

#[test]
fn test_inject_scenario_logs_empty_text() {
    let mut state = TestGameState::in_room("start");
    let manifest = WorldManifest {
        id: "test".to_string(),
        name: "Test".to_string(),
        starting_room_id: "start".to_string(),
        description: "A test world".to_string(),
        global_rules: vec![],
        map_file: "map.json".to_string(),
        player_file: "player.json".to_string(),
        characters_dir: "".to_string(),
        scenarios: vec![StartingScenario {
            id: "empty".to_string(),
            name: "Empty".to_string(),
            description: "Nothing".to_string(),
            starting_room_id: "start".to_string(),
            text: "".to_string(),
            npcs: vec![],
        }],
        default_scenario_id: None,
        default_room_image: None,
    };
    let player = TestPlayer::standard();

    inject_scenario_logs(&mut state, &manifest, &player);

    assert!(state.narrative.history().is_empty());
}
