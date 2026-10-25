use crate::adapters::driven::storage::db::DbPool;

#[test]
fn test_db_creates_empty_games_table() {
    let pool = DbPool::new(":memory:").unwrap();
    let conn = pool.conn();
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='games'")
        .unwrap();
    let exists: Result<String, _> = stmt.query_row([], |row| row.get(0));
    assert!(exists.is_ok(), "games table should exist");

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM games", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0, "no default game should be seeded by migrations");
}

#[test]
fn test_db_games_table_has_persona_columns() {
    let pool = DbPool::new(":memory:").unwrap();
    let conn = pool.conn();
    let (has_persona_key, has_persona_name): (i64, i64) = conn
        .query_row(
            "SELECT
                (SELECT COUNT(*) FROM pragma_table_info('games') WHERE name='persona_key'),
                (SELECT COUNT(*) FROM pragma_table_info('games') WHERE name='persona_name')",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(has_persona_key, 1, "games.persona_key column must exist");
    assert_eq!(has_persona_name, 1, "games.persona_name column must exist");
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
    let pool = DbPool::new(temp_file.to_str().unwrap()).unwrap();
    let conn = pool.conn();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM games", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0, "Re-opening should not create any games");
    let _ = std::fs::remove_file(&temp_file);
}

#[test]
fn test_db_v14_drops_starting_room_id_column() {
    let pool = DbPool::new(":memory:").unwrap();
    let conn = pool.conn();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('worlds') WHERE name='starting_room_id'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        count, 0,
        "worlds.starting_room_id column must be dropped in v14"
    );
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
fn test_db_message_swipes_flat_input_columns() {
    let pool = DbPool::new(":memory:").unwrap();
    let conn = pool.conn();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('message_swipes') WHERE name IN ('impersonated', 'guide_text', 'impersonate_direction')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        count, 3,
        "message_swipes must carry the flattened input columns after v22"
    );
    let merged: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('message_swipes') WHERE name='steering_instruction'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        merged, 0,
        "message_swipes.steering_instruction must be split by the v26 migration"
    );
    let replay: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('message_swipes') WHERE name='replay'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        replay, 0,
        "message_swipes.replay must be dropped by the v22 migration"
    );
}

#[test]
fn test_db_cascade_delete_game() {
    let pool = DbPool::new(":memory:").unwrap();
    let conn = pool.conn();

    conn.execute(
        "INSERT INTO games (world_name, world_key, persona_key, persona_name, name, created_at, updated_at) VALUES ('w', 'w', 'julian', 'Julian', 'n', '1', '1')",
        [],
    )
    .unwrap();
    let game_id: i64 = conn.last_insert_rowid();

    conn.execute(
        "INSERT INTO messages (game_id, message_type, kind, timestamp, active_swipe_index)
         VALUES (?1, 'Narration', 'Narration', '1', 0)",
        rusqlite::params![game_id],
    )
    .unwrap();
    let message_id: i64 = conn.last_insert_rowid();

    conn.execute(
        "INSERT INTO message_swipes (message_id, swipe_index, text, event_header, state, maker)
         VALUES (?1, 0, 'hello', NULL, '{}', 'Narrator')",
        rusqlite::params![message_id],
    )
    .unwrap();

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

    conn.execute(
        "DELETE FROM games WHERE id = ?1",
        rusqlite::params![game_id],
    )
    .unwrap();

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
