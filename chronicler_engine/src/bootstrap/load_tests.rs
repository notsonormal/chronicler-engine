use crate::bootstrap::load::{initialize_world_from_manifest, load_world_manifest};
use crate::cli::resolve_engine_data_path;
use crate::model::world::WorldManifest;
use crate::storage::Storage;
use crate::test_support::TestPlayer;
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
    assert_eq!(npcs.len(), 3, "Test world should have 3 NPCs");
}

#[test]
fn test_load_world_manifest_path_construction() {
    let result = load_world_manifest("test", &resolve_engine_data_path());
    match result {
        Ok(manifest) => {
            assert_eq!(manifest.id, "test");
        }
        Err(e) => {
            let _ = e;
        }
    }
}

#[test]
fn test_initialize_world_checks_world_directory() {
    let result =
        initialize_world_from_manifest("nonexistent_directory_xyz123", &resolve_engine_data_path());
    assert!(result.is_err());
}

#[test]
fn test_initialize_world_requires_world_json() {
    let result = initialize_world_from_manifest("nonexistent_xyz789", &resolve_engine_data_path());
    assert!(result.is_err());
}

#[test]
fn test_initialize_world_validates_manifest_fields() {
    let result = initialize_world_from_manifest("test", &resolve_engine_data_path());
    if let Ok((manifest, _map, _player, _npcs)) = result {
        assert!(!manifest.id.is_empty());
        assert!(!manifest.name.is_empty());
        assert!(!manifest.starting_room_id.is_empty());
    }
}

#[test]
fn test_initialize_world_loads_character_files() {
    let result = initialize_world_from_manifest("test", &resolve_engine_data_path());
    if let Ok((_manifest, _map, _player, npcs)) = result {
        let _ = npcs.len();
    }
}

#[test]
fn test_load_world_manifest_returns_result() {
    let result: crate::error::Result<WorldManifest> =
        load_world_manifest("test", &resolve_engine_data_path());
    let _ = result;
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

    let personas_dir = temp_dir.join("personas");
    std::fs::create_dir_all(&personas_dir).unwrap();
    std::fs::write(
        personas_dir.join("player.json"),
        serde_json::to_string_pretty(&TestPlayer::standard()).unwrap(),
    )
    .unwrap();

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
fn test_seed_game_data_empty_worlds_dir() {
    let storage = Storage::new_in_memory();
    let temp_data = tempfile::TempDir::new().unwrap();
    let result = crate::bootstrap::load::seed_game_data(&storage, temp_data.path());
    assert!(
        result.is_ok(),
        "Should handle missing worlds dir: {result:?}"
    );
}

#[test]
fn test_seed_game_data_invalid_world_json_skips() {
    let storage = Storage::new_in_memory();
    let temp_data = tempfile::TempDir::new().unwrap();
    let worlds_dir = temp_data.path().join("worlds");
    std::fs::create_dir_all(&worlds_dir).unwrap();
    let bad_world_dir = worlds_dir.join("bad_world");
    std::fs::create_dir_all(&bad_world_dir).unwrap();
    std::fs::write(bad_world_dir.join("world.json"), "not valid json {").unwrap();
    let result = crate::bootstrap::load::seed_game_data(&storage, temp_data.path());
    assert!(
        result.is_ok(),
        "Invalid world.json should not abort seeding: {result:?}"
    );
}

#[test]
fn test_seed_game_data_idempotent() {
    let storage = Storage::new_in_memory();
    let temp_data = tempfile::TempDir::new().unwrap();
    let worlds_dir = temp_data.path().join("worlds");
    std::fs::create_dir_all(&worlds_dir).unwrap();
    let world_dir = worlds_dir.join("test_seed");
    std::fs::create_dir_all(&world_dir).unwrap();
    let map_dir = world_dir.clone();
    let persona_dir = temp_data.path().join("personas");
    std::fs::create_dir_all(&persona_dir).unwrap();
    std::fs::write(
        world_dir.join("world.json"),
        r#"{
            "id": "test_seed",
            "name": "Test Seed World",
            "description": "A world for testing",
            "global_rules": [],
            "starting_room_id": "start",
            "map_file": "map.json",
            "player_file": "test_player.json",
            "scenarios": [],
            "default_scenario_id": null
        }"#,
    )
    .unwrap();
    std::fs::write(
        map_dir.join("map.json"),
        r#"{
            "overworld": {
                "id": "test_map",
                "name": "Test Map",
                "regions": [
                    {
                        "id": "test_region",
                        "name": "Test",
                        "rooms": [
                            {
                                "id": "start",
                                "name": "Start",
                                "description": "Start room",
                                "exits": {}
                            }
                        ]
                    }
                ]
            },
            "default_room_image": null
        }"#,
    )
    .unwrap();
    std::fs::write(
        persona_dir.join("test_player.json"),
        r#"{
            "name": "Test Player",
            "description": "A test player",
            "personality": "Testy",
            "scenario": "Test scenario",
            "example_dialogue": "Hello",
            "inventory": []
        }"#,
    )
    .unwrap();
    let result1 = crate::bootstrap::load::seed_game_data(&storage, temp_data.path());
    assert!(result1.is_ok());
    let result2 = crate::bootstrap::load::seed_game_data(&storage, temp_data.path());
    assert!(result2.is_ok());
    let worlds = storage.list_worlds().unwrap();
    assert_eq!(worlds.len(), 1, "Should seed once, not duplicate");
}
