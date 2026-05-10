use std::sync::Mutex;

use rusqlite::Connection;

pub struct DbPool {
    conn: Mutex<Connection>,
}

impl DbPool {
    pub fn new(path: &str) -> Result<Self, crate::error::EngineError> {
        let conn = Connection::open(path)
            .map_err(|e| crate::error::EngineError::Config(format!("Failed to open DB: {e}")))?;
        run_migrations(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
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
            message_id TEXT NOT NULL,
            swipe_index INTEGER NOT NULL DEFAULT 0,
            movement TEXT NOT NULL,
            narrative TEXT NOT NULL,
            scene TEXT NOT NULL,
            character_state TEXT NOT NULL,
            committed INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            UNIQUE(message_id, swipe_index)
        )",
        [],
    )
    .map_err(|e| crate::error::EngineError::Config(format!("Migration failed: {e}")))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_snapshots_message ON game_state_snapshots(message_id, swipe_index)",
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

    Ok(())
}
