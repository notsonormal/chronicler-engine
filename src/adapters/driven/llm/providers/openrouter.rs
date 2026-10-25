//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! OpenRouter LLM provider

use crate::error::EngineError;
use crate::domain::model::settings::{LlmConnection, ReasoningEffort};
use crate::adapters::driven::llm::transport::{call_chat_completions, ChatCompletionRequest};

use crate::application::ports::llm_provider::{LlmCallResult, LlmProvider};
use crate::application::prompting::prompt_merge::merge_single_user_message;

#[derive(Clone, Default)]
pub struct OpenRouterBackend {
    base_url: String,
    api_key: String,
    model: String,
    single_user_message: bool,
    reasoning_effort: Option<ReasoningEffort>,
}

impl OpenRouterBackend {
    pub fn from_config(connection: &LlmConnection) -> Self {
        let api_key = connection.resolve_api_key().unwrap_or_default();
        Self {
            base_url: connection.resolve_base_url(),
            api_key,
            model: connection.model.clone(),
            single_user_message: connection.single_user_message,
            reasoning_effort: connection.reasoning_effort,
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
        call_chat_completions(
            &self.base_url,
            Some(&self.api_key),
            Some("Chronicler Engine"),
            &ChatCompletionRequest {
                model: &self.model,
                system_prompt: system,
                user_text: &user,
                max_tokens,
                reasoning_effort: self.reasoning_effort,
            },
        )
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
