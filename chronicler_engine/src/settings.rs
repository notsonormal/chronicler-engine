//! [DOC: docs/system/dashboard.md]
//! Application settings and configuration

use std::path::PathBuf;

use crate::error::Result;
use crate::domain::model::settings::AppSettings;
use crate::storage::Storage;

const SETTINGS_FILENAME: &str = "settings.json";

/// Returns the default settings file path (data/settings.json).
/// Note: Server loads settings from SQLite DB only; this is for CLI import tooling.
pub fn get_settings_path() -> PathBuf {
    PathBuf::from("data").join(SETTINGS_FILENAME)
}

/// [DOC: docs/architecture/system.md]
/// Loads settings from DB (singleton row, id=1).
/// Falls back to defaults if row doesn't exist.
pub fn load_settings(storage: &Storage) -> Result<AppSettings> {
    match storage.get_settings() {
        Ok(settings) => Ok(settings),
        Err(e) => {
            tracing::warn!("Failed to load settings from DB, using defaults: {}", e);
            Ok(AppSettings::default())
        }
    }
}

impl AppSettings {
    /// Saves settings to DB (singleton row, id=1).
    pub fn save(&self, storage: &Storage) -> Result<()> {
        storage.save_settings(self)
    }
}
