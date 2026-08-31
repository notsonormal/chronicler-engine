//! Unit tests for the SQLite migration chain — schema evolution on old DB shapes.

use rusqlite::Connection;

use super::plumbing::run_migrations;

/// Column names of `table` via `PRAGMA table_info`.
fn columns_of(conn: &Connection, table: &str) -> Vec<String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .expect("prepare table_info");
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .expect("query table_info");
    rows.collect::<Result<Vec<_>, _>>()
        .expect("read table_info rows")
}

#[test]
fn test_v21_drops_per_game_posture_columns_and_keeps_rows() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();

    // Recreate the pre-v21 shape: posture columns present, user_version = 20.
    conn.execute(
        "ALTER TABLE games ADD COLUMN narrative_perspective TEXT NOT NULL DEFAULT 'third'",
        [],
    )
    .unwrap();
    conn.execute(
        "ALTER TABLE games ADD COLUMN narrative_tense TEXT NOT NULL DEFAULT 'past'",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO games (world_name, world_key, name, created_at, updated_at) \
         VALUES ('w', 'w', 'g', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        [],
    )
    .unwrap();
    conn.pragma_update(None, "user_version", 20).unwrap();

    run_migrations(&conn).unwrap();

    let columns = columns_of(&conn, "games");
    assert!(
        !columns.contains(&"narrative_perspective".to_string()),
        "narrative_perspective must be dropped, got: {columns:?}"
    );
    assert!(
        !columns.contains(&"narrative_tense".to_string()),
        "narrative_tense must be dropped, got: {columns:?}"
    );
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1, "existing game rows must survive the drop");

    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 21, "migration must bump user_version to 21");
}

#[test]
fn test_v21_is_noop_on_fresh_databases() {
    // Fresh CREATE TABLE never had the posture columns; the guarded DROPs
    // must no-op cleanly and still land on user_version 21.
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();

    let columns = columns_of(&conn, "games");
    assert!(!columns.contains(&"narrative_perspective".to_string()));
    assert!(!columns.contains(&"narrative_tense".to_string()));

    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 21);
}

#[test]
fn test_v19_defaults_posture_for_game_with_missing_world() {
    // A game whose world_key has no matching worlds row must not abort the
    // migration chain: the posture backfill falls back to the column
    // defaults instead of failing the NOT NULL constraint.
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();

    // Simulate an upgrade from v18: insert a game referencing a world that
    // does not exist, then re-run from v19.
    conn.execute(
        "INSERT INTO games (world_name, world_key, persona_key, persona_name, name, created_at, updated_at) \
         VALUES ('Ghost World', 'missing_world', 'p', 'P', 'orphan-game', 't', 't')",
        [],
    )
    .unwrap();
    conn.pragma_update(None, "user_version", 18).unwrap();

    run_migrations(&conn).expect("migration must survive an orphan world_key");

    // narrator_mode survives on games (v21 drops only perspective/tense); the
    // orphan game must carry the v19 default, and the row itself must live.
    let mode: String = conn
        .query_row(
            "SELECT narrator_mode FROM games WHERE world_key = 'missing_world'",
            [],
            |r| r.get(0),
        )
        .expect("orphan game row must survive the migration chain");
    assert_eq!(mode, "novel", "orphan game must take the novel default");
    let columns = columns_of(&conn, "games");
    assert!(!columns.contains(&"narrative_perspective".to_string()));
    assert!(!columns.contains(&"narrative_tense".to_string()));

    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 21);
}
