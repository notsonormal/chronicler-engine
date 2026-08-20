//! [DOC: docs/diataxis/reference/storage.md]
//! Settings database model

use crate::error::EngineError;
use crate::domain::model::agent::AgentConfig;
use crate::domain::model::settings::{AppSettings, LlmProviderConfig, TextCheckSettings};

/// Database row for `settings` table (singleton, id=1).
pub struct DbSettings {
    pub id: i64,
    pub connections: String, // JSON: Vec<LlmProviderConfig>
    pub narration_connection_id: String,
    pub quantifier_connection_id: String,
    pub response_length: String,
    pub text_check: String, // JSON: TextCheckSettings
    pub agents: String,     // JSON: Vec<AgentConfig>
    pub active_system_prompt_preset_id: String,
    pub active_quantifier_prompt_preset_id: String,
    pub active_impersonate_prompt_preset_id: String,
    pub created_at: String,
    pub updated_at: String,
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
            active_system_prompt_preset_id: row.get(7)?,
            active_quantifier_prompt_preset_id: row.get(8)?,
            active_impersonate_prompt_preset_id: row.get(11)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    }

    pub(crate) fn to_settings(&self) -> Result<AppSettings, EngineError> {
        let connections: Vec<LlmProviderConfig> = serde_json::from_str(&self.connections)
            .map_err(|e| EngineError::Parse(format!("Failed to deserialize connections: {e}")))?;
        let text_check: TextCheckSettings = serde_json::from_str(&self.text_check)
            .map_err(|e| EngineError::Parse(format!("Failed to deserialize text_check: {e}")))?;
        let agents: Vec<AgentConfig> = serde_json::from_str(&self.agents)
            .map_err(|e| EngineError::Parse(format!("Failed to deserialize agents: {e}")))?;

        Ok(AppSettings {
            connections,
            narration_connection_id: self.narration_connection_id.clone(),
            quantifier_connection_id: self.quantifier_connection_id.clone(),
            response_length: self.response_length.clone(),
            text_check,
            agents,
            active_system_prompt_preset_id: self.active_system_prompt_preset_id.clone(),
            active_quantifier_prompt_preset_id: self.active_quantifier_prompt_preset_id.clone(),
            active_impersonate_prompt_preset_id: self.active_impersonate_prompt_preset_id.clone(),
        })
    }
}
