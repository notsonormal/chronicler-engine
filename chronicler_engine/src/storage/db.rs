use std::sync::{Arc, Mutex};

use chrono::Utc;
use rusqlite::Connection;

#[derive(Clone)]
pub struct DbPool {
    conn: Arc<Mutex<Connection>>,
}

impl DbPool {
    pub fn new(path: &str) -> Result<Self, crate::error::EngineError> {
        let conn = Connection::open(path)
            .map_err(|e| crate::error::EngineError::Config(format!("Failed to open DB: {e}")))?;
        run_migrations(&conn)?;
        conn.pragma_update(None, "foreign_keys", true)
            .map_err(|e| {
                crate::error::EngineError::Config(format!("Failed to enable foreign keys: {e}"))
            })?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn conn(&self) -> std::sync::MutexGuard<Connection> {
        match self.conn.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

fn run_migrations(conn: &Connection) -> Result<(), crate::error::EngineError> {
    let version: i32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap_or(0);

    if version < 1 {
        // Breaking change: drop old tables and recreate with the new schema.
        // Old saves are discarded once during the transition to integer IDs.
        let _ = conn.execute("DROP TABLE IF EXISTS game_state_snapshots", []);
        let _ = conn.execute("DROP TABLE IF EXISTS checkpoints", []);
        let _ = conn.execute("DROP TABLE IF EXISTS messages", []);
        let _ = conn.execute("DROP TABLE IF EXISTS games", []);

        conn.execute(
            "CREATE TABLE games (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                world_name TEXT NOT NULL DEFAULT 'default',
                name TEXT NOT NULL DEFAULT 'Unnamed',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

        conn.execute(
            "CREATE TABLE game_state_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL DEFAULT 1,
                movement TEXT NOT NULL,
                narrative TEXT NOT NULL,
                scene TEXT NOT NULL,
                npc_encounter_log TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

        conn.execute(
            "CREATE INDEX idx_snapshots_game_latest ON game_state_snapshots(game_id, created_at DESC)",
            [],
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

        conn.execute(
            "CREATE TABLE messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL DEFAULT 1,
                sender TEXT,
                text TEXT NOT NULL,
                log_type TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                location_header TEXT,
                event_header TEXT,
                snapshot_id INTEGER
            )",
            [],
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

        conn.execute(
            "CREATE INDEX idx_messages_game_id ON messages(game_id, id)",
            [],
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

        // Insert default game row so storage impls have something to reference
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO games (id, world_name, name, created_at, updated_at) VALUES (1, 'default', 'default', ?1, ?1)",
            rusqlite::params![&now],
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

        conn.pragma_update(None, "user_version", 1).map_err(|e| {
            crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
        })?;
    }

    if version < 2 {
        // Rename character_state column to npc_encounter_log for domain vocabulary alignment.
        // Foreign keys are not enforced by default, so ALTER TABLE is safe.
        let _ = conn.execute(
            "ALTER TABLE game_state_snapshots RENAME COLUMN character_state TO npc_encounter_log",
            [],
        );
        conn.pragma_update(None, "user_version", 2).map_err(|e| {
            crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
        })?;
    }

    // llm_messages is independent and uses IF NOT EXISTS so it is safe to rerun.
    conn.execute(
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
        [],
    )
    .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_llm_messages_created_at ON llm_messages(created_at DESC)",
        [],
    )
    .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

    if version < 3 {
        conn.execute(
            "CREATE TABLE prompt_presets (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                preset_type TEXT NOT NULL,
                prompt_text TEXT NOT NULL,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

        conn.execute(
            "CREATE INDEX idx_prompt_presets_type ON prompt_presets(preset_type)",
            [],
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

        conn.pragma_update(None, "user_version", 3).map_err(|e| {
            crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
        })?;
    }

    if version < 4 {
        let _ = conn.execute("ALTER TABLE game_state_snapshots DROP COLUMN committed", []);
        conn.pragma_update(None, "user_version", 4).map_err(|e| {
            crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
        })?;
    }

    if version < 5 {
        let _ = conn.execute(
            "ALTER TABLE games ADD COLUMN name TEXT NOT NULL DEFAULT 'Unnamed'",
            [],
        );
        let _ = conn.execute("DROP TABLE IF EXISTS checkpoints", []);
        let _ = conn.execute("DROP INDEX IF EXISTS idx_checkpoints_snapshot", []);
        conn.pragma_update(None, "user_version", 5).map_err(|e| {
            crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
        })?;
    }

    if version < 6 {
        conn.execute(
            "CREATE TABLE messages_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL DEFAULT 1,
                sender TEXT,
                log_type TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                active_swipe_index INTEGER NOT NULL DEFAULT 0,
                is_deleted INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Migration v6 failed: {e}")))?;

        conn.execute(
            "INSERT INTO messages_new (id, game_id, sender, log_type, timestamp, active_swipe_index, is_deleted)
             SELECT id, game_id, sender, log_type, timestamp, 0, 0 FROM messages",
            [],
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Migration v6 failed: {e}")))?;

        conn.execute(
            "CREATE TABLE message_swipes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message_id INTEGER NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
                swipe_index INTEGER NOT NULL,
                text TEXT NOT NULL,
                snapshot_id INTEGER,
                location_header TEXT,
                event_header TEXT,
                UNIQUE(message_id, swipe_index)
            )",
            [],
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Migration v6 failed: {e}")))?;

        // Migrate old message text into message_swipes BEFORE dropping the old messages table.
        conn.execute(
            "INSERT INTO message_swipes (message_id, swipe_index, text, snapshot_id, location_header, event_header)
             SELECT id, 0, text, snapshot_id, location_header, event_header FROM messages",
            [],
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Migration v6 failed: {e}")))?;

        conn.execute("DROP TABLE messages", [])
            .map_err(|e| crate::error::EngineError::Config(format!("Migration v6 failed: {e}")))?;
        conn.execute("ALTER TABLE messages_new RENAME TO messages", [])
            .map_err(|e| crate::error::EngineError::Config(format!("Migration v6 failed: {e}")))?;

        conn.pragma_update(None, "user_version", 6).map_err(|e| {
            crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
        })?;
    }

    fn merr(v: i32, e: impl std::fmt::Display) -> crate::error::EngineError {
        crate::error::EngineError::Config(format!("Migration v{v} failed: {e}"))
    }

    fn recreate_prompt_presets_table(
        conn: &Connection,
        keep_prompt_text: bool,
    ) -> Result<(), rusqlite::Error> {
        if keep_prompt_text {
            conn.execute(
                "CREATE TABLE prompt_presets_new (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    preset_type TEXT NOT NULL,
                    role TEXT,
                    instructions TEXT,
                    writing_style TEXT,
                    output_format TEXT,
                    prompt_text TEXT,
                    is_default INTEGER NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                )",
                [],
            )?;
            conn.execute(
                "INSERT INTO prompt_presets_new (id, name, preset_type, role, instructions, writing_style, output_format, prompt_text, is_default, created_at, updated_at)
                 SELECT id, name, preset_type, role, instructions, writing_style, output_format, prompt_text, is_default, created_at, updated_at FROM prompt_presets",
                [],
            )?;
        } else {
            conn.execute(
                "CREATE TABLE prompt_presets_new (
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
                [],
            )?;
            conn.execute(
                "INSERT INTO prompt_presets_new (id, name, preset_type, role, instructions, writing_style, output_format, is_default, created_at, updated_at)
                 SELECT id, name, preset_type, role, instructions, writing_style, output_format, is_default, created_at, updated_at FROM prompt_presets",
                [],
            )?;
        }
        conn.execute("DROP TABLE prompt_presets", [])?;
        conn.execute(
            "ALTER TABLE prompt_presets_new RENAME TO prompt_presets",
            [],
        )?;
        Ok(())
    }

    if version < 7 {
        conn.execute("BEGIN", []).map_err(|e| merr(7, e))?;
        // Idempotent: these columns may already exist if a partial migration ran.
        let _ = conn.execute("ALTER TABLE prompt_presets ADD COLUMN role TEXT", []);
        let _ = conn.execute(
            "ALTER TABLE prompt_presets ADD COLUMN instructions TEXT",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE prompt_presets ADD COLUMN writing_style TEXT",
            [],
        );
        let _ = conn.execute(
            "ALTER TABLE prompt_presets ADD COLUMN output_format TEXT",
            [],
        );
        recreate_prompt_presets_table(conn, true).map_err(|e| merr(7, e))?;
        conn.pragma_update(None, "user_version", 7)
            .map_err(|e| merr(7, e))?;
        conn.execute("COMMIT", []).map_err(|e| merr(7, e))?;
    }

    if version < 8 {
        conn.execute("BEGIN", []).map_err(|e| merr(8, e))?;
        recreate_prompt_presets_table(conn, false).map_err(|e| merr(8, e))?;
        conn.pragma_update(None, "user_version", 8)
            .map_err(|e| merr(8, e))?;
        conn.execute("COMMIT", []).map_err(|e| merr(8, e))?;
    }

    if version < 9 {
        // Add ON DELETE CASCADE foreign keys so GameStorage::delete_game
        // can delete a single games row and let SQLite clean up children.
        // Tables must be recreated because SQLite cannot add FK constraints
        // to existing columns via ALTER TABLE.
        conn.execute("BEGIN", []).map_err(|e| merr(9, e))?;

        // Recreate game_state_snapshots with FK to games
        conn.execute(
            "CREATE TABLE game_state_snapshots_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL DEFAULT 1 REFERENCES games(id) ON DELETE CASCADE,
                movement TEXT NOT NULL,
                narrative TEXT NOT NULL,
                scene TEXT NOT NULL,
                npc_encounter_log TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| merr(9, e))?;
        conn.execute(
            "INSERT INTO game_state_snapshots_new
             SELECT id, game_id, movement, narrative, scene, npc_encounter_log, created_at
             FROM game_state_snapshots",
            [],
        )
        .map_err(|e| merr(9, e))?;
        conn.execute("DROP TABLE game_state_snapshots", [])
            .map_err(|e| merr(9, e))?;
        conn.execute(
            "ALTER TABLE game_state_snapshots_new RENAME TO game_state_snapshots",
            [],
        )
        .map_err(|e| merr(9, e))?;

        // Recreate messages with FK to games
        conn.execute(
            "CREATE TABLE messages_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL DEFAULT 1 REFERENCES games(id) ON DELETE CASCADE,
                sender TEXT,
                log_type TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                active_swipe_index INTEGER NOT NULL DEFAULT 0,
                is_deleted INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )
        .map_err(|e| merr(9, e))?;
        conn.execute(
            "INSERT INTO messages_new
             SELECT id, game_id, sender, log_type, timestamp, active_swipe_index, is_deleted
             FROM messages",
            [],
        )
        .map_err(|e| merr(9, e))?;
        conn.execute("DROP TABLE messages", [])
            .map_err(|e| merr(9, e))?;
        conn.execute("ALTER TABLE messages_new RENAME TO messages", [])
            .map_err(|e| merr(9, e))?;

        // Recreate message_swipes (its FK to messages was dropped when messages was dropped)
        conn.execute(
            "CREATE TABLE message_swipes_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message_id INTEGER NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
                swipe_index INTEGER NOT NULL,
                text TEXT NOT NULL,
                snapshot_id INTEGER,
                location_header TEXT,
                event_header TEXT,
                UNIQUE(message_id, swipe_index)
            )",
            [],
        )
        .map_err(|e| merr(9, e))?;
        conn.execute(
            "INSERT INTO message_swipes_new
             SELECT id, message_id, swipe_index, text, snapshot_id, location_header, event_header
             FROM message_swipes",
            [],
        )
        .map_err(|e| merr(9, e))?;
        conn.execute("DROP TABLE message_swipes", [])
            .map_err(|e| merr(9, e))?;
        conn.execute(
            "ALTER TABLE message_swipes_new RENAME TO message_swipes",
            [],
        )
        .map_err(|e| merr(9, e))?;

        // Recreate indexes lost during table drops
        conn.execute(
            "CREATE INDEX idx_messages_game_id ON messages(game_id, id)",
            [],
        )
        .map_err(|e| merr(9, e))?;
        conn.execute(
            "CREATE INDEX idx_snapshots_game_latest ON game_state_snapshots(game_id, created_at DESC)",
            [],
        )
        .map_err(|e| merr(9, e))?;

        conn.pragma_update(None, "user_version", 9)
            .map_err(|e| merr(9, e))?;
        conn.execute("COMMIT", []).map_err(|e| merr(9, e))?;
    }

    Ok(())
}
