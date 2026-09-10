//! [DOC: docs/diataxis/reference/storage.md]
//! Storage-layer plumbing utilities (datetime parsing, schema migrations).

use rusqlite::Connection;

use crate::error::EngineError;

pub(crate) fn parse_datetime(
    rfc3339: &str,
    field: &str,
) -> Result<chrono::DateTime<chrono::Utc>, EngineError> {
    chrono::DateTime::parse_from_rfc3339(rfc3339)
        .map_err(|e| EngineError::Config(format!("Invalid {field}: {e}")))
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

pub(crate) fn run_migrations(conn: &Connection) -> Result<(), EngineError> {
    fn column_exists(conn: &Connection, table: &str, col: &str) -> bool {
        conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info(?) WHERE name = ?",
            [table, col],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0)
            > 0
    }

    let version: i32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap_or(0);

    if version < 9 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))
        };

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
            .map_err(|e| EngineError::Config(format!("Failed to set user_version: {e}")))?;
    }

    if version < 10 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))
        };

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
                player_key TEXT NOT NULL DEFAULT '',  -- dropped in v13
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
            .map_err(|e| EngineError::Config(format!("Failed to set user_version: {e}")))?;
    }

    if version < 12 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))
        };

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
            .map_err(|e| EngineError::Config(format!("Failed to set user_version: {e}")))?;
    }

    if version < 13 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))
        };

        exec("ALTER TABLE games ADD COLUMN persona_key TEXT NOT NULL DEFAULT ''")?;
        exec("ALTER TABLE games ADD COLUMN persona_name TEXT NOT NULL DEFAULT ''")?;
        exec("ALTER TABLE worlds DROP COLUMN player_key")?;
        conn.pragma_update(None, "user_version", 13)
            .map_err(|e| EngineError::Config(format!("Failed to set user_version: {e}")))?;
    }

    if version < 14 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))
        };
        if column_exists(conn, "worlds", "starting_room_id") {
            exec("ALTER TABLE worlds DROP COLUMN starting_room_id")?;
        }
        conn.pragma_update(None, "user_version", 14)
            .map_err(|e| EngineError::Config(format!("Failed to set user_version: {e}")))?;
    }

    if version < 15 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))
        };
        if !column_exists(conn, "message_swipes", "replay") {
            exec("ALTER TABLE message_swipes ADD COLUMN replay TEXT")?;
        }
        conn.pragma_update(None, "user_version", 15)
            .map_err(|e| EngineError::Config(format!("Failed to set user_version: {e}")))?;
    }

    if version < 16 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))
        };
        if !column_exists(conn, "settings", "active_impersonate_prompt_preset_id") {
            exec(
                "ALTER TABLE settings ADD COLUMN active_impersonate_prompt_preset_id \
                 TEXT NOT NULL DEFAULT 'impersonate_default'",
            )?;
        }
        conn.pragma_update(None, "user_version", 16)
            .map_err(|e| EngineError::Config(format!("Failed to set user_version: {e}")))?;
    }

    if version < 17 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))
        };
        if column_exists(conn, "messages", "sender") {
            exec("ALTER TABLE messages DROP COLUMN sender")?;
        }
        conn.pragma_update(None, "user_version", 17)
            .map_err(|e| EngineError::Config(format!("Failed to set user_version: {e}")))?;
    }

    if version < 18 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))
        };
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
            .map_err(|e| EngineError::Config(format!("Failed to set user_version: {e}")))?;
    }

    if version < 19 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))
        };

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

        // Games inherit posture from their world (the new source of truth).
        // COALESCE keeps the migration alive for a game whose world_key has
        // no matching world row — the subquery would otherwise produce NULL
        // and fail the NOT NULL constraint, aborting startup.
        exec(
            "UPDATE games SET \
               narrator_mode = COALESCE((SELECT narrator_mode FROM worlds WHERE worlds.key = games.world_key), 'novel'), \
               narrative_perspective = COALESCE((SELECT narrative_perspective FROM worlds WHERE worlds.key = games.world_key), 'third'), \
               narrative_tense = COALESCE((SELECT narrative_tense FROM worlds WHERE worlds.key = games.world_key), 'past')",
        )?;

        // The COALESCE fallback above silently repairs orphaned games; these
        // warnings keep the data-integrity signal visible in the logs.
        let orphans: Vec<String> = {
            let mut stmt = conn
                .prepare(
                    "SELECT name FROM games \
                     WHERE world_key NOT IN (SELECT key FROM worlds)",
                )
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))?;
            let rows = stmt
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))?
        };
        for key in &orphans {
            tracing::warn!(
                "migration v19: game '{key}' references world_key with no matching world row; \
                 posture defaulted to novel/third/past"
            );
        }

        // Mode-preset registry column on settings. Backfill from the legacy
        // per-type columns before they are dropped below.
        if !column_exists(conn, "settings", "mode_preset_registry") {
            exec(
                "ALTER TABLE settings ADD COLUMN mode_preset_registry TEXT NOT NULL DEFAULT '{}'",
            )?;
            // Fresh default for rows that took the '{}' default (no legacy columns).
            exec(
                "UPDATE settings SET mode_preset_registry = '{\"novel\":{\"system_prompt_preset_id\":\"system_default\",\"quantifier_prompt_preset_id\":\"quantifier_default\",\"impersonate_prompt_preset_id\":\"impersonate_default\"},\"interactive_fiction\":{\"system_prompt_preset_id\":\"system_if_default\",\"quantifier_prompt_preset_id\":\"quantifier_default\",\"impersonate_prompt_preset_id\":\"impersonate_default\"}}' \
               WHERE mode_preset_registry = '{}'",
            )?;
            // Migrate legacy per-type columns into the Novel bundle, where present.
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
            // IF bundle keeps quantifier/impersonate in step with the migrated Novel values.
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
            .map_err(|e| EngineError::Config(format!("Failed to set user_version: {e}")))?;
    }

    if version < 20 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))
        };

        // Per-preset mode-allow flags on the presets table. The column is
        // born all-allowed on every existing row — no user value can
        // pre-exist a column at birth.
        if !column_exists(conn, "prompt_presets", "allowed_modes") {
            exec(
                "ALTER TABLE prompt_presets ADD COLUMN allowed_modes \
                 TEXT NOT NULL DEFAULT '[\"novel\",\"interactive_fiction\"]'",
            )?;
        }
        // One-time tighten of the novel system seed; every other row keeps
        // its all-allowed birth value.
        exec(
            "UPDATE prompt_presets SET allowed_modes = '[\"novel\"]' \
             WHERE id = 'system_default'",
        )?;

        // Reshape the settings registry from the v19 object-keyed JSON to the
        // mode-tagged list, preserving user-customized bundle values. The RHS
        // is evaluated against the pre-update column value (SQLite semantics).
        // The json_type guard keeps a partially applied block safe to re-run.
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
            .map_err(|e| EngineError::Config(format!("Failed to set user_version: {e}")))?;
    }

    if version < 21 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))
        };

        // Per-game posture is dead data: the prompt assembler stamps world
        // posture as the sole owner, and no code reads the per-game copies.
        if column_exists(conn, "games", "narrative_perspective") {
            exec("ALTER TABLE games DROP COLUMN narrative_perspective")?;
        }
        if column_exists(conn, "games", "narrative_tense") {
            exec("ALTER TABLE games DROP COLUMN narrative_tense")?;
        }

        conn.pragma_update(None, "user_version", 21)
            .map_err(|e| EngineError::Config(format!("Failed to set user_version: {e}")))?;
    }

    if version < 22 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))
        };

        // Swipe generation inputs flatten from the v15 replay blob into two direct
        // columns. A corrupt blob degraded the turn to a plain retry at parse time,
        // so the backfill skips non-JSON values. Producers never set
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
            .map_err(|e| EngineError::Config(format!("Failed to set user_version: {e}")))?;
    }

    if version < 23 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))
        };

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

        let orphans: Vec<String> = {
            let mut stmt = conn
                .prepare(
                    "SELECT name FROM games \
                     WHERE world_key NOT IN (SELECT key FROM worlds)",
                )
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))?;
            let rows = stmt
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))?
        };
        for key in &orphans {
            tracing::warn!(
                "migration v23: game '{key}' references world_key with no matching world row; \
                 posture defaulted to third/past"
            );
        }

        conn.pragma_update(None, "user_version", 23)
            .map_err(|e| EngineError::Config(format!("Failed to set user_version: {e}")))?;
    }

    if version < 24 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))
        };

        // Options autogeneration, on the v19/v23 posture pattern: world-default
        // toggle with per-game override. The options preset id is mode-agnostic;
        // games inherit a copy from settings at creation.
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

        // Games inherit the toggle from their world (mirrors v19/v23 backfill).
        exec(
            "UPDATE games SET \
               options_always_on = COALESCE((SELECT options_always_on FROM worlds WHERE worlds.key = games.world_key), 0)",
        )?;

        let orphans: Vec<String> = {
            let mut stmt = conn
                .prepare(
                    "SELECT name FROM games \
                     WHERE world_key NOT IN (SELECT key FROM worlds)",
                )
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))?;
            let rows = stmt
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| EngineError::Config(format!("Migration failed: {e}")))?
        };
        for key in &orphans {
            tracing::warn!(
                "migration v24: game '{key}' references world_key with no matching world row; \
                 options_always_on defaulted to off"
            );
        }

        conn.pragma_update(None, "user_version", 24)
            .map_err(|e| EngineError::Config(format!("Failed to set user_version: {e}")))?;
    }

    Ok(())
}
