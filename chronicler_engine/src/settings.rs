use std::path::PathBuf;

use crate::error::{EngineError, Result};
use crate::model::settings::AppSettings;

const SETTINGS_FILENAME: &str = "settings.json";

pub fn get_settings_path() -> PathBuf {
    if let Ok(path) = std::env::var("CHRONICLER_SETTINGS_PATH") {
        return PathBuf::from(path);
    }
    PathBuf::from("data").join(SETTINGS_FILENAME)
}

/// [DOC: docs/architecture/system.md]
pub fn load_settings() -> Result<AppSettings> {
    let path = get_settings_path();
    if !path.exists() {
        let defaults = AppSettings::default();
        defaults.save()?;
        return Ok(defaults);
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| EngineError::Io(format!("Failed to read settings: {e}")))?;

    let mut settings: AppSettings = serde_json::from_str::<AppSettings>(&content)
        .map_err(|e| EngineError::Parse(format!("Failed to parse settings: {e}")))?;

    inject_env_overrides(&mut settings);
    Ok(settings)
}

/// Env resolution happens at load time.
fn inject_env_overrides(settings: &mut AppSettings) {
    for conn in &mut settings.connections {
        if conn.api_key.is_none() {
            match conn.provider {
                crate::model::llm_backend::LlmBackendType::OpenRouter
                | crate::model::llm_backend::LlmBackendType::DeepSeek => {
                    if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
                        conn.api_key = Some(key);
                    }
                }
                crate::model::llm_backend::LlmBackendType::Ollama
                | crate::model::llm_backend::LlmBackendType::Mock => {}
            }
        }
        if conn.base_url.is_none() {
            match conn.provider {
                crate::model::llm_backend::LlmBackendType::Ollama => {
                    if let Ok(url) = std::env::var("OLLAMA_BASE_URL") {
                        conn.base_url = Some(url);
                    }
                }
                crate::model::llm_backend::LlmBackendType::OpenRouter
                | crate::model::llm_backend::LlmBackendType::DeepSeek
                | crate::model::llm_backend::LlmBackendType::Mock => {}
            }
        }
    }
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
