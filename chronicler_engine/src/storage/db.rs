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

    Ok(())
}
