use std::sync::Arc;

use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::settings::Connection;
use crate::storage::llm_message_storage::LlmMessageStorage;

use super::backend::{LlmBackend, LlmCallResult};

#[derive(Clone, Default)]
#[allow(dead_code)]
pub struct DeepSeekBackend {
    api_key: String,
    model: String,
    max_context_tokens: u32,
    storage: Option<Arc<dyn LlmMessageStorage>>,
}

impl DeepSeekBackend {
    pub fn from_connection(
        connection: &Connection,
        storage: Option<Arc<dyn LlmMessageStorage>>,
    ) -> Self {
        let api_key = connection.resolve_api_key().unwrap_or_default();
        Self {
            api_key,
            model: connection.model.clone(),
            max_context_tokens: connection.resolve_max_context_tokens(),
            storage,
        }
    }

    fn not_implemented() -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Config(
            "DeepSeek backend is not yet implemented. Configure an OpenRouter or Ollama connection."
                .into(),
        ))
    }
}

impl LlmBackend for DeepSeekBackend {
    fn model(&self) -> &str {
        &self.model
    }

    fn name(&self) -> &str {
        "DeepSeek"
    }

    fn save_message(&self, message: &crate::model::llm_message::LlmMessage) {
        if let Some(storage) = &self.storage {
            let _ = storage.save(message);
        }
    }

    fn generate_dialogue(
        &self,
        _agent_name: &str,
        _context: &crate::narrative::prompt::PromptContext,
        _npc: &NpcCard,
    ) -> Result<LlmCallResult, EngineError> {
        Self::not_implemented()
    }

    fn narrate_action(
        &self,
        _agent_name: &str,
        _context: &crate::narrative::prompt::PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Self::not_implemented()
    }

    fn narrate_arrival(
        &self,
        _agent_name: &str,
        _context: &crate::narrative::prompt::PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Self::not_implemented()
    }

    fn narrate_continuation(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _trigger_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Self::not_implemented()
    }

    fn complete(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Self::not_implemented()
    }
}
