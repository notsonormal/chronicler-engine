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
