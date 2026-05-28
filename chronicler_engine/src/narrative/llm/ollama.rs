use std::sync::Arc;

use crate::error::{EngineError, LlmFailure};
use crate::model::settings::Connection;
use crate::narrative::llm_client::call_ollama;
use crate::storage::Storage;

use super::backend::{LlmBackend, LlmCallResult, merge_single_user_message};

#[derive(Clone, Default)]
pub struct OllamaBackend {
    base_url: String,
    model: String,
    single_user_message: bool,
    storage: Option<Arc<Storage>>,
}

impl OllamaBackend {
    pub fn from_connection(connection: &Connection, storage: Option<Arc<Storage>>) -> Self {
        Self {
            base_url: connection.resolve_base_url(),
            model: connection.model.clone(),
            single_user_message: connection.single_user_message,
            storage,
        }
    }

    fn call(
        &self,
        system_prompt: &str,
        user_text: &str,
        max_tokens: Option<u32>,
    ) -> Result<crate::narrative::llm_client::ChatCompletionResult, EngineError> {
        let (system, user) = if self.single_user_message {
            ("", merge_single_user_message(system_prompt, user_text))
        } else {
            (system_prompt, user_text.to_string())
        };
        let result = call_ollama(&self.base_url, &self.model, system, &user, max_tokens)?;
        if result.text.trim().is_empty() {
            return Err(EngineError::Llm(LlmFailure::EmptyResponse));
        }
        Ok(result)
    }
}

impl LlmBackend for OllamaBackend {
    fn model(&self) -> &str {
        &self.model
    }

    fn narrate_continuation(
        &self,
        agent_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        _trigger_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        log::info!("[LLM] Generating continuation narration");
        Ok(self.wrap_and_save(
            agent_name,
            self.call(system_prompt, user_prompt, max_tokens)?,
        ))
    }

    fn complete(
        &self,
        agent_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        log::info!("[LLM] Generating action from prompt");
        Ok(self.wrap_and_save(
            agent_name,
            self.call(system_prompt, user_prompt, max_tokens)?,
        ))
    }

    fn name(&self) -> &str {
        "Ollama"
    }

    fn save_message(&self, message: &crate::model::llm_message::LlmMessage) {
        if let Some(storage) = &self.storage {
            let _ = storage.save_llm_message(message);
        }
    }
}
