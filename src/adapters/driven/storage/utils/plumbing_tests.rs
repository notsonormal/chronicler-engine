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
fn test_pre_v21_database_restores_posture_and_keeps_rows() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();

    // Recreate the pre-v21 shape (user_version = 20): v21 dropped the
    // columns as write-only; v23 restores them. The orphan game takes the
    // defaults.
    conn.execute(
        "INSERT INTO games (world_name, world_key, name, created_at, updated_at) \
         VALUES ('w', 'w', 'g', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        [],
    )
    .unwrap();
    conn.pragma_update(None, "user_version", 20).unwrap();

    run_migrations(&conn).unwrap();

    let columns = columns_of(&conn, "games");
    assert!(columns.contains(&"narrative_perspective".to_string()));
    assert!(columns.contains(&"narrative_tense".to_string()));
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1, "existing game rows must survive the chain");
    let (perspective, tense): (String, String) = conn
        .query_row(
            "SELECT narrative_perspective, narrative_tense FROM games",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(perspective, "third", "orphan game takes the default");
    assert_eq!(tense, "past", "orphan game takes the default");

    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        version, 23,
        "the migration chain must land on the latest version"
    );
}

#[test]
fn test_fresh_database_has_game_posture_columns() {
    // Fresh installs run the whole chain: v19 adds the columns, v21 drops
    // them, v23 restores them. Terminal shape: present, version 23.
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();

    let columns = columns_of(&conn, "games");
    assert!(columns.contains(&"narrative_perspective".to_string()));
    assert!(columns.contains(&"narrative_tense".to_string()));

    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 23);
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

    // The orphan game must carry the defaults, and the row itself must live.
    let mode: String = conn
        .query_row(
            "SELECT narrator_mode FROM games WHERE world_key = 'missing_world'",
            [],
            |r| r.get(0),
        )
        .expect("orphan game row must survive the migration chain");
    assert_eq!(mode, "novel", "orphan game must take the novel default");
    let (perspective, tense): (String, String) = conn
        .query_row(
            "SELECT narrative_perspective, narrative_tense FROM games WHERE world_key = 'missing_world'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(perspective, "third", "orphan game takes the default");
    assert_eq!(tense, "past", "orphan game takes the default");

    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 23);
}

#[test]
fn test_v23_backfills_game_posture_from_world() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();

    // Recreate the post-v21 shape: posture columns absent, user_version = 22.
    conn.execute("ALTER TABLE games DROP COLUMN narrative_perspective", [])
        .unwrap();
    conn.execute("ALTER TABLE games DROP COLUMN narrative_tense", [])
        .unwrap();
    // A world with non-default posture and a game that must inherit it.
    conn.execute(
        "INSERT INTO worlds (key, name, created_at, updated_at) \
         VALUES ('w1', 'World One', 't', 't')",
        [],
    )
    .unwrap();
    conn.execute(
        "UPDATE worlds SET narrative_perspective = 'second', narrative_tense = 'present' \
         WHERE key = 'w1'",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO games (world_name, world_key, name, created_at, updated_at) \
         VALUES ('w1', 'w1', 'g', 't', 't')",
        [],
    )
    .unwrap();
    conn.pragma_update(None, "user_version", 22).unwrap();

    run_migrations(&conn).unwrap();

    let (perspective, tense): (String, String) = conn
        .query_row(
            "SELECT narrative_perspective, narrative_tense FROM games WHERE world_key = 'w1'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(perspective, "second", "game inherits the world's posture");
    assert_eq!(tense, "present", "game inherits the world's posture");

    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 23);
}

#[test]
fn test_v22_backfills_swipe_inputs_from_replay_blob_and_drops_column() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();

    // Recreate the pre-v22 shape: the v15 replay blob column with filled rows.
    conn.execute("ALTER TABLE message_swipes ADD COLUMN replay TEXT", [])
        .unwrap();
    conn.execute(
        "INSERT INTO games (world_name, world_key, name, created_at, updated_at) \
         VALUES ('w', 'w', 'g', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO messages (game_id, message_type, timestamp) \
         VALUES (1, '\"Narration\"', '2026-01-01T00:00:00Z')",
        [],
    )
    .unwrap();
    let blobs = [
        // (replay json, expected impersonated, expected steering_instruction)
        // guided row
        (
            Some(
                r#"{"guide":"make it ominous","impersonate":false,"impersonate_direction":null,"impersonate_preset_id":null}"#,
            ),
            0,
            Some("make it ominous"),
        ),
        // directed impersonation: the pinned preset id must be discarded
        (
            Some(
                r#"{"guide":null,"impersonate":true,"impersonate_direction":"as the player","impersonate_preset_id":"gone_preset"}"#,
            ),
            1,
            Some("as the player"),
        ),
        // bare impersonation
        (
            Some(
                r#"{"guide":null,"impersonate":true,"impersonate_direction":null,"impersonate_preset_id":null}"#,
            ),
            1,
            None,
        ),
        // plain row without a blob
        (None, 0, None),
        // corrupt blob: degrades to plain, mirroring the old parse-time behavior
        (Some("{not json"), 0, None),
    ];
    for (idx, (blob, _, _)) in blobs.iter().enumerate() {
        conn.execute(
            "INSERT INTO message_swipes (message_id, swipe_index, text, replay) \
             VALUES (1, ?1, 't', ?2)",
            rusqlite::params![idx as i64, blob],
        )
        .unwrap();
    }
    conn.pragma_update(None, "user_version", 21).unwrap();

    run_migrations(&conn).unwrap();

    for (idx, (_, expected_impersonated, expected_steering_instruction)) in blobs.iter().enumerate()
    {
        let (impersonated, steering_instruction): (i64, Option<String>) = conn
            .query_row(
                "SELECT impersonated, steering_instruction FROM message_swipes WHERE swipe_index = ?1",
                rusqlite::params![idx as i64],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(
            impersonated, *expected_impersonated,
            "row {idx}: impersonated flag"
        );
        assert_eq!(
            steering_instruction.as_deref(),
            *expected_steering_instruction,
            "row {idx}: steering_instruction"
        );
    }

    let columns = columns_of(&conn, "message_swipes");
    assert!(
        !columns.contains(&"replay".to_string()),
        "replay must be dropped, got: {columns:?}"
    );
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 23, "migration must bump user_version to 23");
}

#[test]
fn test_v22_is_noop_on_fresh_databases() {
    // Fresh installs never had the replay blob; the guarded migration must
    // no-op cleanly and still land on the latest user_version.
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();

    let columns = columns_of(&conn, "message_swipes");
    assert!(columns.contains(&"impersonated".to_string()));
    assert!(columns.contains(&"steering_instruction".to_string()));
    assert!(!columns.contains(&"replay".to_string()));

    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 23);
}
