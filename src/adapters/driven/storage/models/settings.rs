//! [DOC: docs/diataxis/reference/storage.md]
//! Settings database model

use crate::error::EngineError;
use crate::domain::model::agent::AgentConfig;
use crate::domain::model::settings::{
    AppSettings, LlmProviderConfig, ModePresetRegistry, TextCheckSettings,
};

pub struct DbSettings {
    pub id: i64,
    pub connections: String, // JSON: Vec<LlmProviderConfig>
    pub narration_connection_id: String,
    pub quantifier_connection_id: String,
    pub response_length: String,
    pub text_check: String,           // JSON: TextCheckSettings
    pub agents: String,               // JSON: Vec<AgentConfig>
    pub mode_preset_registry: String, // JSON: ModePresetRegistry
    pub created_at: String,
    pub updated_at: String,
    pub active_options_prompt_preset_id: String,
}

impl DbSettings {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(DbSettings {
            id: row.get(0)?,
            connections: row.get(1)?,
            narration_connection_id: row.get(2)?,
            quantifier_connection_id: row.get(3)?,
            response_length: row.get(4)?,
            text_check: row.get(5)?,
            agents: row.get(6)?,
            mode_preset_registry: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
            active_options_prompt_preset_id: row.get(10)?,
        })
    }

    pub(crate) fn to_settings(&self) -> Result<AppSettings, EngineError> {
        let connections: Vec<LlmProviderConfig> = serde_json::from_str(&self.connections)
            .map_err(|e| EngineError::Parse(format!("Failed to deserialize connections: {e}")))?;
        let text_check: TextCheckSettings = serde_json::from_str(&self.text_check)
            .map_err(|e| EngineError::Parse(format!("Failed to deserialize text_check: {e}")))?;
        let agents: Vec<AgentConfig> = serde_json::from_str(&self.agents)
            .map_err(|e| EngineError::Parse(format!("Failed to deserialize agents: {e}")))?;
        let mode_preset_registry: ModePresetRegistry =
            serde_json::from_str(&self.mode_preset_registry).map_err(|e| {
                EngineError::Parse(format!("Failed to deserialize mode_preset_registry: {e}"))
            })?;

        Ok(AppSettings {
            connections,
            narration_connection_id: self.narration_connection_id.clone(),
            quantifier_connection_id: self.quantifier_connection_id.clone(),
            response_length: self.response_length.clone(),
            text_check,
            agents,
            mode_preset_registry,
            active_options_prompt_preset_id: self.active_options_prompt_preset_id.clone(),
        })
    }
}
