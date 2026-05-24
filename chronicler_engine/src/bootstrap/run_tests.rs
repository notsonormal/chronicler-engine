use crate::bootstrap::run::find_latest_game_for_world;

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
