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
