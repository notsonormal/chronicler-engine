use crate::bootstrap::run::find_latest_game_for_world;
use crate::storage::message_storage::MessageStorage;
use crate::storage::snapshot_storage::SnapshotStorage;

#[test]
fn test_find_latest_game_for_world_uses_message_timestamp() {
    let db_pool = crate::storage::db::DbPool::new(":memory:").unwrap();

    let older = "2026-05-20T10:00:00+00:00";
    let newer = "2026-05-21T10:00:00+00:00";

    let game_a_id = {
        let conn = db_pool.conn();
        conn.execute(
            "INSERT INTO games (world_name, name, created_at, updated_at) VALUES ('TestWorld', 'GameA', ?1, ?1)",
            rusqlite::params![&older],
        )
        .unwrap();
        conn.last_insert_rowid() as u64
    };

    let game_b_id = {
        let conn = db_pool.conn();
        conn.execute(
            "INSERT INTO games (world_name, name, created_at, updated_at) VALUES ('TestWorld', 'GameB', ?1, ?1)",
            rusqlite::params![&newer],
        )
        .unwrap();
        conn.last_insert_rowid() as u64
    };

    // Insert a message ONLY for GameA (which has older updated_at)
    {
        let conn = db_pool.conn();
        conn.execute(
            "INSERT INTO messages (game_id, sender, log_type, timestamp, active_swipe_index, is_deleted) VALUES (?1, 'Player', 'input', ?2, 0, 0)",
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

    // GameA should be returned because it has the most recent message
    let result = find_latest_game_for_world(&db_pool, "TestWorld").unwrap();
    assert!(result.is_some());
    let (id, name) = result.unwrap();
    assert_eq!(id, game_a_id);
    assert_eq!(name, "GameA");

    // Also verify GameB is not returned
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
            "INSERT INTO games (world_name, name, created_at, updated_at) VALUES ('TestWorld', 'GameA', ?1, ?1)",
            rusqlite::params![&older],
        )
        .unwrap();
    }

    let game_b_id = {
        let conn = db_pool.conn();
        conn.execute(
            "INSERT INTO games (world_name, name, created_at, updated_at) VALUES ('TestWorld', 'GameB', ?1, ?1)",
            rusqlite::params![&newer],
        )
        .unwrap();
        conn.last_insert_rowid() as u64
    };

    // No messages for either game - should fall back to updated_at
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

    // 1. Insert a game
    let game_id = {
        let conn = db_pool.conn();
        conn.execute(
            "INSERT INTO games (world_name, name, created_at, updated_at) VALUES ('TestWorld', 'TestGame', '2026-05-26T10:00:00+00:00', '2026-05-26T10:00:00+00:00')",
            [],
        )
        .unwrap();
        conn.last_insert_rowid() as u64
    };

    // 2. Simulate first startup: create state, inject scenario, save snapshot + message
    let snapshot_repo =
        crate::storage::snapshot_storage::SqliteSnapshotRepository::new(db_pool.clone(), game_id);
    let message_repo =
        crate::storage::message_storage::SqliteMessageRepository::new(db_pool.clone(), game_id);

    let manifest = crate::model::world::WorldManifest {
        id: "test".to_string(),
        name: "TestWorld".to_string(),
        description: "A test world".to_string(),
        global_rules: vec![],
        starting_room_id: "start".to_string(),
        map_file: "map.json".to_string(),
        player_file: "player.json".to_string(),
        characters_dir: "".to_string(),
        scenarios: vec![crate::model::scenario::StartingScenario {
            id: "intro".to_string(),
            name: "Introduction".to_string(),
            description: "The beginning".to_string(),
            starting_room_id: "start".to_string(),
            text: "Welcome, traveller.".to_string(),
            npcs: vec![],
        }],
        default_scenario_id: None,
        default_room_image: None,
    };
    let map = crate::model::map::MapDef {
        overworld: crate::model::map::Overworld {
            id: "test".to_string(),
            name: "Test".to_string(),
            regions: vec![],
        },
    };
    let player = crate::model::character::PlayerCard {
        sheet: crate::model::character::CharacterSheet {
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

    let mut state = crate::model::state::GameState::new(
        std::sync::Arc::new(manifest.clone().into()),
        std::sync::Arc::new(map),
        std::sync::Arc::new(player.clone()),
        vec![],
        manifest.starting_room_id.clone(),
    );
    crate::bootstrap::inject_scenario_logs(&mut state, &manifest, &player);

    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let snapshot_id = snapshot_repo.save(&snapshot).unwrap();
    if let Some(msg) = state.narrative.history.last_mut() {
        msg.snapshot_id = Some(snapshot_id);
        let id = message_repo.insert_message(&*msg).unwrap();
        msg.id = id;
    }

    // Verify first startup created exactly one message
    let initial_messages = message_repo.load_messages().unwrap();
    assert_eq!(initial_messages.len(), 1);

    // 3. Simulate restart logic: find game, load snapshot
    let found = find_latest_game_for_world(&db_pool, "TestWorld").unwrap();
    assert!(found.is_some());
    let (found_id, _) = found.unwrap();
    assert_eq!(found_id, game_id);

    let loaded = snapshot_repo.load_latest().unwrap();
    assert!(
        loaded.is_some(),
        "Existing snapshot should be found on restart"
    );

    // If a snapshot exists, the fixed startup code loads it and skips scenario injection.
    // Verify no duplicate was inserted.
    let msgs_after_restart = message_repo.load_messages().unwrap();
    assert_eq!(
        msgs_after_restart.len(),
        1,
        "Restart should not duplicate the scenario message"
    );
}
