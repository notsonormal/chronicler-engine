//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Application settings and configuration

use crate::adapters::driven::storage::Storage;
use crate::domain::model::settings::AppSettings;

/// Falls back to built-in defaults; a malformed seed file fails the boot.
pub fn load_settings(storage: &Storage) -> AppSettings {
    match storage.get_settings() {
        Ok(settings) => settings,
        Err(e) => {
            tracing::warn!("Failed to load settings from DB, using defaults: {}", e);
            AppSettings::default()
        }
    }
}
