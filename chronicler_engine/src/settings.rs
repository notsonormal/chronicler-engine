//! [DOC: docs/system/dashboard.md]
//! Application settings and configuration

use std::path::PathBuf;

use crate::error::Result;
use crate::model::settings::AppSettings;
use crate::storage::Storage;

const SETTINGS_FILENAME: &str = "settings.json";

/// Backward compat. Settings are DB-backed since Phase 2.
pub fn get_settings_path() -> PathBuf {
    if let Ok(path) = std::env::var("CHRONICLER_SETTINGS_PATH") {
        return PathBuf::from(path);
    }
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
