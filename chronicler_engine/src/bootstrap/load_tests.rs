use crate::storage::Storage;

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
            "map_file": "map.json",
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

    // Personas seeded from data/personas/ scan (ADR-026).
    let personas = storage.list_personas().unwrap();
    assert_eq!(personas.len(), 1, "Should seed one persona");
    assert!(
        storage.get_persona("test_player").unwrap().is_some(),
        "Persona keyed by filename stem"
    );
}

#[test]
fn test_seed_game_data_scans_personas_dir() {
    let storage = Storage::new_in_memory();
    let temp_data = tempfile::TempDir::new().unwrap();
    let personas_dir = temp_data.path().join("personas");
    std::fs::create_dir_all(&personas_dir).unwrap();
    std::fs::write(
        personas_dir.join("foo.json"),
        r#"{
            "name": "Foo",
            "description": "A test persona",
            "personality": "Adventurous",
            "scenario": "Wandering",
            "example_dialogue": "Hi",
            "inventory": []
        }"#,
    )
    .unwrap();

    let result = crate::bootstrap::load::seed_game_data(&storage, temp_data.path());
    assert!(result.is_ok(), "seeding should succeed without worlds/");
    assert!(
        storage.get_persona("foo").unwrap().is_some(),
        "Persona should be seeded from data/personas/foo.json"
    );
}

#[test]
fn test_seed_game_data_no_personas_dir_ok() {
    let storage = Storage::new_in_memory();
    let temp_data = tempfile::TempDir::new().unwrap();
    let worlds_dir = temp_data.path().join("worlds");
    std::fs::create_dir_all(&worlds_dir).unwrap();

    let result = crate::bootstrap::load::seed_game_data(&storage, temp_data.path());
    assert!(
        result.is_ok(),
        "seeding should succeed with no personas/ dir"
    );
    assert!(
        storage.list_personas().unwrap().is_empty(),
        "No personas seeded when dir is absent"
    );
}
