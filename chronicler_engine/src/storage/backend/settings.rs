use chrono::Utc;

use crate::error::EngineError;
use crate::model::settings::AppSettings;
use crate::storage::backend::{Backend, Operation, Storage};
use crate::storage::models::settings::DbSettings;

impl Storage {
    pub fn get_settings(&self) -> Result<AppSettings, EngineError> {
        self.with_backend_mut(Operation::GetSettings, |backend, _game_id| match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn.prepare(
                    "SELECT id, connections, narration_connection_id, quantifier_connection_id, response_length, text_check, agents, active_system_prompt_preset_id, active_quantifier_prompt_preset_id FROM settings WHERE id = 1",
                )?;
                let row = stmt.query_row([], |row| {
                    Ok(DbSettings {
                        id: row.get(0)?,
                        connections: row.get(1)?,
                        narration_connection_id: row.get(2)?,
                        quantifier_connection_id: row.get(3)?,
                        response_length: row.get(4)?,
                        text_check: row.get(5)?,
                        agents: row.get(6)?,
                        active_system_prompt_preset_id: row.get(7)?,
                        active_quantifier_prompt_preset_id: row.get(8)?,
                        created_at: String::new(),
                        updated_at: String::new(),
                    })
                });
                match row {
                    Ok(db_settings) => settings_from_db(&db_settings),
                    Err(rusqlite::Error::QueryReturnedNoRows) => {
                        // Return default settings if row doesn't exist (shouldn't happen after seeding)
                        Ok(AppSettings::default())
                    }
                    Err(e) => Err(EngineError::Database(e.to_string())),
                }
            }
            Backend::InMemory(data) => Ok(data.settings.clone()),
            Backend::Test { .. } => unimplemented!(),
        })
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), EngineError> {
        self.with_backend_mut(Operation::SaveSettings, |backend, _game_id| match backend {
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
                    [
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
            Backend::Test { .. } => unimplemented!(),
        })
    }

    pub fn seed_settings(&self, settings: &AppSettings) -> Result<(), EngineError> {
        // For settings, seed is the same as save (singleton row)
        self.save_settings(settings)
    }
}

fn settings_from_db(db: &DbSettings) -> Result<AppSettings, EngineError> {
    let connections: Vec<crate::model::settings::Connection> =
        serde_json::from_str(&db.connections)
            .map_err(|e| EngineError::Parse(format!("Failed to deserialize connections: {e}")))?;
    let text_check: crate::model::settings::TextCheckSettings =
        serde_json::from_str(&db.text_check)
            .map_err(|e| EngineError::Parse(format!("Failed to deserialize text_check: {e}")))?;
    let agents: Vec<crate::model::agent::AgentConfig> = serde_json::from_str(&db.agents)
        .map_err(|e| EngineError::Parse(format!("Failed to deserialize agents: {e}")))?;

    Ok(AppSettings {
        connections,
        narration_connection_id: db.narration_connection_id.clone(),
        quantifier_connection_id: db.quantifier_connection_id.clone(),
        response_length: db.response_length.clone(),
        text_check,
        agents,
        active_system_prompt_preset_id: db.active_system_prompt_preset_id.clone(),
        active_quantifier_prompt_preset_id: db.active_quantifier_prompt_preset_id.clone(),
    })
}
