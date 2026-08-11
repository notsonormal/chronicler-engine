//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! DeepSeek LLM provider

use crate::error::EngineError;
use crate::domain::model::settings::LlmProviderConfig;

use crate::application::ports::llm_provider::{LlmProvider, LlmCallResult};

#[derive(Clone, Default)]
pub struct DeepSeekBackend {
    #[allow(dead_code)] // Stored for future implementation
    api_key: String,
    model: String,
}

impl DeepSeekBackend {
    pub fn from_config(connection: &LlmProviderConfig) -> Self {
        let api_key = connection.resolve_api_key().unwrap_or_default();
        Self {
            api_key,
            model: connection.model.clone(),
        }
    }

    fn not_implemented() -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Config(
            "DeepSeek backend is not yet implemented. Configure an OpenRouter or Ollama connection."
                .into(),
        ))
    }
}

impl LlmProvider for DeepSeekBackend {
    fn model(&self) -> &str {
        &self.model
    }

    fn name(&self) -> &str {
        "DeepSeek"
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
