use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::settings::Connection;

use super::backend::LlmBackend;

#[derive(Clone, Default)]
#[allow(dead_code)]
pub struct DeepSeekBackend {
    api_key: String,
    model: String,
    max_context_tokens: u32,
}

impl DeepSeekBackend {
    pub fn from_connection(connection: &Connection) -> Self {
        let api_key = connection.resolve_api_key().unwrap_or_default();
        Self {
            api_key,
            model: connection.model.clone(),
            max_context_tokens: connection.resolve_max_context_tokens(),
        }
    }
}

impl LlmBackend for DeepSeekBackend {
    fn generate_dialogue(
        &self,
        _context: &crate::narrative::prompt::PromptContext,
        _npc: &NpcCard,
    ) -> Result<String, EngineError> {
        Err(EngineError::Config(
            "DeepSeek backend is not yet implemented. Configure an OpenRouter or Ollama connection."
                .into(),
        ))
    }

    fn narrate_action(
        &self,
        _context: &crate::narrative::prompt::PromptContext,
    ) -> Result<String, EngineError> {
        Err(EngineError::Config(
            "DeepSeek backend is not yet implemented. Configure an OpenRouter or Ollama connection."
                .into(),
        ))
    }

    fn narrate_arrival(
        &self,
        _context: &crate::narrative::prompt::PromptContext,
    ) -> Result<String, EngineError> {
        Err(EngineError::Config(
            "DeepSeek backend is not yet implemented. Configure an OpenRouter or Ollama connection."
                .into(),
        ))
    }

    fn narrate_continuation(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
        _trigger_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<String, EngineError> {
        Err(EngineError::Config(
            "DeepSeek backend is not yet implemented. Configure an OpenRouter or Ollama connection."
                .into(),
        ))
    }

    fn narrate_action_from_prompt(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<String, EngineError> {
        Err(EngineError::Config(
            "DeepSeek backend is not yet implemented. Configure an OpenRouter or Ollama connection."
                .into(),
        ))
    }

    fn name(&self) -> &str {
        "DeepSeek"
    }
}
