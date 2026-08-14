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
                    "SELECT id, connections, narration_connection_id, quantifier_connection_id, response_length, text_check, agents, active_system_prompt_preset_id, active_quantifier_prompt_preset_id, created_at, updated_at FROM settings WHERE id = 1",
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
                    "INSERT OR REPLACE INTO settings (id, connections, narration_connection_id, quantifier_connection_id, response_length, text_check, agents, active_system_prompt_preset_id, active_quantifier_prompt_preset_id, created_at, updated_at)
                     VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
        // For settings, seed is the same as save (singleton row)
        self.save_settings(settings)
    }
}

impl DbSettings {
    fn to_settings(&self) -> Result<AppSettings, EngineError> {
        let connections: Vec<crate::domain::model::settings::LlmProviderConfig> =
            serde_json::from_str(&self.connections).map_err(|e| {
                EngineError::Parse(format!("Failed to deserialize connections: {e}"))
            })?;
        let text_check: crate::domain::model::settings::TextCheckSettings =
            serde_json::from_str(&self.text_check).map_err(|e| {
                EngineError::Parse(format!("Failed to deserialize text_check: {e}"))
            })?;
        let agents: Vec<crate::domain::model::agent::AgentConfig> =
            serde_json::from_str(&self.agents)
                .map_err(|e| EngineError::Parse(format!("Failed to deserialize agents: {e}")))?;

        Ok(AppSettings {
            connections,
            narration_connection_id: self.narration_connection_id.clone(),
            quantifier_connection_id: self.quantifier_connection_id.clone(),
            response_length: self.response_length.clone(),
            text_check,
            agents,
            active_system_prompt_preset_id: self.active_system_prompt_preset_id.clone(),
            active_quantifier_prompt_preset_id: self.active_quantifier_prompt_preset_id.clone(),
        })
    }
}
