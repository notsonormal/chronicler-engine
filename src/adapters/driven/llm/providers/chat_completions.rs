//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! The chat-completions provider: one backend for every OpenAI-compatible host.

use crate::adapters::driven::llm::transport::{
    call_chat_completions, ChatCompletionRequest, ChatCompletionResult, ChatCompletionsEndpoint,
    ProviderRouting,
};
use crate::application::ports::llm_provider::{LlmCallResult, LlmProvider};
use crate::application::prompting::prompt_merge::merge_single_user_message;
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::{LlmConnection, ReasoningEffort};
use crate::error::EngineError;

#[derive(Clone)]
pub struct ChatCompletionsBackend {
    provider: LlmBackendType,
    /// The session header value: one session per connection.
    connection_id: String,
    base_url: String,
    api_key: Option<String>,
    model: String,
    single_user_message: bool,
    reasoning_effort: Option<ReasoningEffort>,
    /// Empty when the provider has no routing.
    provider_order: Vec<String>,
    pin_provider_order: bool,
}

impl ChatCompletionsBackend {
    pub fn from_config(connection: &LlmConnection) -> Self {
        Self {
            provider: connection.provider,
            // The form test sends a connection that has no id yet.
            connection_id: if connection.id.is_empty() {
                "unsaved-connection".to_string()
            } else {
                connection.id.clone()
            },
            base_url: connection.resolve_base_url(),
            // A provider without a key env var takes no key, even a stored one.
            api_key: connection
                .provider
                .api_key_env_var()
                .and(connection.resolve_api_key()),
            model: connection.model.clone(),
            single_user_message: connection.single_user_message,
            reasoning_effort: connection.reasoning_effort,
            provider_order: if connection.provider.supports_routing() {
                connection.provider_order.clone()
            } else {
                Vec::new()
            },
            pin_provider_order: connection.pin_provider_order,
        }
    }

    fn call(
        &self,
        system_prompt: &str,
        user_text: &str,
        max_tokens: Option<u32>,
    ) -> Result<ChatCompletionResult, EngineError> {
        let (system, user) = if self.single_user_message {
            ("", merge_single_user_message(system_prompt, user_text))
        } else {
            (system_prompt, user_text.to_string())
        };
        let user = self.close_gemma4_thinking_channel(user);
        call_chat_completions(
            &ChatCompletionsEndpoint {
                base_url: &self.base_url,
                api_key: self.api_key.as_deref(),
                title: self.provider.attribution_title(),
                session: self
                    .provider
                    .session_header()
                    .map(|name| (name, self.connection_id.as_str())),
            },
            &ChatCompletionRequest {
                model: &self.model,
                system_prompt: system,
                user_text: &user,
                max_tokens,
                reasoning_effort: self.reasoning_effort,
                provider_routing: (!self.provider_order.is_empty()).then(|| ProviderRouting {
                    order: &self.provider_order,
                    allow_fallbacks: !self.pin_provider_order,
                }),
            },
        )
    }

    /// Ollama runs a `{{ .Prompt }}` passthrough template, so on Ollama this
    /// backend owns turn structure. Gemma 4 can loop in its thinking channel
    /// until it spends `max_tokens`; the closed channel stops that.
    /// `sanitize_llm_output` removes any marker that still leaks.
    fn close_gemma4_thinking_channel(&self, text: String) -> String {
        let model = self.model.to_lowercase();
        let is_gemma4 = model.contains("gemma-4") || model.contains("gemma4");
        if self.provider == LlmBackendType::Ollama && is_gemma4 {
            format!("{text}\n<|turn>model\n<|channel>thought\n<channel|>")
        } else {
            text
        }
    }
}

impl LlmProvider for ChatCompletionsBackend {
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
        self.provider.display_label()
    }
}
