use crate::adapters::driven::storage::Storage;
use crate::bootstrap::load::seed_game_data;

#[test]
fn test_seed_game_data_empty_worlds_dir() {
    let storage = Storage::new_in_memory();
    let temp_data = tempfile::TempDir::new().unwrap();
    let result = seed_game_data(&storage, temp_data.path());
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
    let result = seed_game_data(&storage, temp_data.path());
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
    let result1 = seed_game_data(&storage, temp_data.path());
    assert!(result1.is_ok());
    let result2 = seed_game_data(&storage, temp_data.path());
    assert!(result2.is_ok());
    let worlds = storage.list_worlds().unwrap();
    assert_eq!(worlds.len(), 1, "Should seed once, not duplicate");

    // Personas seeded from data/personas/ scan.
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

    let result = seed_game_data(&storage, temp_data.path());
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

    let result = seed_game_data(&storage, temp_data.path());
    assert!(
        result.is_ok(),
        "seeding should succeed with no personas/ dir"
    );
    assert!(
        storage.list_personas().unwrap().is_empty(),
        "No personas seeded when dir is absent"
    );
}
#[test]
fn test_seed_settings_writes_the_file_into_an_empty_row() {
    use crate::bootstrap::load::seed_settings;
    use crate::domain::model::settings::AppSettings;

    let storage = Storage::new_in_memory();
    let temp_data = tempfile::TempDir::new().unwrap();
    let settings = AppSettings {
        response_length: "seeded from file".into(),
        ..AppSettings::default()
    };
    std::fs::write(
        temp_data.path().join("settings.json"),
        serde_json::to_string(&settings).unwrap(),
    )
    .unwrap();

    seed_settings(&storage, temp_data.path()).expect("seed should succeed");

    let loaded = storage.get_settings().expect("get settings");
    assert_eq!(loaded.response_length, "seeded from file");
}

#[test]
fn test_seed_settings_is_a_noop_without_a_file() {
    use crate::bootstrap::load::seed_settings;

    let storage = Storage::new_in_memory();
    let temp_data = tempfile::TempDir::new().unwrap();

    let result = seed_settings(&storage, temp_data.path());
    assert!(result.is_ok(), "a missing seed file is not an error");
}

#[test]
fn test_seed_settings_malformed_file_fails() {
    use crate::bootstrap::load::seed_settings;

    let storage = Storage::new_in_memory();
    let temp_data = tempfile::TempDir::new().unwrap();
    std::fs::write(temp_data.path().join("settings.json"), "{ not json").unwrap();

    let result = seed_settings(&storage, temp_data.path());
    assert!(
        result.is_err(),
        "a malformed seed file must fail rather than silently serve defaults"
    );
}

#[test]
fn test_seed_settings_dangling_narration_id_fails() {
    use crate::bootstrap::load::seed_settings;
    use crate::domain::model::settings::AppSettings;

    let storage = Storage::new_in_memory();
    let temp_data = tempfile::TempDir::new().unwrap();
    let settings = AppSettings {
        narration_connection_id: "not-a-connection".into(),
        ..AppSettings::default()
    };
    std::fs::write(
        temp_data.path().join("settings.json"),
        serde_json::to_string(&settings).unwrap(),
    )
    .unwrap();

    let err = seed_settings(&storage, temp_data.path())
        .expect_err("a dangling reference must be rejected before the write");
    assert!(
        err.to_string().contains("not-a-connection"),
        "error should name the missing id, got: {err}"
    );
}

#[test]
fn test_seed_settings_does_not_overwrite_an_edited_row() {
    use crate::bootstrap::load::seed_settings;
    use crate::domain::model::settings::AppSettings;

    let storage = Storage::new_in_memory();
    let temp_data = tempfile::TempDir::new().unwrap();

    let edited = AppSettings {
        response_length: "edited in the dashboard".into(),
        ..Default::default()
    };
    storage
        .save_settings(&edited)
        .expect("save edited settings");

    let from_file = AppSettings {
        response_length: "from data/settings.json".into(),
        ..Default::default()
    };
    std::fs::write(
        temp_data.path().join("settings.json"),
        serde_json::to_string(&from_file).unwrap(),
    )
    .unwrap();

    seed_settings(&storage, temp_data.path()).expect("seed should succeed");

    let loaded = storage.get_settings().expect("get settings");
    assert_eq!(
        loaded.response_length, "edited in the dashboard",
        "the seed must not clobber an edited row"
    );
}

#[test]
fn test_shipped_settings_file_seeds_the_settings_row() {
    use crate::bootstrap::load::seed_settings;

    let data_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    let storage = crate::test_support::sqlite_storage().expect("sqlite storage");

    seed_settings(&storage, &data_dir).expect("seeding the shipped file must succeed");

    let stored = storage.get_settings().expect("get settings");
    assert_eq!(
        stored.narration_connection_id, "deepseek-v4-flash",
        "a fresh database must serve the narrator the shipped file names"
    );
    stored
        .narration_connection()
        .expect("the seeded narrator connection id must resolve");
    stored
        .quantifier_connection()
        .expect("the seeded quantifier connection id must resolve");
    assert!(
        stored.agents.iter().any(|a| a.name == "options"),
        "the shipped settings must register the options agent, or the feature is silently off"
    );
}
