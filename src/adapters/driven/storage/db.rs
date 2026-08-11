//! [DOC: docs/diataxis/reference/storage.md]
//! SQLite database connection pool

use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::adapters::driven::storage::utils::run_migrations;

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

    /// Insert a new games row and return the new rowid.
    /// Single source of truth for the `games` INSERT column list.
    pub fn insert_game(
        &self,
        world_name: &str,
        world_key: &str,
        persona_key: &str,
        persona_name: &str,
        name: &str,
    ) -> Result<u64, crate::error::EngineError> {
        let conn = self.conn();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO games (world_name, world_key, persona_key, persona_name, name, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            rusqlite::params![world_name, world_key, persona_key, persona_name, name, &now],
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Failed to create game: {e}")))?;
        Ok(conn.last_insert_rowid() as u64)
    }
}
