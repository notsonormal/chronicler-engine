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
