use crate::storage::db::DbPool;

#[test]
fn test_db_creates_games_table() {
    let pool = DbPool::new(":memory:").unwrap();
    let conn = pool.conn();
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='games'")
        .unwrap();
    let exists: Result<String, _> = stmt.query_row([], |row| row.get(0));
    assert!(exists.is_ok(), "games table should exist");
}

#[test]
fn test_db_inserts_default_game() {
    let pool = DbPool::new(":memory:").unwrap();
    let conn = pool.conn();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM games", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1, "default game should be inserted");
}

#[test]
fn test_db_game_state_snapshots_has_game_id() {
    let pool = DbPool::new(":memory:").unwrap();
    let conn = pool.conn();
    let mut stmt = conn
        .prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='game_state_snapshots'")
        .unwrap();
    let schema: String = stmt.query_row([], |row| row.get(0)).unwrap();
    assert!(
        schema.contains("game_id"),
        "game_state_snapshots should have game_id column"
    );
}

#[test]
fn test_db_messages_has_game_id() {
    let pool = DbPool::new(":memory:").unwrap();
    let conn = pool.conn();
    let mut stmt = conn
        .prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='messages'")
        .unwrap();
    let schema: String = stmt.query_row([], |row| row.get(0)).unwrap();
    assert!(
        schema.contains("game_id"),
        "messages should have game_id column"
    );
}

#[test]
fn test_db_new_invalid_path() {
    let result = DbPool::new("/nonexistent/path/to/db.sqlite");
    assert!(result.is_err());
}

#[test]
fn test_db_llm_messages_table_exists() {
    let pool = DbPool::new(":memory:").unwrap();
    let conn = pool.conn();
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='llm_messages'")
        .unwrap();
    let exists: Result<String, _> = stmt.query_row([], |row| row.get(0));
    assert!(exists.is_ok(), "llm_messages table should exist");
}

#[test]
fn test_db_prompt_presets_table_exists() {
    let pool = DbPool::new(":memory:").unwrap();
    let conn = pool.conn();
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='prompt_presets'")
        .unwrap();
    let exists: Result<String, _> = stmt.query_row([], |row| row.get(0));
    assert!(exists.is_ok(), "prompt_presets table should exist");
}

#[test]
fn test_db_reopen_idempotent() {
    let temp_file =
        std::env::temp_dir().join(format!("chronicler_reopen_{}.db", std::process::id()));
    {
        let _pool = DbPool::new(temp_file.to_str().unwrap()).unwrap();
    }
    // Re-opening should not create duplicate default game
    let pool = DbPool::new(temp_file.to_str().unwrap()).unwrap();
    let conn = pool.conn();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM games", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        count, 1,
        "Re-opening should not create duplicate default game"
    );
    let _ = std::fs::remove_file(&temp_file);
}

#[test]
fn test_db_message_swipes_table_exists() {
    let pool = DbPool::new(":memory:").unwrap();
    let conn = pool.conn();
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='message_swipes'")
        .unwrap();
    let exists: Result<String, _> = stmt.query_row([], |row| row.get(0));
    assert!(exists.is_ok(), "message_swipes table should exist");
}

#[test]
fn test_db_cascade_delete_game() {
    let pool = DbPool::new(":memory:").unwrap();
    let conn = pool.conn();

    // Insert a game (id 1 is already present from migration v1)
    conn.execute(
        "INSERT INTO games (world_name, name, created_at, updated_at) VALUES ('w', 'n', '1', '1')",
        [],
    )
    .unwrap();
    let game_id: i64 = conn.last_insert_rowid();

    // Insert a snapshot for that game
    conn.execute(
        "INSERT INTO game_state_snapshots (game_id, movement, narrative, scene, npc_encounter_log, created_at)
         VALUES (?1, '{}', '{}', '{}', '{}', '1')",
        rusqlite::params![game_id],
    )
    .unwrap();

    // Insert a message for that game
    conn.execute(
        "INSERT INTO messages (game_id, sender, message_type, timestamp, active_swipe_index, is_deleted)
         VALUES (?1, NULL, 'Narration', '1', 0, 0)",
        rusqlite::params![game_id],
    )
    .unwrap();
    let message_id: i64 = conn.last_insert_rowid();

    // Insert a swipe for that message
    conn.execute(
        "INSERT INTO message_swipes (message_id, swipe_index, text, snapshot_id, location_header, event_header)
         VALUES (?1, 0, 'hello', NULL, NULL, NULL)",
        rusqlite::params![message_id],
    )
    .unwrap();

    // Verify rows exist
    let snap_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM game_state_snapshots WHERE game_id = ?1",
            rusqlite::params![game_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(snap_count, 1, "snapshot should exist before delete");

    let msg_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM messages WHERE game_id = ?1",
            rusqlite::params![game_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(msg_count, 1, "message should exist before delete");

    let swipe_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM message_swipes WHERE message_id = ?1",
            rusqlite::params![message_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(swipe_count, 1, "swipe should exist before delete");

    // Delete the game — CASCADE should clean up everything
    conn.execute(
        "DELETE FROM games WHERE id = ?1",
        rusqlite::params![game_id],
    )
    .unwrap();

    let snap_count_after: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM game_state_snapshots WHERE game_id = ?1",
            rusqlite::params![game_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(snap_count_after, 0, "snapshot should be cascaded-deleted");

    let msg_count_after: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM messages WHERE game_id = ?1",
            rusqlite::params![game_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(msg_count_after, 0, "message should be cascaded-deleted");

    let swipe_count_after: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM message_swipes WHERE message_id = ?1",
            rusqlite::params![message_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(swipe_count_after, 0, "swipe should be cascaded-deleted");
}
