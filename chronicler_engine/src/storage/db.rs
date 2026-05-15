use std::sync::{Arc, Mutex};

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
    conn.execute(
        "CREATE TABLE IF NOT EXISTS game_state_snapshots (
            id TEXT PRIMARY KEY,
            turn_id TEXT NOT NULL,
            swipe_index INTEGER NOT NULL DEFAULT 0,
            movement TEXT NOT NULL,
            narrative TEXT NOT NULL,
            scene TEXT NOT NULL,
            character_state TEXT NOT NULL,
            committed INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            UNIQUE(turn_id, swipe_index)
        )",
        [],
    )
    .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_snapshots_turn ON game_state_snapshots(turn_id, swipe_index)",
        [],
    )
    .map_err(|e| {
        crate::error::EngineError::Config(format!("Migration failed: {e}"))
    })?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_snapshots_latest ON game_state_snapshots(created_at DESC)",
        [],
    )
    .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS checkpoints (
            id TEXT PRIMARY KEY,
            turn_id TEXT NOT NULL,
            swipe_index INTEGER NOT NULL DEFAULT 0,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_checkpoints_turn ON checkpoints(turn_id, swipe_index)",
        [],
    )
    .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

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

    Ok(())
}
