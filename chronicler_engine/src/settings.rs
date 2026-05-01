use std::path::PathBuf;

use crate::error::{EngineError, Result};
use crate::model::settings::AppSettings;

const SETTINGS_FILENAME: &str = "settings.json";

pub fn get_settings_path() -> PathBuf {
    PathBuf::from("data").join(SETTINGS_FILENAME)
}

pub fn load_settings() -> Result<AppSettings> {
    let path = get_settings_path();
    if !path.exists() {
        let defaults = AppSettings::default();
        defaults.save()?;
        return Ok(defaults);
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| EngineError::Io(format!("Failed to read settings: {e}")))?;
    serde_json::from_str(&content)
        .map_err(|e| EngineError::Parse(format!("Failed to parse settings: {e}")))
}

impl AppSettings {
    pub fn save(&self) -> Result<()> {
        let path = get_settings_path();
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| EngineError::Serialize(format!("Failed to serialize settings: {e}")))?;
        std::fs::write(&path, content)
            .map_err(|e| EngineError::Io(format!("Failed to write settings: {e}")))?;
        Ok(())
    }
}
