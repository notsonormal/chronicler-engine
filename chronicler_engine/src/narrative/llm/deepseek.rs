//! [DOC: docs/system/llm_processing.md]
//! DeepSeek LLM provider

use std::sync::Arc;

use crate::error::EngineError;
use crate::model::settings::Connection;
use crate::storage::Storage;

use super::backend::{LlmBackend, LlmCallResult};

#[derive(Clone, Default)]
pub struct DeepSeekBackend {
    #[allow(dead_code)] // Stored for future implementation
    api_key: String,
    model: String,
    storage: Option<Arc<Storage>>,
}

impl DeepSeekBackend {
    pub fn from_connection(connection: &Connection, storage: Option<Arc<Storage>>) -> Self {
        let api_key = connection.resolve_api_key().unwrap_or_default();
        Self {
            api_key,
            model: connection.model.clone(),
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

    fn storage(&self) -> Option<&Arc<Storage>> {
        self.storage.as_ref()
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
