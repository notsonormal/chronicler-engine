//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]
//! OpenRouter LLM provider

use crate::error::{EngineError, LlmFailure};
use crate::domain::model::settings::LlmProviderConfig;
use crate::adapters::driven::llm::transport::call_openrouter_with_model;

use crate::application::ports::llm_provider::{LlmCallResult, LlmProvider};
use crate::application::prompting::prompt_merge::merge_single_user_message;

#[derive(Clone, Default)]
pub struct OpenRouterBackend {
    api_key: String,
    model: String,
    single_user_message: bool,
}

impl OpenRouterBackend {
    pub fn from_config(connection: &LlmProviderConfig) -> Self {
        let api_key = connection.resolve_api_key().unwrap_or_default();
        Self {
            api_key,
            model: connection.model.clone(),
            single_user_message: connection.single_user_message,
        }
    }

    fn call(
        &self,
        system_prompt: &str,
        user_text: &str,
        max_tokens: Option<u32>,
    ) -> Result<crate::adapters::driven::llm::transport::ChatCompletionResult, EngineError> {
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

impl LlmProvider for OpenRouterBackend {
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
        let chat = self.call(system_prompt, user_prompt, max_tokens)?;
        Ok(LlmCallResult {
            text: chat.text,
            raw_request_json: chat.raw_request_json,
            raw_response_json: chat.raw_response_json,
            backend_name: self.name().to_string(),
            model_name: self.model().to_string(),
            agent_name: agent_name.to_string(),
        })
    }

    fn name(&self) -> &str {
        "OpenRouter"
    }
}
