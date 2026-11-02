//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! The chat-completions provider: one backend for every OpenAI-compatible host.

use uuid::Uuid;

use crate::adapters::driven::llm::transport::{
    call_chat_completions, ChatCompletionRequest, ChatCompletionResult, ChatCompletionsEndpoint,
    ProviderRouting,
};
use crate::application::ports::llm_provider::{LlmCallResult, LlmProvider};
use crate::application::prompting::prompt_merge::merge_single_user_message;
use crate::domain::model::game::GameId;
use crate::domain::model::generation_parameters::CallSettings;
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::{LlmConnection, ReasoningEffort};
use crate::error::EngineError;

/// Fixed, so a Game keeps the same session id across restarts.
const GAME_SESSION_NAMESPACE: Uuid = Uuid::from_u128(0x6368_726f_6e69_636c_6572_2d67_616d_6531);

#[derive(Clone)]
pub struct ChatCompletionsBackend {
    provider: LlmBackendType,
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
        agent_name: &str,
        system_prompt: &str,
        user_text: &str,
        settings: &CallSettings,
    ) -> Result<ChatCompletionResult, EngineError> {
        let (system, user) = if self.single_user_message {
            ("", merge_single_user_message(system_prompt, user_text))
        } else {
            (system_prompt, user_text.to_string())
        };
        let user = self.close_gemma4_thinking_channel(user);
        let session_id = settings
            .game
            .filter(|_| self.provider.sends_game_session())
            .map(Self::game_session);
        let session_header = self
            .provider
            .session_header()
            .map(|name| (name, self.session_header_value(agent_name, settings.game)));
        call_chat_completions(
            &ChatCompletionsEndpoint {
                base_url: &self.base_url,
                api_key: self.api_key.as_deref(),
                title: self.provider.attribution_title(),
                session: session_header
                    .as_ref()
                    .map(|(name, value)| (*name, value.as_str())),
            },
            &ChatCompletionRequest {
                model: &self.model,
                system_prompt: system,
                user_text: &user,
                call: settings,
                session_id: session_id.as_deref(),
                reasoning_effort: self.reasoning_effort,
                provider_routing: (!self.provider_order.is_empty()).then(|| ProviderRouting {
                    order: &self.provider_order,
                    allow_fallbacks: !self.pin_provider_order,
                }),
            },
        )
    }

    /// One session for each Game and Role, so that the large prompts of one
    /// Role or Game do not push the cached prompts of another out of the
    /// provider cache.
    fn session_header_value(&self, agent_name: &str, game: Option<GameId>) -> String {
        let owner = game.map_or_else(|| self.connection_id.clone(), Self::game_session);
        format!("{owner}:{agent_name}")
    }

    fn game_session(game: GameId) -> String {
        Uuid::new_v5(&GAME_SESSION_NAMESPACE, &game.0.to_be_bytes()).to_string()
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
        settings: &CallSettings,
    ) -> Result<LlmCallResult, EngineError> {
        tracing::info!("[LLM] Generating action from prompt");
        let chat = self.call(agent_name, system_prompt, user_prompt, settings)?;
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
