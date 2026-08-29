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

    pub fn insert_game_with_posture(
        &self,
        request: &crate::domain::model::game::NewGame,
    ) -> Result<u64, crate::error::EngineError> {
        let conn = self.conn();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO games (world_name, world_key, persona_key, persona_name, name, created_at, updated_at, narrator_mode, narrative_perspective, narrative_tense, active_system_prompt_preset_id, active_quantifier_prompt_preset_id, active_impersonate_prompt_preset_id) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                request.world_name,
                request.world_key,
                request.persona_key,
                request.persona_name,
                request.name,
                &now,
                request.narrator_mode.as_str(),
                request.narrative_perspective.as_str(),
                request.narrative_tense.as_str(),
                request.system_prompt_preset_id,
                request.quantifier_prompt_preset_id,
                request.impersonate_prompt_preset_id,
            ],
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Failed to create game: {e}")))?;
        Ok(conn.last_insert_rowid() as u64)
    }
}
