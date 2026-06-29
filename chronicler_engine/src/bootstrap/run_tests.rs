use crate::bootstrap::init_game::resolve_game_id;
use crate::bootstrap::run::{ensure_presets, find_latest_game_for_world, list_game_names_for_world};
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::prompt_preset::PresetType;
use crate::storage::Storage;
#[test]
fn resolve_game_id_auto_creates_with_persona() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();

    let world = crate::domain::model::world::WorldCard {
        key: "redmist_estate".to_string(),
        name: "Redmist Estate".to_string(),
        description: String::new(),
        scenarios: vec![],
        ..Default::default()
    };

    let game_id = resolve_game_id(&db_pool, &world, "julian", "Julian").unwrap();
    assert!(game_id > 0);

    let (world_key, persona_key, persona_name): (String, String, String) = db_pool
        .conn()
        .query_row(
            "SELECT world_key, persona_key, persona_name FROM games WHERE id = ?1",
            rusqlite::params![game_id as i64],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(world_key, "redmist_estate");
    assert_eq!(persona_key, "julian");
    assert_eq!(persona_name, "Julian");

    let again = resolve_game_id(&db_pool, &world, "julian", "Julian").unwrap();
    assert_eq!(again, game_id);

    let count: i64 = db_pool
        .conn()
        .query_row("SELECT COUNT(*) FROM games", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1, "Second call should not create a duplicate game");
}
#[test]
fn test_find_latest_game_for_world_uses_message_timestamp() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();

    let older = "2026-05-20T10:00:00+00:00";
    let newer = "2026-05-21T10:00:00+00:00";

    let game_a_id = {
        let conn = db_pool.conn();
        conn.execute(
            "INSERT INTO games (world_name, world_key, name, created_at, updated_at) VALUES ('TestWorld', 'TestWorld', 'GameA', ?1, ?1)",
            rusqlite::params![&older],
        )
        .unwrap();
        conn.last_insert_rowid() as u64
    };

    let game_b_id = {
        let conn = db_pool.conn();
        conn.execute(
            "INSERT INTO games (world_name, world_key, name, created_at, updated_at) VALUES ('TestWorld', 'TestWorld', 'GameB', ?1, ?1)",
            rusqlite::params![&newer],
        )
        .unwrap();
        conn.last_insert_rowid() as u64
    };

    {
        let conn = db_pool.conn();
        conn.execute(
            "INSERT INTO messages (game_id, sender, message_type, timestamp, active_swipe_index, is_deleted) VALUES (?1, 'Player', 'Input', ?2, 0, 0)",
            rusqlite::params![game_a_id as i64, &newer],
        )
        .unwrap();
        let msg_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO message_swipes (message_id, swipe_index, text, snapshot_id, location_header, event_header) VALUES (?1, 0, 'hello', 0, NULL, NULL)",
            rusqlite::params![msg_id],
        )
        .unwrap();
    }

    let result = find_latest_game_for_world(&db_pool, "TestWorld").unwrap();
    assert!(result.is_some());
    let (id, name) = result.unwrap();
    assert_eq!(id, game_a_id);
    assert_eq!(name, "GameA");

    assert_ne!(id, game_b_id);
}

#[test]
fn test_find_latest_game_for_world_fallback_to_updated_at() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();

    let older = "2026-05-20T10:00:00+00:00";
    let newer = "2026-05-21T10:00:00+00:00";

    {
        let conn = db_pool.conn();
        conn.execute(
            "INSERT INTO games (world_name, world_key, name, created_at, updated_at) VALUES ('TestWorld', 'TestWorld', 'GameA', ?1, ?1)",
            rusqlite::params![&older],
        )
        .unwrap();
    }

    let game_b_id = {
        let conn = db_pool.conn();
        conn.execute(
            "INSERT INTO games (world_name, world_key, name, created_at, updated_at) VALUES ('TestWorld', 'TestWorld', 'GameB', ?1, ?1)",
            rusqlite::params![&newer],
        )
        .unwrap();
        conn.last_insert_rowid() as u64
    };

    let result = find_latest_game_for_world(&db_pool, "TestWorld").unwrap();
    assert!(result.is_some());
    let (id, _name) = result.unwrap();
    assert_eq!(id, game_b_id);
}

#[test]
fn test_find_latest_game_for_world_no_games() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();

    let result = find_latest_game_for_world(&db_pool, "NonExistent").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_restart_with_existing_game_does_not_duplicate_scenario() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();

    let game_id = {
        let conn = db_pool.conn();
        conn.execute(
            "INSERT INTO games (world_name, world_key, name, created_at, updated_at) VALUES ('TestWorld', 'TestWorld', 'TestGame', '2026-05-26T10:00:00+00:00', '2026-05-26T10:00:00+00:00')",
            [],
        )
        .unwrap();
        conn.last_insert_rowid() as u64
    };

    let storage = Storage::new_sqlite(db_pool.clone(), game_id);

    let world_card = crate::domain::model::world::WorldCard {
        key: "test".to_string(),
        name: "TestWorld".to_string(),
        description: "A test world".to_string(),
        scenarios: vec![crate::domain::model::scenario::StartingScenario {
            id: "intro".to_string(),
            name: "Introduction".to_string(),
            description: "The beginning".to_string(),
            starting_room_id: "start".to_string(),
            text: "Welcome, traveller.".to_string(),
            npcs: vec![],
        }],
        ..Default::default()
    };
    let map = crate::domain::model::map::MapDef {
        overworld: crate::domain::model::map::Overworld {
            id: "test".to_string(),
            name: "Test".to_string(),
            regions: vec![],
        },
    };
    let player = crate::domain::model::character::PlayerCard {
        key: "alice".to_string(),
        sheet: crate::domain::model::character::CharacterSheet {
            name: "Alice".to_string(),
            description: "".to_string(),
            personality: "".to_string(),
            scenario: "".to_string(),
            example_dialogue: "".to_string(),
            summary: None,
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
    };

    let mut state = GameState::new(
        std::sync::Arc::new(world_card.clone()),
        std::sync::Arc::new(map),
        std::sync::Arc::new(player.clone()),
        vec![],
        world_card.starting_room_id(),
    );
    crate::bootstrap::inject_scenario_logs(&mut state, &world_card, &player);

    let snapshot = crate::domain::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let snapshot_id = storage.save_snapshot(&snapshot).unwrap();
    if let Some(msg) = state.narrative.history.last_mut() {
        msg.set_snapshot_id(Some(snapshot_id));
        let id = storage.insert_message(&*msg).unwrap();
        msg.id = id;
    }

    let initial_messages = storage.load_message_rows().unwrap();
    assert_eq!(initial_messages.len(), 1);

    let found = find_latest_game_for_world(&db_pool, "TestWorld").unwrap();
    assert!(found.is_some());
    let (found_id, _) = found.unwrap();
    assert_eq!(found_id, game_id);

    let loaded = storage.load_latest_snapshot().unwrap();
    assert!(
        loaded.is_some(),
        "Existing snapshot should be found on restart"
    );

    let msgs_after_restart = storage.load_message_rows().unwrap();
    assert_eq!(
        msgs_after_restart.len(),
        1,
        "Restart should not duplicate the scenario message"
    );
}
#[test]
fn test_list_game_names_for_world_empty() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();
    let result = list_game_names_for_world(&db_pool, "NonExistent").unwrap();
    assert!(result.is_empty());
}
#[test]
fn test_list_game_names_for_world_single_game() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();
    {
        let conn = db_pool.conn();
        conn.execute(
            "INSERT INTO games (world_name, world_key, name, created_at, updated_at) VALUES ('TestWorld', 'TestWorld', 'Only Game', '2026-05-26T10:00:00+00:00', '2026-05-26T10:00:00+00:00')",
            [],
        )
        .unwrap();
    }
    let result = list_game_names_for_world(&db_pool, "TestWorld").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], "Only Game");
}
#[test]
fn test_list_game_names_for_world_multiple_games() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();
    {
        let conn = db_pool.conn();
        conn.execute(
            "INSERT INTO games (world_name, world_key, name, created_at, updated_at) VALUES ('TestWorld', 'TestWorld', 'Game Alpha', '2026-05-26T10:00:00+00:00', '2026-05-26T10:00:00+00:00')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO games (world_name, world_key, name, created_at, updated_at) VALUES ('TestWorld', 'TestWorld', 'Game Beta', '2026-05-26T10:00:00+00:00', '2026-05-26T10:00:00+00:00')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO games (world_name, world_key, name, created_at, updated_at) VALUES ('TestWorld', 'TestWorld', 'Game Gamma', '2026-05-26T10:00:00+00:00', '2026-05-26T10:00:00+00:00')",
            [],
        )
        .unwrap();
    }
    let result = list_game_names_for_world(&db_pool, "TestWorld").unwrap();
    assert_eq!(result.len(), 3);
    assert!(result.contains(&"Game Alpha".to_string()));
    assert!(result.contains(&"Game Beta".to_string()));
    assert!(result.contains(&"Game Gamma".to_string()));
}
#[test]
fn test_list_game_names_for_world_filters_by_world() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();
    {
        let conn = db_pool.conn();
        conn.execute(
            "INSERT INTO games (world_name, world_key, name, created_at, updated_at) VALUES ('WorldA', 'WorldA', 'A Game 1', '2026-05-26T10:00:00+00:00', '2026-05-26T10:00:00+00:00')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO games (world_name, world_key, name, created_at, updated_at) VALUES ('WorldA', 'WorldA', 'A Game 2', '2026-05-26T10:00:00+00:00', '2026-05-26T10:00:00+00:00')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO games (world_name, world_key, name, created_at, updated_at) VALUES ('WorldB', 'WorldB', 'B Game 1', '2026-05-26T10:00:00+00:00', '2026-05-26T10:00:00+00:00')",
            [],
        )
        .unwrap();
    }
    let result_a = list_game_names_for_world(&db_pool, "WorldA").unwrap();
    let result_b = list_game_names_for_world(&db_pool, "WorldB").unwrap();
    assert_eq!(result_a.len(), 2);
    assert_eq!(result_b.len(), 1);
    assert_eq!(result_b[0], "B Game 1");
}
#[test]
fn test_list_game_names_for_world_error_handling() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();
    let result = list_game_names_for_world(&db_pool, "TestWorld");
    assert!(result.is_err() || result.unwrap().is_empty());
}
#[test]
fn test_ensure_presets_empty_data_dir() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();
    let temp_data = tempfile::TempDir::new().unwrap();
    let result = ensure_presets(&db_pool, temp_data.path());
    assert!(result.is_ok());
}
#[test]
fn test_ensure_presets_creates_system_preset() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();
    let temp_data = tempfile::TempDir::new().unwrap();
    let system_dir = temp_data.path().join("prompt_presets").join("system");
    std::fs::create_dir_all(&system_dir).unwrap();
    let preset_content = serde_json::json!({
        "id": "test_system",
        "name": "Test System",
        "role": "You are a test narrator.",
        "instructions": "Test instructions",
        "is_default": true
    });
    std::fs::write(
        system_dir.join("test_system.json"),
        serde_json::to_string_pretty(&preset_content).unwrap(),
    )
    .unwrap();
    ensure_presets(&db_pool, temp_data.path()).unwrap();
    let storage = Storage::new_sqlite(db_pool, 1);
    let presets = storage.list_presets(PresetType::System).unwrap();
    assert_eq!(presets.len(), 1);
    assert_eq!(presets[0].id, "test_system");
    assert_eq!(presets[0].name, "Test System");
    assert_eq!(
        presets[0].role,
        Some("You are a test narrator.".to_string())
    );
}
#[test]
fn test_ensure_presets_creates_quantifier_preset() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();
    let temp_data = tempfile::TempDir::new().unwrap();
    let quantifier_dir = temp_data.path().join("prompt_presets").join("quantifier");
    std::fs::create_dir_all(&quantifier_dir).unwrap();
    let preset_content = serde_json::json!({
        "id": "test_quantifier",
        "name": "Test Quantifier",
        "role": "You are a scene quantifier.",
        "is_default": true
    });
    std::fs::write(
        quantifier_dir.join("test_quantifier.json"),
        serde_json::to_string_pretty(&preset_content).unwrap(),
    )
    .unwrap();
    ensure_presets(&db_pool, temp_data.path()).unwrap();
    let storage = Storage::new_sqlite(db_pool, 1);
    let presets = storage.list_presets(PresetType::Quantifier).unwrap();
    assert_eq!(presets.len(), 1);
    assert_eq!(presets[0].id, "test_quantifier");
}
#[test]
fn test_ensure_presets_skips_existing_preset_with_content() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();
    let temp_data = tempfile::TempDir::new().unwrap();
    let system_dir = temp_data.path().join("prompt_presets").join("system");
    std::fs::create_dir_all(&system_dir).unwrap();
    let preset_content = serde_json::json!({
        "id": "existing_preset",
        "name": "Should Not Change",
        "role": "New role from file",
        "is_default": true
    });
    std::fs::write(
        system_dir.join("existing_preset.json"),
        serde_json::to_string_pretty(&preset_content).unwrap(),
    )
    .unwrap();
    let storage = Storage::new_sqlite(db_pool.clone(), 1);
    let existing = crate::domain::model::prompt_preset::PromptPreset {
        id: "existing_preset".to_string(),
        name: "Original Name".to_string(),
        role: Some("Original role".to_string()),
        instructions: None,
        writing_style: None,
        output_format: None,
        is_default: true,
        preset_type: PresetType::System,
    };
    storage.save_preset(&existing).unwrap();
    ensure_presets(&db_pool, temp_data.path()).unwrap();
    let found = storage.get_preset("existing_preset").unwrap().unwrap();
    assert_eq!(found.name, "Original Name");
    assert_eq!(found.role, Some("Original role".to_string()));
}
#[test]
fn test_ensure_presets_updates_empty_preset() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();
    let temp_data = tempfile::TempDir::new().unwrap();
    let system_dir = temp_data.path().join("prompt_presets").join("system");
    std::fs::create_dir_all(&system_dir).unwrap();
    let preset_content = serde_json::json!({
        "id": "empty_preset",
        "name": "Updated Name",
        "role": "New role from file",
        "is_default": true
    });
    std::fs::write(
        system_dir.join("empty_preset.json"),
        serde_json::to_string_pretty(&preset_content).unwrap(),
    )
    .unwrap();
    let storage = Storage::new_sqlite(db_pool.clone(), 1);
    let empty = crate::domain::model::prompt_preset::PromptPreset {
        id: "empty_preset".to_string(),
        name: "Empty".to_string(),
        role: None,
        instructions: None,
        writing_style: None,
        output_format: None,
        is_default: true,
        preset_type: PresetType::System,
    };
    storage.save_preset(&empty).unwrap();
    ensure_presets(&db_pool, temp_data.path()).unwrap();
    let found = storage.get_preset("empty_preset").unwrap().unwrap();
    assert_eq!(found.name, "Updated Name");
    assert_eq!(found.role, Some("New role from file".to_string()));
}
#[test]
fn test_ensure_presets_ignores_non_json_files() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();
    let temp_data = tempfile::TempDir::new().unwrap();
    let system_dir = temp_data.path().join("prompt_presets").join("system");
    std::fs::create_dir_all(&system_dir).unwrap();
    std::fs::write(system_dir.join("readme.txt"), "This is not a preset").unwrap();
    std::fs::write(system_dir.join("data.yaml"), "key: value").unwrap();
    ensure_presets(&db_pool, temp_data.path()).unwrap();
    let storage = Storage::new_sqlite(db_pool, 1);
    let presets = storage.list_presets(PresetType::System).unwrap();
    assert!(presets.is_empty(), "Non-JSON files should be ignored");
}
#[test]
fn test_ensure_presets_handles_invalid_json() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();
    let temp_data = tempfile::TempDir::new().unwrap();
    let system_dir = temp_data.path().join("prompt_presets").join("system");
    std::fs::create_dir_all(&system_dir).unwrap();
    std::fs::write(system_dir.join("invalid.json"), "not valid json {").unwrap();
    let result = ensure_presets(&db_pool, temp_data.path());
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid preset seed")
    );
}
#[test]
fn test_ensure_presets_uses_default_id_for_missing_id() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();
    let temp_data = tempfile::TempDir::new().unwrap();
    let system_dir = temp_data.path().join("prompt_presets").join("system");
    std::fs::create_dir_all(&system_dir).unwrap();
    let preset_content = serde_json::json!({
        "name": "No ID Preset",
        "role": "Test role",
        "is_default": true
    });
    std::fs::write(
        system_dir.join("no_id.json"),
        serde_json::to_string_pretty(&preset_content).unwrap(),
    )
    .unwrap();
    ensure_presets(&db_pool, temp_data.path()).unwrap();
    let storage = Storage::new_sqlite(db_pool, 1);
    let presets = storage.list_presets(PresetType::System).unwrap();
    assert_eq!(presets.len(), 1);
    assert_eq!(
        presets[0].id, "default",
        "Should use 'default' when id is missing"
    );
}
#[test]
fn test_ensure_presets_all_fields_mapped() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();
    let temp_data = tempfile::TempDir::new().unwrap();
    let system_dir = temp_data.path().join("prompt_presets").join("system");
    std::fs::create_dir_all(&system_dir).unwrap();
    let preset_content = serde_json::json!({
        "id": "full_preset",
        "name": "Full Preset",
        "role": "Test role",
        "instructions": "Test instructions",
        "writing_style": "Test style",
        "output_format": "Test format",
        "is_default": true
    });
    std::fs::write(
        system_dir.join("full_preset.json"),
        serde_json::to_string_pretty(&preset_content).unwrap(),
    )
    .unwrap();
    ensure_presets(&db_pool, temp_data.path()).unwrap();
    let storage = Storage::new_sqlite(db_pool, 1);
    let found = storage.get_preset("full_preset").unwrap().unwrap();
    assert_eq!(found.name, "Full Preset");
    assert_eq!(found.role, Some("Test role".to_string()));
    assert_eq!(found.instructions, Some("Test instructions".to_string()));
    assert_eq!(found.writing_style, Some("Test style".to_string()));
    assert_eq!(found.output_format, Some("Test format".to_string()));
    assert!(found.is_default);
    assert_eq!(found.preset_type, PresetType::System);
}
#[test]
fn test_ensure_presets_both_types() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();
    let temp_data = tempfile::TempDir::new().unwrap();
    let system_dir = temp_data.path().join("prompt_presets").join("system");
    std::fs::create_dir_all(&system_dir).unwrap();
    std::fs::write(
        system_dir.join("sys.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "id": "system_test",
            "name": "System Test",
            "role": "System role"
        }))
        .unwrap(),
    )
    .unwrap();
    let quantifier_dir = temp_data.path().join("prompt_presets").join("quantifier");
    std::fs::create_dir_all(&quantifier_dir).unwrap();
    std::fs::write(
        quantifier_dir.join("quant.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "id": "quantifier_test",
            "name": "Quantifier Test",
            "role": "Quantifier role"
        }))
        .unwrap(),
    )
    .unwrap();
    ensure_presets(&db_pool, temp_data.path()).unwrap();
    let storage = Storage::new_sqlite(db_pool, 1);
    let system = storage.list_presets(PresetType::System).unwrap();
    let quantifier = storage.list_presets(PresetType::Quantifier).unwrap();
    assert_eq!(system.len(), 1);
    assert_eq!(system[0].id, "system_test");
    assert_eq!(quantifier.len(), 1);
    assert_eq!(quantifier[0].id, "quantifier_test");
}
#[test]
fn test_ensure_presets_idempotent() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();
    let temp_data = tempfile::TempDir::new().unwrap();
    let system_dir = temp_data.path().join("prompt_presets").join("system");
    std::fs::create_dir_all(&system_dir).unwrap();
    std::fs::write(
        system_dir.join("idempotent.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "id": "idempotent_test",
            "name": "Idempotent Test",
            "role": "Test role"
        }))
        .unwrap(),
    )
    .unwrap();
    ensure_presets(&db_pool, temp_data.path()).unwrap();
    ensure_presets(&db_pool, temp_data.path()).unwrap();
    let storage = Storage::new_sqlite(db_pool, 1);
    let presets = storage.list_presets(PresetType::System).unwrap();
    assert_eq!(presets.len(), 1);
    assert_eq!(presets[0].id, "idempotent_test");
}
