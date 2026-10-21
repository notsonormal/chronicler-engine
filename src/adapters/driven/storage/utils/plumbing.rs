//! [DOC: docs/diataxis/reference/storage.md]
//! Storage-layer plumbing utilities (datetime parsing, schema migrations).

use rusqlite::Connection;

use crate::error::{DatabaseContext, EngineError};

pub(crate) fn parse_datetime(
    rfc3339: &str,
    field: &str,
) -> Result<chrono::DateTime<chrono::Utc>, EngineError> {
    chrono::DateTime::parse_from_rfc3339(rfc3339)
        .map_err(|e| EngineError::Parse(format!("Invalid {field}: {e}")))
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

const LATEST_VERSION: i32 = 30;

pub(crate) fn run_migrations(conn: &Connection) -> Result<(), EngineError> {
    migrate_up_to(conn, LATEST_VERSION)
}

/// Runs the steps after the database's version, up to and including `up_to`. A
/// test stops before a later step that erases the rows an earlier step wrote.
pub(crate) fn migrate_up_to(conn: &Connection, up_to: i32) -> Result<(), EngineError> {
    fn column_exists(conn: &Connection, table: &str, col: &str) -> bool {
        conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info(?) WHERE name = ?",
            [table, col],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0)
            > 0
    }

    fn games_with_unknown_world(conn: &Connection) -> Result<Vec<String>, EngineError> {
        let mut stmt = conn
            .prepare(
                "SELECT name FROM games \
                 WHERE world_key NOT IN (SELECT key FROM worlds)",
            )
            .db_context("Migration failed")?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .db_context("Migration failed")?;
        rows.collect::<Result<Vec<_>, _>>()
            .db_context("Migration failed")
    }

    let version: i32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap_or(0);
    let due = |step: i32| version < step && step <= up_to;

    // Batched to avoid one fsync per statement under default `synchronous=FULL`;
    // the single commit is still fsynced, so durability is unchanged.
    let tx = conn
        .unchecked_transaction()
        .db_context("Failed to begin migration")?;
    let conn = &tx;
    let exec = |sql: &str| conn.execute(sql, []).db_context("Migration failed");

    if due(9) {
        exec(
            "CREATE TABLE IF NOT EXISTS games (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                world_name TEXT NOT NULL DEFAULT 'default',
                world_key TEXT NOT NULL DEFAULT 'default',
                name TEXT NOT NULL DEFAULT 'Unnamed',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )?;

        exec(
            "CREATE TABLE IF NOT EXISTS game_state_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL DEFAULT 1 REFERENCES games(id) ON DELETE CASCADE,
                movement TEXT NOT NULL,
                narrative TEXT NOT NULL,
                scene TEXT NOT NULL,
                npc_encounter_log TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
        )?;

        exec(
            "CREATE INDEX IF NOT EXISTS idx_snapshots_game_latest ON game_state_snapshots(game_id, created_at DESC)",
        )?;

        exec(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL DEFAULT 1 REFERENCES games(id) ON DELETE CASCADE,
                sender TEXT,
                message_type TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                active_swipe_index INTEGER NOT NULL DEFAULT 0,
                is_deleted INTEGER NOT NULL DEFAULT 0
            )",
        )?;

        exec("CREATE INDEX IF NOT EXISTS idx_messages_game_id ON messages(game_id, id)")?;

        // `snapshot_id` is deliberately not a FOREIGN KEY: a cascade would delete
        // a swipe whenever its snapshot is deleted.
        exec(
            "CREATE TABLE IF NOT EXISTS message_swipes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message_id INTEGER NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
                swipe_index INTEGER NOT NULL,
                text TEXT NOT NULL,
                snapshot_id INTEGER,
                location_header TEXT,
                event_header TEXT,
                UNIQUE(message_id, swipe_index)
            )",
        )?;

        exec(
            "CREATE TABLE IF NOT EXISTS llm_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_name TEXT NOT NULL,
                backend_name TEXT NOT NULL,
                model_name TEXT NOT NULL,
                system_prompt TEXT NOT NULL,
                user_prompt TEXT NOT NULL,
                raw_request_json TEXT NOT NULL,
                raw_response_json TEXT NOT NULL,
                parsed_response TEXT NOT NULL,
                error_message TEXT,
                created_at TEXT NOT NULL
            )",
        )?;

        exec(
            "CREATE INDEX IF NOT EXISTS idx_llm_messages_created_at ON llm_messages(created_at DESC)",
        )?;

        exec(
            "CREATE TABLE IF NOT EXISTS prompt_presets (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                preset_type TEXT NOT NULL,
                role TEXT,
                instructions TEXT,
                writing_style TEXT,
                output_format TEXT,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )?;

        exec("CREATE INDEX IF NOT EXISTS idx_prompt_presets_type ON prompt_presets(preset_type)")?;

        conn.pragma_update(None, "user_version", 9)
            .db_context("Failed to set user_version")?;
    }

    if due(10) {
        exec(
            "CREATE TABLE IF NOT EXISTS worlds (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL UNIQUE,      -- original string ID (e.g. 'redmist_estate')
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                global_rules TEXT NOT NULL DEFAULT '[]',  -- JSON: Vec<String>
                scenarios TEXT NOT NULL DEFAULT '[]',     -- JSON: Vec<StartingScenario>
                default_scenario_id TEXT,
                default_room_image TEXT,
                player_key TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )?;

        exec(
            "CREATE TABLE IF NOT EXISTS maps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                world_id INTEGER NOT NULL REFERENCES worlds(id) ON DELETE CASCADE,
                map_data TEXT NOT NULL,         -- JSON: full serialized MapDef
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )?;
        exec("CREATE INDEX IF NOT EXISTS idx_maps_world ON maps(world_id)")?;

        exec(
            "CREATE TABLE IF NOT EXISTS personas (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL UNIQUE,      -- filename stem (e.g. 'julian')
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                personality TEXT NOT NULL DEFAULT '',
                scenario TEXT NOT NULL DEFAULT '',
                example_dialogue TEXT NOT NULL DEFAULT '',
                summary TEXT,
                profile_image TEXT,
                headshot_image TEXT,
                inventory TEXT NOT NULL DEFAULT '[]',  -- JSON: Vec<String>
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )?;

        exec(
            "CREATE TABLE IF NOT EXISTS characters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL,             -- from NpcCard.id (e.g. 'elena_voss')
                world_id INTEGER NOT NULL REFERENCES worlds(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                personality TEXT NOT NULL DEFAULT '',
                scenario TEXT NOT NULL DEFAULT '',
                example_dialogue TEXT NOT NULL DEFAULT '',
                summary TEXT,
                profile_image TEXT,
                headshot_image TEXT,
                inventory TEXT NOT NULL DEFAULT '[]',     -- JSON: Vec<String>
                triggers TEXT NOT NULL DEFAULT '[]',      -- JSON: Vec<Trigger>
                relationships TEXT NOT NULL DEFAULT '[]', -- JSON: Vec<Relationship>
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(key, world_id)
            )",
        )?;
        exec("CREATE INDEX IF NOT EXISTS idx_characters_world ON characters(world_id)")?;

        exec(
            "CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),  -- singleton row
                connections TEXT NOT NULL DEFAULT '[]',  -- JSON: Vec<Connection>
                narration_connection_id TEXT NOT NULL DEFAULT 'openrouter-gpt-4o-mini',
                quantifier_connection_id TEXT NOT NULL DEFAULT 'openrouter-gpt-4o-mini',
                response_length TEXT NOT NULL DEFAULT '',
                text_check TEXT NOT NULL DEFAULT '{}',    -- JSON: TextCheckSettings
                agents TEXT NOT NULL DEFAULT '[]',        -- JSON: Vec<AgentConfig>
                active_system_prompt_preset_id TEXT NOT NULL DEFAULT 'system_default',
                active_quantifier_prompt_preset_id TEXT NOT NULL DEFAULT 'quantifier_default',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
        )?;

        conn.pragma_update(None, "user_version", 11)
            .db_context("Failed to set user_version")?;
    }

    if due(12) {
        if !column_exists(conn, "games", "world_key") {
            exec("ALTER TABLE games ADD COLUMN world_key TEXT NOT NULL DEFAULT ''")?;
        }
        exec(
            "UPDATE games SET world_key = COALESCE(
            (SELECT key FROM worlds WHERE worlds.name = games.world_name),
            'redmist_estate'
        ) WHERE world_key = ''",
        )?;

        conn.pragma_update(None, "user_version", 12)
            .db_context("Failed to set user_version")?;
    }

    if due(13) {
        exec("ALTER TABLE games ADD COLUMN persona_key TEXT NOT NULL DEFAULT ''")?;
        exec("ALTER TABLE games ADD COLUMN persona_name TEXT NOT NULL DEFAULT ''")?;
        exec("ALTER TABLE worlds DROP COLUMN player_key")?;
        conn.pragma_update(None, "user_version", 13)
            .db_context("Failed to set user_version")?;
    }

    if due(14) {
        if column_exists(conn, "worlds", "starting_room_id") {
            exec("ALTER TABLE worlds DROP COLUMN starting_room_id")?;
        }
        conn.pragma_update(None, "user_version", 14)
            .db_context("Failed to set user_version")?;
    }

    if due(15) {
        if !column_exists(conn, "message_swipes", "replay") {
            exec("ALTER TABLE message_swipes ADD COLUMN replay TEXT")?;
        }
        conn.pragma_update(None, "user_version", 15)
            .db_context("Failed to set user_version")?;
    }

    if due(16) {
        if !column_exists(conn, "settings", "active_impersonate_prompt_preset_id") {
            exec(
                "ALTER TABLE settings ADD COLUMN active_impersonate_prompt_preset_id \
                 TEXT NOT NULL DEFAULT 'impersonate_default'",
            )?;
        }
        conn.pragma_update(None, "user_version", 16)
            .db_context("Failed to set user_version")?;
    }

    if due(17) {
        if column_exists(conn, "messages", "sender") {
            exec("ALTER TABLE messages DROP COLUMN sender")?;
        }
        conn.pragma_update(None, "user_version", 17)
            .db_context("Failed to set user_version")?;
    }

    if due(18) {
        if !column_exists(conn, "settings", "narrative_perspective") {
            exec(
                "ALTER TABLE settings ADD COLUMN narrative_perspective \
                 TEXT NOT NULL DEFAULT 'third'",
            )?;
        }
        if !column_exists(conn, "settings", "narrative_tense") {
            exec(
                "ALTER TABLE settings ADD COLUMN narrative_tense \
                 TEXT NOT NULL DEFAULT 'past'",
            )?;
        }
        conn.pragma_update(None, "user_version", 18)
            .db_context("Failed to set user_version")?;
    }

    if due(19) {
        if !column_exists(conn, "worlds", "narrator_mode") {
            exec("ALTER TABLE worlds ADD COLUMN narrator_mode TEXT NOT NULL DEFAULT 'novel'")?;
        }
        if !column_exists(conn, "worlds", "narrative_perspective") {
            exec(
                "ALTER TABLE worlds ADD COLUMN narrative_perspective TEXT NOT NULL DEFAULT 'third'",
            )?;
        }
        if !column_exists(conn, "worlds", "narrative_tense") {
            exec("ALTER TABLE worlds ADD COLUMN narrative_tense TEXT NOT NULL DEFAULT 'past'")?;
        }

        if !column_exists(conn, "games", "narrator_mode") {
            exec("ALTER TABLE games ADD COLUMN narrator_mode TEXT NOT NULL DEFAULT 'novel'")?;
        }
        if !column_exists(conn, "games", "narrative_perspective") {
            exec(
                "ALTER TABLE games ADD COLUMN narrative_perspective TEXT NOT NULL DEFAULT 'third'",
            )?;
        }
        if !column_exists(conn, "games", "narrative_tense") {
            exec("ALTER TABLE games ADD COLUMN narrative_tense TEXT NOT NULL DEFAULT 'past'")?;
        }
        if !column_exists(conn, "games", "active_system_prompt_preset_id") {
            exec(
                "ALTER TABLE games ADD COLUMN active_system_prompt_preset_id \
                 TEXT NOT NULL DEFAULT 'system_default'",
            )?;
        }
        if !column_exists(conn, "games", "active_quantifier_prompt_preset_id") {
            exec(
                "ALTER TABLE games ADD COLUMN active_quantifier_prompt_preset_id \
                 TEXT NOT NULL DEFAULT 'quantifier_default'",
            )?;
        }
        if !column_exists(conn, "games", "active_impersonate_prompt_preset_id") {
            exec(
                "ALTER TABLE games ADD COLUMN active_impersonate_prompt_preset_id \
                 TEXT NOT NULL DEFAULT 'impersonate_default'",
            )?;
        }

        // COALESCE keeps the migration alive for a game whose world_key has
        // no matching world row: the subquery would otherwise produce NULL and
        // fail the NOT NULL constraint, aborting startup.
        exec(
            "UPDATE games SET \
               narrator_mode = COALESCE((SELECT narrator_mode FROM worlds WHERE worlds.key = games.world_key), 'novel'), \
               narrative_perspective = COALESCE((SELECT narrative_perspective FROM worlds WHERE worlds.key = games.world_key), 'third'), \
               narrative_tense = COALESCE((SELECT narrative_tense FROM worlds WHERE worlds.key = games.world_key), 'past')",
        )?;

        let orphans = games_with_unknown_world(conn)?;
        for key in &orphans {
            tracing::warn!(
                "migration v19: game '{key}' references world_key with no matching world row; \
                 posture defaulted to novel/third/past"
            );
        }

        if !column_exists(conn, "settings", "mode_preset_registry") {
            exec(
                "ALTER TABLE settings ADD COLUMN mode_preset_registry TEXT NOT NULL DEFAULT '{}'",
            )?;
            // Fresh default for rows that took the '{}' default (no legacy columns).
            exec(
                "UPDATE settings SET mode_preset_registry = '{\"novel\":{\"system_prompt_preset_id\":\"system_default\",\"quantifier_prompt_preset_id\":\"quantifier_default\",\"impersonate_prompt_preset_id\":\"impersonate_default\"},\"interactive_fiction\":{\"system_prompt_preset_id\":\"system_if_default\",\"quantifier_prompt_preset_id\":\"quantifier_default\",\"impersonate_prompt_preset_id\":\"impersonate_default\"}}' \
               WHERE mode_preset_registry = '{}'",
            )?;
            exec(
                "UPDATE settings SET mode_preset_registry = json_set(mode_preset_registry, '$.novel.system_prompt_preset_id', active_system_prompt_preset_id) \
               WHERE active_system_prompt_preset_id IS NOT NULL",
            )?;
            exec(
                "UPDATE settings SET mode_preset_registry = json_set(mode_preset_registry, '$.novel.quantifier_prompt_preset_id', active_quantifier_prompt_preset_id) \
               WHERE active_quantifier_prompt_preset_id IS NOT NULL",
            )?;
            exec(
                "UPDATE settings SET mode_preset_registry = json_set(mode_preset_registry, '$.novel.impersonate_prompt_preset_id', active_impersonate_prompt_preset_id) \
               WHERE active_impersonate_prompt_preset_id IS NOT NULL",
            )?;
            exec(
                "UPDATE settings SET mode_preset_registry = json_set(mode_preset_registry, '$.interactive_fiction.quantifier_prompt_preset_id', mode_preset_registry->'$.novel.quantifier_prompt_preset_id')",
            )?;
            exec(
                "UPDATE settings SET mode_preset_registry = json_set(mode_preset_registry, '$.interactive_fiction.impersonate_prompt_preset_id', mode_preset_registry->'$.novel.impersonate_prompt_preset_id')",
            )?;
        }

        if column_exists(conn, "settings", "narrative_perspective") {
            exec("ALTER TABLE settings DROP COLUMN narrative_perspective")?;
        }
        if column_exists(conn, "settings", "narrative_tense") {
            exec("ALTER TABLE settings DROP COLUMN narrative_tense")?;
        }
        if column_exists(conn, "settings", "active_system_prompt_preset_id") {
            exec("ALTER TABLE settings DROP COLUMN active_system_prompt_preset_id")?;
        }
        if column_exists(conn, "settings", "active_quantifier_prompt_preset_id") {
            exec("ALTER TABLE settings DROP COLUMN active_quantifier_prompt_preset_id")?;
        }
        if column_exists(conn, "settings", "active_impersonate_prompt_preset_id") {
            exec("ALTER TABLE settings DROP COLUMN active_impersonate_prompt_preset_id")?;
        }

        conn.pragma_update(None, "user_version", 19)
            .db_context("Failed to set user_version")?;
    }

    if due(20) {
        // The column is born all-allowed on every existing row — no user value
        // can pre-exist a column at birth.
        if !column_exists(conn, "prompt_presets", "allowed_modes") {
            exec(
                "ALTER TABLE prompt_presets ADD COLUMN allowed_modes \
                 TEXT NOT NULL DEFAULT '[\"novel\",\"interactive_fiction\"]'",
            )?;
        }
        exec(
            "UPDATE prompt_presets SET allowed_modes = '[\"novel\"]' \
             WHERE id = 'system_default'",
        )?;

        // The RHS is evaluated against the pre-update column value (SQLite
        // semantics). The json_type guard keeps a partially applied block safe
        // to re-run.
        exec(
            "UPDATE settings SET mode_preset_registry = json_array( \
               json_object('mode', 'novel', \
                 'system_prompt_preset_id', json_extract(mode_preset_registry, '$.novel.system_prompt_preset_id'), \
                 'quantifier_prompt_preset_id', json_extract(mode_preset_registry, '$.novel.quantifier_prompt_preset_id'), \
                 'impersonate_prompt_preset_id', json_extract(mode_preset_registry, '$.novel.impersonate_prompt_preset_id')), \
               json_object('mode', 'interactive_fiction', \
                 'system_prompt_preset_id', json_extract(mode_preset_registry, '$.interactive_fiction.system_prompt_preset_id'), \
                 'quantifier_prompt_preset_id', json_extract(mode_preset_registry, '$.interactive_fiction.quantifier_prompt_preset_id'), \
                 'impersonate_prompt_preset_id', json_extract(mode_preset_registry, '$.interactive_fiction.impersonate_prompt_preset_id'))) \
             WHERE json_type(mode_preset_registry) = 'object'",
        )?;

        conn.pragma_update(None, "user_version", 20)
            .db_context("Failed to set user_version")?;
    }

    if due(21) {
        // The prompt assembler stamps world posture as the sole owner; nothing
        // reads the per-game copies.
        if column_exists(conn, "games", "narrative_perspective") {
            exec("ALTER TABLE games DROP COLUMN narrative_perspective")?;
        }
        if column_exists(conn, "games", "narrative_tense") {
            exec("ALTER TABLE games DROP COLUMN narrative_tense")?;
        }

        conn.pragma_update(None, "user_version", 21)
            .db_context("Failed to set user_version")?;
    }

    if due(22) {
        // A corrupt v15 replay blob degraded the turn to a plain retry at parse
        // time, so the backfill skips non-JSON values. Producers never set
        // impersonate_direction and guide together; the impersonate direction wins.
        if !column_exists(conn, "message_swipes", "impersonated") {
            exec("ALTER TABLE message_swipes ADD COLUMN impersonated INTEGER NOT NULL DEFAULT 0")?;
        }
        if !column_exists(conn, "message_swipes", "steering_instruction") {
            exec("ALTER TABLE message_swipes ADD COLUMN steering_instruction TEXT")?;
        }
        if column_exists(conn, "message_swipes", "replay") {
            exec(
                "UPDATE message_swipes \
                 SET impersonated = COALESCE(json_extract(replay, '$.impersonate'), 0), \
                     steering_instruction = COALESCE(json_extract(replay, '$.impersonate_direction'), json_extract(replay, '$.guide')) \
                 WHERE replay IS NOT NULL AND json_valid(replay)",
            )?;
            exec("ALTER TABLE message_swipes DROP COLUMN replay")?;
        }

        conn.pragma_update(None, "user_version", 22)
            .db_context("Failed to set user_version")?;
    }

    if due(23) {
        // Restore the per-game posture columns v21 dropped as write-only: the
        // narration path now reads posture from the game, so the copies are
        // live data again. Backfill mirrors v19.
        if !column_exists(conn, "games", "narrative_perspective") {
            exec(
                "ALTER TABLE games ADD COLUMN narrative_perspective TEXT NOT NULL DEFAULT 'third'",
            )?;
        }
        if !column_exists(conn, "games", "narrative_tense") {
            exec("ALTER TABLE games ADD COLUMN narrative_tense TEXT NOT NULL DEFAULT 'past'")?;
        }

        exec(
            "UPDATE games SET \
               narrative_perspective = COALESCE((SELECT narrative_perspective FROM worlds WHERE worlds.key = games.world_key), 'third'), \
               narrative_tense = COALESCE((SELECT narrative_tense FROM worlds WHERE worlds.key = games.world_key), 'past')",
        )?;

        let orphans = games_with_unknown_world(conn)?;
        for key in &orphans {
            tracing::warn!(
                "migration v23: game '{key}' references world_key with no matching world row; \
                 posture defaulted to third/past"
            );
        }

        conn.pragma_update(None, "user_version", 23)
            .db_context("Failed to set user_version")?;
    }

    if due(24) {
        // Options autogeneration follows the v19/v23 posture pattern. The options
        // preset id is mode-agnostic; games inherit a copy from settings at creation.
        if !column_exists(conn, "worlds", "options_always_on") {
            exec("ALTER TABLE worlds ADD COLUMN options_always_on INTEGER NOT NULL DEFAULT 0")?;
        }
        if !column_exists(conn, "games", "options_always_on") {
            exec("ALTER TABLE games ADD COLUMN options_always_on INTEGER NOT NULL DEFAULT 0")?;
        }
        if !column_exists(conn, "games", "active_options_prompt_preset_id") {
            exec(
                "ALTER TABLE games ADD COLUMN active_options_prompt_preset_id \
                 TEXT NOT NULL DEFAULT 'options_default'",
            )?;
        }
        if !column_exists(conn, "settings", "active_options_prompt_preset_id") {
            exec(
                "ALTER TABLE settings ADD COLUMN active_options_prompt_preset_id \
                 TEXT NOT NULL DEFAULT 'options_default'",
            )?;
        }

        exec(
            "UPDATE games SET \
               options_always_on = COALESCE((SELECT options_always_on FROM worlds WHERE worlds.key = games.world_key), 0)",
        )?;

        let orphans = games_with_unknown_world(conn)?;
        for key in &orphans {
            tracing::warn!(
                "migration v24: game '{key}' references world_key with no matching world row; \
                 options_always_on defaulted to off"
            );
        }

        conn.pragma_update(None, "user_version", 24)
            .db_context("Failed to set user_version")?;
    }

    if due(25) {
        // The Game display name is separate from the stable generated `name`,
        // so a rename can never disturb the uniqueness scan. Existing rows are
        // backfilled from their key so the first render after upgrade already
        // shows a readable label; a key that does not parse passes through.
        if !column_exists(conn, "games", "display_name") {
            exec("ALTER TABLE games ADD COLUMN display_name TEXT NOT NULL DEFAULT ''")?;
        }
        let legacy_rows: Vec<(i64, String)> = {
            let mut stmt = conn
                .prepare("SELECT id, name FROM games WHERE display_name = ''")
                .db_context("Migration failed")?;
            let rows = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .db_context("Migration failed")?;
            rows.collect::<Result<Vec<_>, _>>()
                .db_context("Migration failed")?
        };
        for (id, name) in legacy_rows {
            let display_name = crate::domain::model::utils::game_name::default_display_name(&name);
            conn.execute(
                "UPDATE games SET display_name = ?1 WHERE id = ?2",
                rusqlite::params![display_name, id],
            )
            .db_context("Migration failed")?;
        }

        conn.pragma_update(None, "user_version", 25)
            .db_context("Failed to set user_version")?;
    }

    if due(26) {
        // The `impersonated` flag picks the owner column, so no row ends up with
        // both: the old merged column fills that column and the split column
        // keeps the other. The old column is dropped below, so the UPDATE runs
        // at most once.
        if !column_exists(conn, "message_swipes", "guide_text") {
            exec("ALTER TABLE message_swipes ADD COLUMN guide_text TEXT")?;
        }
        if !column_exists(conn, "message_swipes", "impersonate_direction") {
            exec("ALTER TABLE message_swipes ADD COLUMN impersonate_direction TEXT")?;
        }
        if column_exists(conn, "message_swipes", "steering_instruction") {
            exec(
                "UPDATE message_swipes SET \
                   guide_text = CASE WHEN impersonated = 0 THEN steering_instruction ELSE guide_text END, \
                   impersonate_direction = CASE WHEN impersonated = 1 THEN steering_instruction ELSE impersonate_direction END",
            )?;
            exec("ALTER TABLE message_swipes DROP COLUMN steering_instruction")?;
        }

        conn.pragma_update(None, "user_version", 26)
            .db_context("Failed to set user_version")?;
    }

    if due(27) {
        // The new columns are additive and backfill nothing. `room_report` renames
        // today's `location_header`, so the existing rows survive with no reader
        // picking between two columns.
        if !column_exists(conn, "messages", "kind") {
            exec("ALTER TABLE messages ADD COLUMN kind TEXT")?;
        }
        if !column_exists(conn, "message_swipes", "state") {
            exec("ALTER TABLE message_swipes ADD COLUMN state TEXT")?;
        }
        if !column_exists(conn, "message_swipes", "maker") {
            exec("ALTER TABLE message_swipes ADD COLUMN maker TEXT")?;
        }
        if column_exists(conn, "message_swipes", "location_header") {
            exec("ALTER TABLE message_swipes RENAME COLUMN location_header TO room_report")?;
        }
        if !column_exists(conn, "games", "failure") {
            exec("ALTER TABLE games ADD COLUMN failure TEXT")?;
        }

        // The dock is a table of its own, because the config routes write the Game
        // record with no busy check.
        exec(
            "CREATE TABLE IF NOT EXISTS game_docks (\n                game_id INTEGER PRIMARY KEY REFERENCES games(id) ON DELETE CASCADE,\n                offered TEXT NOT NULL DEFAULT '[]',\n                notice TEXT\n            )",
        )?;

        conn.pragma_update(None, "user_version", 27)
            .db_context("Failed to set user_version")?;
    }

    if due(28) {
        // The Snapshot tables go, and so do the Games the older code saved: their
        // Messages hold no kind and their Swipes no state and no maker, which the new
        // records require. The two tables are rebuilt empty, because SQLite cannot
        // add `NOT NULL` to a column in place.
        exec("DELETE FROM game_docks")?;
        exec("DROP TABLE IF EXISTS message_swipes")?;
        exec("DROP TABLE IF EXISTS messages")?;
        exec("DROP TABLE IF EXISTS game_state_snapshots")?;
        exec("DELETE FROM games")?;

        exec(
            "CREATE TABLE messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL DEFAULT 1 REFERENCES games(id) ON DELETE CASCADE,
                message_type TEXT NOT NULL,
                kind TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                active_swipe_index INTEGER NOT NULL DEFAULT 0
            )",
        )?;
        exec("CREATE INDEX IF NOT EXISTS idx_messages_game_id ON messages(game_id, id)")?;
        exec(
            "CREATE TABLE message_swipes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message_id INTEGER NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
                swipe_index INTEGER NOT NULL,
                text TEXT NOT NULL,
                room_report TEXT,
                event_header TEXT,
                impersonated INTEGER NOT NULL DEFAULT 0,
                guide_text TEXT,
                impersonate_direction TEXT,
                state TEXT NOT NULL,
                maker TEXT NOT NULL,
                UNIQUE(message_id, swipe_index)
            )",
        )?;

        conn.pragma_update(None, "user_version", 28)
            .db_context("Failed to set user_version")?;
    }

    if due(29) {
        // The story log derives the room it shows from each Swipe's state, so the
        // stored room name has no reader.
        if column_exists(conn, "message_swipes", "room_report") {
            exec("ALTER TABLE message_swipes DROP COLUMN room_report")?;
        }

        conn.pragma_update(None, "user_version", 29)
            .db_context("Failed to set user_version")?;
    }

    if due(30) {
        // The trigger context of a Swipe state changed from a Trigger reference to the
        // whole event record, so the Games the older code saved cannot be read.
        exec("DELETE FROM game_docks")?;
        exec("DELETE FROM message_swipes")?;
        exec("DELETE FROM messages")?;
        exec("DELETE FROM games")?;

        conn.pragma_update(None, "user_version", 30)
            .db_context("Failed to set user_version")?;
    }

    tx.commit().db_context("Failed to commit migrations")?;

    Ok(())
}
