//! [DOC: docs/diataxis/reference/storage.md]
//! Settings storage operations

use chrono::Utc;

use crate::adapters::driven::storage::models::settings::DbSettings;
use crate::adapters::driven::storage::{Backend, Storage};
use crate::domain::model::settings::AppSettings;
use crate::error::EngineError;

/// [TRIVIAL_ENUM]
#[derive(Clone, Copy)]
enum SettingsWrite {
    Replace,
    OnlyIfAbsent,
}

impl Storage {
    /// Read the singleton settings row from an already-locked backend.
    ///
    /// Split from `get_settings` so the read-modify-write path can reuse it
    /// without taking the backend lock a second time.
    fn read_settings(backend: &mut Backend) -> Result<AppSettings, EngineError> {
        match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let mut stmt = conn.prepare(
                "SELECT id, connections, narration_connection_id, quantifier_connection_id, response_length, text_check, agents, mode_preset_registry, created_at, updated_at, active_options_prompt_preset_id FROM settings WHERE id = 1",
            )?;
                let result = stmt.query_row([], DbSettings::from_row);
                match result {
                    Ok(db_settings) => db_settings.to_settings(),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AppSettings::default()),
                    Err(e) => Err(EngineError::Database(e)),
                }
            }
            Backend::InMemory(data) => Ok(data.settings.clone()),
        }
    }

    /// Write the singleton settings row into an already-locked backend.
    fn write_settings(
        backend: &mut Backend,
        settings: &AppSettings,
        mode: SettingsWrite,
    ) -> Result<(), EngineError> {
        match backend {
            Backend::Sqlite { pool } => {
                let conn = pool.conn();
                let now = Utc::now().to_rfc3339();
                let connections_json =
                    serde_json::to_string(&settings.connections).map_err(|e| {
                        EngineError::Parse(format!("Failed to serialize connections: {e}"))
                    })?;
                let text_check_json = serde_json::to_string(&settings.text_check).map_err(|e| {
                    EngineError::Parse(format!("Failed to serialize text_check: {e}"))
                })?;
                let agents_json = serde_json::to_string(&settings.agents)
                    .map_err(|e| EngineError::Parse(format!("Failed to serialize agents: {e}")))?;
                let mode_preset_registry_json =
                    serde_json::to_string(&settings.mode_preset_registry).map_err(|e| {
                        EngineError::Parse(format!("Failed to serialize mode_preset_registry: {e}"))
                    })?;

                let verb = match mode {
                    SettingsWrite::Replace => "REPLACE",
                    SettingsWrite::OnlyIfAbsent => "IGNORE",
                };
                conn.execute(
                    &format!(
                        "INSERT OR {verb} INTO settings (id, connections, narration_connection_id, quantifier_connection_id, response_length, text_check, agents, mode_preset_registry, created_at, updated_at, active_options_prompt_preset_id)
                         VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
                    ),
                    rusqlite::params![
                        &connections_json,
                        &settings.narration_connection_id,
                        &settings.quantifier_connection_id,
                        &settings.response_length,
                        &text_check_json,
                        &agents_json,
                        &mode_preset_registry_json,
                        &now,
                        &now,
                        &settings.active_options_prompt_preset_id,
                    ],
                )?;
                Ok(())
            }
            Backend::InMemory(data) => {
                if matches!(mode, SettingsWrite::OnlyIfAbsent) && data.settings_seeded {
                    return Ok(());
                }
                data.settings = settings.clone();
                data.settings_seeded = true;
                Ok(())
            }
        }
    }

    pub fn get_settings(&self) -> Result<AppSettings, EngineError> {
        self.with_backend_mut("get_settings", Self::read_settings)
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), EngineError> {
        self.with_backend_mut("save_settings", |backend| {
            Self::write_settings(backend, settings, SettingsWrite::Replace)
        })
    }

    /// Apply `f` to the settings row and persist the result under one lock.
    ///
    /// Read-modify-write under a single backend acquisition keeps concurrent
    /// edits atomic; an `Err` from `f` aborts before any write.
    pub fn update_settings<T, F>(&self, f: F) -> Result<T, EngineError>
    where
        F: FnOnce(&mut AppSettings) -> Result<T, EngineError>,
    {
        self.with_backend_mut("update_settings", |backend| {
            let mut settings = Self::read_settings(backend)?;
            let out = f(&mut settings)?;
            Self::write_settings(backend, &settings, SettingsWrite::Replace)?;
            Ok(out)
        })
    }

    /// Insert the settings row only when it is absent — the boot seed, so after
    /// the first run the database outranks `data/settings.json`.
    pub fn seed_settings(&self, settings: &AppSettings) -> Result<(), EngineError> {
        self.with_backend_mut("seed_settings", |backend| {
            Self::write_settings(backend, settings, SettingsWrite::OnlyIfAbsent)
        })
    }
}
