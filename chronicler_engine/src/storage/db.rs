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

    if version < 9 {
        let exec = |sql: &str| {
            conn.execute(sql, [])
                .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))
        };

        exec(
            "CREATE TABLE IF NOT EXISTS games (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                world_name TEXT NOT NULL DEFAULT 'default',
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

        // Insert default game row only if none exists so re-opening a DB is idempotent.
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM games", [], |row| row.get(0))
            .unwrap_or(0);
        if count == 0 {
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO games (id, world_name, name, created_at, updated_at) VALUES (1, 'default', 'default', ?1, ?1)",
                rusqlite::params![&now],
            )
            .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;
        }

        conn.pragma_update(None, "user_version", 9).map_err(|e| {
            crate::error::EngineError::Config(format!("Failed to set user_version: {e}"))
        })?;
    }

    // Template for future migrations:
    // if version < 10 { ...; conn.pragma_update(None, "user_version", 10)?; }

    Ok(())
}
