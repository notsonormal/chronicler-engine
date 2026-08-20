//! [DOC: docs/diataxis/reference/storage.md]
//! Settings storage operations

use chrono::Utc;

use crate::error::EngineError;
use crate::domain::model::settings::AppSettings;
use crate::adapters::driven::storage::{Backend, Storage};
use crate::adapters::driven::storage::models::settings::DbSettings;

impl Storage {
    pub fn get_settings(&self) -> Result<AppSettings, EngineError> {
        self.with_backend_mut("get_settings", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn.prepare(
                    "SELECT id, connections, narration_connection_id, quantifier_connection_id, response_length, text_check, agents, active_system_prompt_preset_id, active_quantifier_prompt_preset_id, created_at, updated_at, active_impersonate_prompt_preset_id FROM settings WHERE id = 1",
                )?;
                let result = stmt.query_row([], DbSettings::from_row);
                match result {
                    Ok(db_settings) => db_settings.to_settings(),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AppSettings::default()),
                    Err(e) => Err(EngineError::Database(e))
                }
            }
            Backend::InMemory(data) => Ok(data.settings.clone()),
        })
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), EngineError> {
        self.with_backend_mut("save_settings", |backend| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let now = Utc::now().to_rfc3339();
                let connections_json = serde_json::to_string(&settings.connections)
                    .map_err(|e| EngineError::Parse(format!("Failed to serialize connections: {e}")))?;
                let text_check_json = serde_json::to_string(&settings.text_check)
                    .map_err(|e| EngineError::Parse(format!("Failed to serialize text_check: {e}")))?;
                let agents_json = serde_json::to_string(&settings.agents)
                    .map_err(|e| EngineError::Parse(format!("Failed to serialize agents: {e}")))?;

                conn.execute(
                    "INSERT OR REPLACE INTO settings (id, connections, narration_connection_id, quantifier_connection_id, response_length, text_check, agents, active_system_prompt_preset_id, active_quantifier_prompt_preset_id, created_at, updated_at, active_impersonate_prompt_preset_id)
                     VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    rusqlite::params![
                        &connections_json,
                        &settings.narration_connection_id,
                        &settings.quantifier_connection_id,
                        &settings.response_length,
                        &text_check_json,
                        &agents_json,
                        &settings.active_system_prompt_preset_id,
                        &settings.active_quantifier_prompt_preset_id,
                        &now,
                        &now,
                        &settings.active_impersonate_prompt_preset_id,
                    ],
                )?;
                Ok(())
            }
            Backend::InMemory(data) => {
                data.settings = settings.clone();
                Ok(())
            }
        })
    }

    pub fn seed_settings(&self, settings: &AppSettings) -> Result<(), EngineError> {
        self.save_settings(settings)
    }
}
