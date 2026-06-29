//! [DOC: docs/system/llm_processing.md]
//! Ollama LLM provider

use std::sync::Arc;

use crate::error::{EngineError, LlmFailure};
use crate::domain::model::settings::Connection;
use crate::adapters::driven::llm::transport::call_ollama;
use crate::adapters::driven::storage::Storage;

use crate::application::ports::llm_provider::{LlmBackend, LlmCallResult, merge_single_user_message};

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
    ) -> Result<crate::adapters::driven::llm::transport::ChatCompletionResult, EngineError> {
        let user = if self.single_user_message {
            merge_single_user_message(system_prompt, user_text)
        } else {
            user_text.to_string()
        };
        let user = self.preprocess_user_text(&user);
        let result = call_ollama(
            &self.base_url,
            &self.model,
            system_prompt,
            &user,
            max_tokens,
        )?;
        if result.text.trim().is_empty() {
            return Err(EngineError::Llm(LlmFailure::EmptyResponse));
        }
        Ok(result)
    }

    pub fn preprocess_user_text(&self, text: &str) -> String {
        let m = self.model.to_lowercase();
        if m.contains("gemma-4") || m.contains("gemma4") {
            format!("{text}\n<|turn>model\n<|channel>thought\n<channel|>")
        } else {
            text.to_string()
        }
    }
}

impl LlmBackend for OllamaBackend {
    fn model(&self) -> &str {
        &self.model
    }

    fn complete(
        &self,
        agent_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        tracing::info!("[LLM] Generating action from prompt");
        Ok(self.wrap_and_save(
            agent_name,
            self.call(system_prompt, user_prompt, max_tokens)?,
        ))
    }

    fn name(&self) -> &str {
        "Ollama"
    }

    fn storage(&self) -> Option<&Arc<Storage>> {
        self.storage.as_ref()
    }
}
