use std::path::PathBuf;
use std::sync::RwLock;

use crate::error::Result;
use crate::model::settings::AppSettings;
use crate::storage::Storage;

const SETTINGS_FILENAME: &str = "settings.json";

// For backward compatibility with tests
pub fn get_settings_path() -> PathBuf {
    if let Ok(path) = std::env::var("CHRONICLER_SETTINGS_PATH") {
        return PathBuf::from(path);
    }
    PathBuf::from("data").join(SETTINGS_FILENAME)
}

// Global DB pool reference for settings operations
static DB_POOL: RwLock<Option<crate::storage::db::DbPool>> = RwLock::new(None);

/// Initialize the DB pool reference for settings operations.
/// Called during bootstrap/run.rs initialization.
/// For tests, pass an in-memory DB pool.
pub fn init_settings_db(pool: crate::storage::db::DbPool) {
    let mut guard = DB_POOL.write().unwrap_or_else(|e| e.into_inner());
    *guard = Some(pool);
}

/// Initialize settings with an in-memory DB (for testing).
/// [DOC: docs/architecture/system.md]
#[cfg(test)]
pub fn init_settings_in_memory() {
    if let Ok(pool) = crate::storage::db::DbPool::new(":memory:") {
        init_settings_db(pool);
    }
}

/// Get the DB pool reference.
fn get_db_pool() -> Option<crate::storage::db::DbPool> {
    let guard = DB_POOL.read().unwrap_or_else(|e| e.into_inner());
    guard.clone()
}

/// [DOC: docs/architecture/system.md]
/// Loads settings from DB (singleton row, id=1).
/// Falls back to defaults if row doesn't exist.
pub fn load_settings() -> Result<AppSettings> {
    if let Some(pool) = get_db_pool() {
        let storage = Storage::new_sqlite(pool, 1);
        match storage.get_settings() {
            Ok(settings) => return Ok(settings),
            Err(e) => {
                // Log but fall back to defaults
                tracing::warn!("Failed to load settings from DB, using defaults: {}", e);
            }
        }
    }

    // Fallback to default settings (for tests or early initialization)
    let defaults = AppSettings::default();
    // Auto-save defaults to DB if pool available
    if let Some(pool) = get_db_pool() {
        let storage = Storage::new_sqlite(pool, 1);
        if let Err(e) = storage.seed_settings(&defaults) {
            tracing::warn!("Failed to seed default settings: {}", e);
        }
    }
    Ok(defaults)
}

impl AppSettings {
    /// Saves settings to DB (singleton row, id=1).
    pub fn save(&self) -> Result<()> {
        if let Some(pool) = get_db_pool() {
            let storage = Storage::new_sqlite(pool, 1);
            storage.save_settings(self)?;
            Ok(())
        } else {
            // Fallback: no-op for tests or early initialization
            tracing::debug!("Settings save called but DB pool not initialized - ignoring");
            Ok(())
        }
    }
}
