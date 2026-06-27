//! [DOC: docs/system/llm_processing.md]
//! OpenRouter LLM provider

use std::sync::Arc;

use crate::error::{EngineError, LlmFailure};
use crate::model::settings::Connection;
use crate::narrative::llm_client::call_openrouter_with_model;
use crate::storage::Storage;

use super::backend::{LlmBackend, LlmCallResult, merge_single_user_message};

#[derive(Clone, Default)]
pub struct OpenRouterBackend {
    api_key: String,
    model: String,
    single_user_message: bool,
    storage: Option<Arc<Storage>>,
}

impl OpenRouterBackend {
    pub fn from_connection(connection: &Connection, storage: Option<Arc<Storage>>) -> Self {
        let api_key = connection.resolve_api_key().unwrap_or_default();
        Self {
            api_key,
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
        let result =
            call_openrouter_with_model(&self.api_key, system, &user, &self.model, max_tokens)?;
        if result.text.trim().is_empty() {
            return Err(EngineError::Llm(LlmFailure::EmptyResponse));
        }
        Ok(result)
    }
}

impl LlmBackend for OpenRouterBackend {
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
        "OpenRouter"
    }

    fn storage(&self) -> Option<&Arc<Storage>> {
        self.storage.as_ref()
    }
}
