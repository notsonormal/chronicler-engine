//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]
//! LLM message persistence port

use chrono::{DateTime, Utc};

use crate::error::EngineError;

#[derive(Debug, Clone)]
pub struct LlmMessage {
    pub id: i64,
    pub agent_name: String,
    pub backend_name: String,
    pub model_name: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub raw_request_json: String,
    pub raw_response_json: String,
    pub parsed_response: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub trait LlmMessageRepository: Send + Sync {
    fn save_llm_message(&self, message: &LlmMessage) -> Result<(), EngineError>;
    fn list_latest_llm_messages(&self, limit: usize) -> Result<Vec<LlmMessage>, EngineError>;
}
