//! Unit tests for the SQLite migration chain — schema evolution on old DB shapes.

use rusqlite::Connection;

use super::plumbing::{migrate_up_to, run_migrations};

/// The last step these tests run: v28 erases the rows they arrange.
const UP_TO_V27: i32 = 27;

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
    migrate_up_to(&conn, UP_TO_V27).unwrap();

    conn.execute(
        "INSERT INTO games (world_name, world_key, name, created_at, updated_at) \
         VALUES ('w', 'w', 'g', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
        [],
    )
    .unwrap();
    conn.pragma_update(None, "user_version", 20).unwrap();

    migrate_up_to(&conn, UP_TO_V27).unwrap();

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
        version, 27,
        "the migration chain must land on the latest version"
    );
}

#[test]
fn test_fresh_database_has_game_posture_columns() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();

    let columns = columns_of(&conn, "games");
    assert!(columns.contains(&"narrative_perspective".to_string()));
    assert!(columns.contains(&"narrative_tense".to_string()));

    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 34);
}

#[test]
fn test_v19_defaults_posture_for_game_with_missing_world() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_up_to(&conn, UP_TO_V27).unwrap();

    conn.execute(
        "INSERT INTO games (world_name, world_key, persona_key, persona_name, name, created_at, updated_at) \
         VALUES ('Ghost World', 'missing_world', 'p', 'P', 'orphan-game', 't', 't')",
        [],
    )
    .unwrap();
    conn.pragma_update(None, "user_version", 18).unwrap();

    migrate_up_to(&conn, UP_TO_V27).expect("migration must survive an orphan world_key");

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
    assert_eq!(version, 27);
}

#[test]
fn test_v23_backfills_game_posture_from_world() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_up_to(&conn, UP_TO_V27).unwrap();

    conn.execute("ALTER TABLE games DROP COLUMN narrative_perspective", [])
        .unwrap();
    conn.execute("ALTER TABLE games DROP COLUMN narrative_tense", [])
        .unwrap();
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

    migrate_up_to(&conn, UP_TO_V27).unwrap();

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
    assert_eq!(version, 27);
}

#[test]
fn test_v22_backfills_swipe_inputs_from_replay_blob_and_drops_column() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_up_to(&conn, UP_TO_V27).unwrap();

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
        (
            Some(
                r#"{"guide":"make it ominous","impersonate":false,"impersonate_direction":null,"impersonate_preset_id":null}"#,
            ),
            0,
            Some("make it ominous"),
            None,
        ),
        // directed impersonation: the pinned preset id must be discarded
        (
            Some(
                r#"{"guide":null,"impersonate":true,"impersonate_direction":"as the player","impersonate_preset_id":"gone_preset"}"#,
            ),
            1,
            None,
            Some("as the player"),
        ),
        (
            Some(
                r#"{"guide":null,"impersonate":true,"impersonate_direction":null,"impersonate_preset_id":null}"#,
            ),
            1,
            None,
            None,
        ),
        (None, 0, None, None),
        // corrupt blob: degrades to plain, mirroring the old parse-time behavior
        (Some("{not json"), 0, None, None),
    ];
    for (idx, (blob, _, _, _)) in blobs.iter().enumerate() {
        conn.execute(
            "INSERT INTO message_swipes (message_id, swipe_index, text, replay) \
             VALUES (1, ?1, 't', ?2)",
            rusqlite::params![idx as i64, blob],
        )
        .unwrap();
    }
    conn.pragma_update(None, "user_version", 21).unwrap();

    migrate_up_to(&conn, UP_TO_V27).unwrap();

    for (idx, (_, expected_impersonated, expected_guide_text, expected_direction)) in
        blobs.iter().enumerate()
    {
        let (impersonated, guide_text, impersonate_direction): (
            i64,
            Option<String>,
            Option<String>,
        ) = conn
            .query_row(
                "SELECT impersonated, guide_text, impersonate_direction FROM message_swipes WHERE swipe_index = ?1",
                rusqlite::params![idx as i64],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(
            impersonated, *expected_impersonated,
            "row {idx}: impersonated flag"
        );
        assert_eq!(
            guide_text.as_deref(),
            *expected_guide_text,
            "row {idx}: guide_text"
        );
        assert_eq!(
            impersonate_direction.as_deref(),
            *expected_direction,
            "row {idx}: impersonate_direction"
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
    assert_eq!(
        version, 27,
        "the migration chain must land on the latest version"
    );
}

#[test]
fn test_v22_is_noop_on_fresh_databases() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();

    let columns = columns_of(&conn, "message_swipes");
    assert!(columns.contains(&"impersonated".to_string()));
    assert!(columns.contains(&"guide_text".to_string()));
    assert!(columns.contains(&"impersonate_direction".to_string()));
    assert!(!columns.contains(&"replay".to_string()));

    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 34);
}

#[test]
fn test_v26_splits_merged_steering_instruction_into_its_two_columns() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_up_to(&conn, UP_TO_V27).unwrap();

    conn.execute("ALTER TABLE message_swipes DROP COLUMN guide_text", [])
        .unwrap();
    conn.execute(
        "ALTER TABLE message_swipes DROP COLUMN impersonate_direction",
        [],
    )
    .unwrap();
    conn.execute(
        "ALTER TABLE message_swipes ADD COLUMN steering_instruction TEXT",
        [],
    )
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
    let rows = [
        (0, Some("make it ominous"), Some("make it ominous"), None),
        (1, Some("as the player"), None, Some("as the player")),
        (1, None, None, None),
        (0, None, None, None),
    ];
    for (idx, (impersonated, steering_instruction, _, _)) in rows.iter().enumerate() {
        conn.execute(
            "INSERT INTO message_swipes (message_id, swipe_index, text, impersonated, steering_instruction) \
             VALUES (1, ?1, 't', ?2, ?3)",
            rusqlite::params![idx as i64, impersonated, steering_instruction],
        )
        .unwrap();
    }
    conn.pragma_update(None, "user_version", 25).unwrap();

    migrate_up_to(&conn, UP_TO_V27).unwrap();

    for (idx, (_, _, expected_guide_text, expected_direction)) in rows.iter().enumerate() {
        let (guide_text, impersonate_direction): (Option<String>, Option<String>) = conn
            .query_row(
                "SELECT guide_text, impersonate_direction FROM message_swipes WHERE swipe_index = ?1",
                rusqlite::params![idx as i64],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(
            guide_text.as_deref(),
            *expected_guide_text,
            "row {idx}: guide_text"
        );
        assert_eq!(
            impersonate_direction.as_deref(),
            *expected_direction,
            "row {idx}: impersonate_direction"
        );
    }

    let columns = columns_of(&conn, "message_swipes");
    assert!(
        !columns.contains(&"steering_instruction".to_string()),
        "steering_instruction must be dropped, got: {columns:?}"
    );
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        version, 27,
        "the migration chain must land on the latest version"
    );
}

#[test]
fn test_v25_backfills_game_display_name_from_stable_name() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_up_to(&conn, UP_TO_V27).unwrap();

    conn.execute("ALTER TABLE games DROP COLUMN display_name", [])
        .unwrap();
    conn.execute(
        "INSERT INTO games (world_name, world_key, name, created_at, updated_at) \
         VALUES ('Redmist Estate', 'redmist_estate', 'Redmist Estate_2026-09-29_1', 't', 't')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO games (world_name, world_key, name, created_at, updated_at) \
         VALUES ('Plain', 'plain', 'Hand Written', 't', 't')",
        [],
    )
    .unwrap();
    conn.pragma_update(None, "user_version", 24).unwrap();

    migrate_up_to(&conn, UP_TO_V27).unwrap();

    let generated: String = conn
        .query_row(
            "SELECT display_name FROM games WHERE world_key = 'redmist_estate'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        generated, "Redmist Estate — 29 Sep 2026 (1)",
        "a generated key must backfill to its readable display name"
    );
    let plain: String = conn
        .query_row(
            "SELECT display_name FROM games WHERE world_key = 'plain'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        plain, "Hand Written",
        "a name that is not a generated key passes through unchanged"
    );

    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 27);
}

#[test]
fn test_v28_deletes_saved_games_and_drops_the_snapshot_tables() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_up_to(&conn, UP_TO_V27).unwrap();
    conn.execute(
        "INSERT INTO games (world_name, world_key, name, created_at, updated_at) \
         VALUES ('w', 'w', 'g', 't', 't')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO game_state_snapshots (game_id, movement, narrative, scene, npc_encounter_log, created_at) \
         VALUES (1, '{}', '{}', '{}', '{}', 't')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO messages (game_id, message_type, timestamp) VALUES (1, '\"Narration\"', 't')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO message_swipes (message_id, swipe_index, text) VALUES (1, 0, 't')",
        [],
    )
    .unwrap();
    conn.execute("INSERT INTO game_docks (game_id) VALUES (1)", [])
        .unwrap();

    run_migrations(&conn).unwrap();

    for table in ["games", "messages", "message_swipes", "game_docks"] {
        let rows: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 0, "{table} must hold no row of a saved Game");
    }
    assert!(
        columns_of(&conn, "game_state_snapshots").is_empty(),
        "the Snapshot table must be gone"
    );
    let swipe_columns = columns_of(&conn, "message_swipes");
    assert!(
        !swipe_columns
            .iter()
            .any(|column| column.starts_with("snapshot"))
    );
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 34);
}

#[test]
fn test_v28_requires_the_kind_the_state_and_the_maker() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();
    conn.execute(
        "INSERT INTO games (world_name, world_key, name, created_at, updated_at) \
         VALUES ('w', 'w', 'g', 't', 't')",
        [],
    )
    .unwrap();

    let no_kind = conn.execute(
        "INSERT INTO messages (game_id, message_type, timestamp) VALUES (1, '\"Narration\"', 't')",
        [],
    );
    assert!(no_kind.is_err(), "a Message with no kind must be refused");

    conn.execute(
        "INSERT INTO messages (game_id, message_type, kind, timestamp) \
         VALUES (1, '\"Narration\"', '\"Narration\"', 't')",
        [],
    )
    .unwrap();
    let no_state = conn.execute(
        "INSERT INTO message_swipes (message_id, swipe_index, text, maker) \
         VALUES (1, 0, 't', '\"Narrator\"')",
        [],
    );
    assert!(no_state.is_err(), "a Swipe with no state must be refused");
    let no_maker = conn.execute(
        "INSERT INTO message_swipes (message_id, swipe_index, text, state) \
         VALUES (1, 0, 't', '{}')",
        [],
    );
    assert!(no_maker.is_err(), "a Swipe with no maker must be refused");
}

#[test]
fn test_v29_drops_the_room_report_and_keeps_the_swipes() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_up_to(&conn, 28).unwrap();
    conn.execute(
        "INSERT INTO games (world_name, world_key, name, created_at, updated_at) \
         VALUES ('w', 'w', 'g', 't', 't')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO messages (game_id, message_type, kind, timestamp) \
         VALUES (1, '\"Narration\"', '\"Narration\"', 't')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO message_swipes (message_id, swipe_index, text, room_report, state, maker) \
         VALUES (1, 0, 'kept text', 'Old Room', '{}', '\"Narrator\"')",
        [],
    )
    .unwrap();

    migrate_up_to(&conn, 29).unwrap();

    assert!(
        !columns_of(&conn, "message_swipes").contains(&"room_report".to_string()),
        "the stored room name must be gone"
    );
    let text: String = conn
        .query_row("SELECT text FROM message_swipes", [], |r| r.get(0))
        .unwrap();
    assert_eq!(text, "kept text", "the Swipe itself survives");
}

#[test]
fn test_v30_deletes_the_saved_games() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_up_to(&conn, 29).unwrap();
    conn.execute(
        "INSERT INTO games (world_name, world_key, name, created_at, updated_at) \
         VALUES ('w', 'w', 'g', 't', 't')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO messages (game_id, message_type, kind, timestamp) \
         VALUES (1, '\"Narration\"', '\"Narration\"', 't')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO message_swipes (message_id, swipe_index, text, state, maker) \
         VALUES (1, 0, 't', '{\"trigger_context\":{\"npc_id\":\"n\"}}', '\"Narrator\"')",
        [],
    )
    .unwrap();
    conn.execute("INSERT INTO game_docks (game_id) VALUES (1)", [])
        .unwrap();

    run_migrations(&conn).unwrap();

    for table in ["games", "messages", "message_swipes", "game_docks"] {
        let rows: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 0, "{table} must hold no row of a saved Game");
    }
}

#[test]
fn test_v31_renames_the_character_ids_tag_in_stored_presets() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_up_to(&conn, 30).unwrap();
    conn.execute(
        "INSERT INTO prompt_presets (id, name, preset_type, role, instructions, writing_style, \
         output_format, created_at, updated_at) VALUES ('tagged', 'n', 'quantifier', \
         'Read <available_npc_ids>.', 'Use the exact NPC IDs provided in the \
         <available_npc_ids> list.', NULL, '<available_npc_ids>', 't', 't')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO prompt_presets (id, name, preset_type, instructions, created_at, updated_at) \
         VALUES ('plain', 'n', 'quantifier', 'Use the NPC IDs.', 't', 't')",
        [],
    )
    .unwrap();

    run_migrations(&conn).unwrap();

    let row = |id: &str| -> (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) {
        conn.query_row(
            "SELECT role, instructions, writing_style, output_format \
             FROM prompt_presets WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap()
    };
    assert_eq!(
        row("tagged"),
        (
            Some("Read <available_character_ids>.".to_string()),
            Some(
                "Use the exact NPC IDs provided in the <available_character_ids> list.".to_string()
            ),
            None,
            Some("<available_character_ids>".to_string()),
        ),
        "the tag is renamed in every text column and a NULL column stays NULL"
    );
    assert_eq!(
        row("plain"),
        (None, Some("Use the NPC IDs.".to_string()), None, None),
        "a preset without the tag is unchanged"
    );
}

#[test]
fn test_v32_renames_the_latest_narration_tag_in_stored_presets() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_up_to(&conn, 31).unwrap();
    conn.execute(
        "INSERT INTO prompt_presets (id, name, preset_type, role, instructions, writing_style, \
         output_format, created_at, updated_at) VALUES ('tagged', 'n', 'quantifier', \
         'Read <LatestNarration>.', 'Base your decision on <LatestNarration> only.', NULL, \
         '<LatestNarration>', 't', 't')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO prompt_presets (id, name, preset_type, instructions, created_at, updated_at) \
         VALUES ('plain', 'n', 'quantifier', 'The narration says it all.', 't', 't')",
        [],
    )
    .unwrap();

    run_migrations(&conn).unwrap();

    let row = |id: &str| -> (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) {
        conn.query_row(
            "SELECT role, instructions, writing_style, output_format \
             FROM prompt_presets WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap()
    };
    assert_eq!(
        row("tagged"),
        (
            Some("Read <LatestMessage>.".to_string()),
            Some("Base your decision on <LatestMessage> only.".to_string()),
            None,
            Some("<LatestMessage>".to_string()),
        ),
        "the tag is renamed in every text column and a NULL column stays NULL"
    );
    assert_eq!(
        row("plain"),
        (
            None,
            Some("The narration says it all.".to_string()),
            None,
            None
        ),
        "a preset without the tag is unchanged"
    );
}

fn insert_settings_with_length(conn: &Connection, response_length: &str) {
    conn.execute(
        "INSERT INTO settings (id, connections, response_length, created_at, updated_at) \
         VALUES (1, '[]', ?1, 't', 't')",
        [response_length],
    )
    .unwrap();
}

fn insert_preset(conn: &Connection, id: &str, preset_type: &str, output_format: Option<&str>) {
    conn.execute(
        "INSERT INTO prompt_presets (id, name, preset_type, output_format, created_at, updated_at) \
         VALUES (?1, 'n', ?2, ?3, 't', 't')",
        rusqlite::params![id, preset_type, output_format],
    )
    .unwrap();
}

fn output_format_of(conn: &Connection, id: &str) -> Option<String> {
    conn.query_row(
        "SELECT output_format FROM prompt_presets WHERE id = ?1",
        [id],
        |r| r.get(0),
    )
    .unwrap()
}

#[test]
fn test_v33_moves_the_response_length_into_the_system_presets_and_adds_the_slots() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_up_to(&conn, 32).unwrap();
    insert_settings_with_length(&conn, "three words");
    insert_preset(&conn, "system_a", "system", Some("Write one paragraph."));
    insert_preset(&conn, "system_none", "system", None);
    insert_preset(
        &conn,
        "impersonate_a",
        "impersonate",
        Some("Reply as the Persona."),
    );
    insert_preset(&conn, "quantifier_a", "quantifier", Some("Answer in JSON."));

    run_migrations(&conn).unwrap();

    assert_eq!(
        output_format_of(&conn, "system_a").as_deref(),
        Some("Write one paragraph.\n\nResponse Length:\nthree words"),
        "a system preset with an output format gets the words in the shape of the old render"
    );
    assert_eq!(output_format_of(&conn, "system_none"), None);
    assert_eq!(
        output_format_of(&conn, "impersonate_a").as_deref(),
        Some("Reply as the Persona."),
        "an Impersonate preset never had the words"
    );
    assert_eq!(
        output_format_of(&conn, "quantifier_a").as_deref(),
        Some("Answer in JSON.")
    );
    let columns = columns_of(&conn, "settings");
    assert!(!columns.contains(&"response_length".to_string()));
    assert!(columns.contains(&"impersonate_connection_id".to_string()));
    assert!(columns.contains(&"options_connection_id".to_string()));
}

#[test]
fn test_v33_without_a_settings_row_or_with_an_empty_length_appends_nothing() {
    let no_row = Connection::open_in_memory().unwrap();
    migrate_up_to(&no_row, 32).unwrap();
    insert_preset(&no_row, "system_a", "system", Some("Write one paragraph."));
    run_migrations(&no_row).unwrap();
    assert_eq!(
        output_format_of(&no_row, "system_a").as_deref(),
        Some("Write one paragraph."),
        "a fresh database has no settings row yet"
    );
    assert!(!columns_of(&no_row, "settings").contains(&"response_length".to_string()));

    let empty = Connection::open_in_memory().unwrap();
    migrate_up_to(&empty, 32).unwrap();
    insert_settings_with_length(&empty, "");
    insert_preset(&empty, "system_a", "system", Some("Write one paragraph."));
    run_migrations(&empty).unwrap();
    assert_eq!(
        output_format_of(&empty, "system_a").as_deref(),
        Some("Write one paragraph."),
        "an empty text renders no header"
    );
}

#[test]
fn test_v34_adds_the_preset_parameters_and_option_count_and_keeps_the_rows() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_up_to(&conn, 33).unwrap();
    insert_preset(&conn, "options_a", "options", Some("Suggest actions."));

    run_migrations(&conn).unwrap();

    let columns = columns_of(&conn, "prompt_presets");
    assert!(columns.contains(&"parameters".to_string()));
    assert!(columns.contains(&"option_count".to_string()));
    let (parameters, option_count, output_format): (String, Option<i64>, Option<String>) = conn
        .query_row(
            "SELECT parameters, option_count, output_format FROM prompt_presets WHERE id = 'options_a'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(parameters, "{}", "an old preset has no parameters set");
    assert_eq!(option_count, None);
    assert_eq!(output_format.as_deref(), Some("Suggest actions."));
    let version: i32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 34);
}

#[test]
fn test_v34_runs_over_a_database_that_already_has_the_columns() {
    let conn = Connection::open_in_memory().unwrap();
    migrate_up_to(&conn, 33).unwrap();
    conn.execute_batch(
        "ALTER TABLE prompt_presets ADD COLUMN parameters TEXT NOT NULL DEFAULT '{}'; \
         ALTER TABLE prompt_presets ADD COLUMN option_count INTEGER;",
    )
    .unwrap();

    run_migrations(&conn).unwrap();

    let version: i32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 34);
}
