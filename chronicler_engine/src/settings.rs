//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Application settings and configuration

use std::path::PathBuf;

use crate::error::Result;
use crate::domain::model::settings::AppSettings;
use crate::adapters::driven::storage::Storage;

const SETTINGS_FILENAME: &str = "settings.json";

/// Note: Server loads settings from SQLite DB only; this is for CLI import tooling.
pub fn get_settings_path() -> PathBuf {
    PathBuf::from("data").join(SETTINGS_FILENAME)
}

pub fn load_settings(storage: &Storage) -> Result<AppSettings> {
    match storage.get_settings() {
        Ok(settings) => Ok(settings),
        Err(e) => {
            tracing::warn!("Failed to load settings from DB, using defaults: {}", e);
            Ok(AppSettings::default())
        }
    }
}

pub fn save_settings(settings: &AppSettings, storage: &Storage) -> Result<()> {
    storage.save_settings(settings)
}
