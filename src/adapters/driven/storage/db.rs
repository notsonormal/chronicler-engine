//! [DOC: docs/diataxis/reference/storage.md]
//! SQLite database connection pool

use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::adapters::driven::storage::utils::run_migrations;
use crate::error::DatabaseContext;

#[derive(Clone)]
pub struct DbPool {
    conn: Arc<Mutex<Connection>>,
}

impl DbPool {
    pub fn new(path: &str) -> Result<Self, crate::error::EngineError> {
        let conn = Connection::open(path).db_context("Failed to open DB")?;
        run_migrations(&conn)?;
        conn.pragma_update(None, "foreign_keys", true)
            .db_context("Failed to enable foreign keys")?;
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
