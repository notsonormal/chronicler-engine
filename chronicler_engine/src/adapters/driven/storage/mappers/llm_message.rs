//! [DOC: docs/system/storage.md]
//! LLM message mapper

use chrono::{DateTime, Utc};

use crate::error::EngineError;
use crate::domain::model::llm_message::LlmMessage;
use crate::adapters::driven::storage::models::llm_message::DbLlmMessage;

impl TryFrom<&DbLlmMessage> for LlmMessage {
    type Error = EngineError;

    fn try_from(db: &DbLlmMessage) -> Result<Self, Self::Error> {
        let created_at = DateTime::parse_from_rfc3339(&db.created_at)
            .map_err(|e| {
                EngineError::Config(format!("Failed to parse llm_message created_at: {e}"))
            })?
            .with_timezone(&Utc);

        Ok(LlmMessage {
            id: db.id,
            agent_name: db.agent_name.clone(),
            backend_name: db.backend_name.clone(),
            model_name: db.model_name.clone(),
            system_prompt: db.system_prompt.clone(),
            user_prompt: db.user_prompt.clone(),
            raw_request_json: db.raw_request_json.clone(),
            raw_response_json: db.raw_response_json.clone(),
            parsed_response: db.parsed_response.clone(),
            error_message: db.error_message.clone(),
            created_at,
        })
    }
}

impl From<&LlmMessage> for DbLlmMessage {
    fn from(msg: &LlmMessage) -> Self {
        Self {
            id: msg.id,
            agent_name: msg.agent_name.clone(),
            backend_name: msg.backend_name.clone(),
            model_name: msg.model_name.clone(),
            system_prompt: msg.system_prompt.clone(),
            user_prompt: msg.user_prompt.clone(),
            raw_request_json: msg.raw_request_json.clone(),
            raw_response_json: msg.raw_response_json.clone(),
            parsed_response: msg.parsed_response.clone(),
            error_message: msg.error_message.clone(),
            created_at: msg.created_at.to_rfc3339(),
        }
    }
}
