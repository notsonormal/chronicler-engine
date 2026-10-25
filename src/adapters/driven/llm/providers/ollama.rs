//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! Ollama LLM provider

use crate::error::EngineError;
use crate::domain::model::settings::{LlmConnection, ReasoningEffort};
use crate::adapters::driven::llm::transport::{call_chat_completions, ChatCompletionRequest};

use crate::application::ports::llm_provider::{LlmCallResult, LlmProvider};
use crate::application::prompting::prompt_merge::merge_single_user_message;

#[derive(Clone, Default)]
pub struct OllamaBackend {
    base_url: String,
    model: String,
    single_user_message: bool,
    reasoning_effort: Option<ReasoningEffort>,
}

impl OllamaBackend {
    pub fn from_config(connection: &LlmConnection) -> Self {
        Self {
            base_url: connection.resolve_base_url(),
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
        let user = if self.single_user_message {
            merge_single_user_message(system_prompt, user_text)
        } else {
            user_text.to_string()
        };
        let user = self.preprocess_user_text(&user);
        call_chat_completions(
            &self.base_url,
            None,
            None,
            &ChatCompletionRequest {
                model: &self.model,
                system_prompt,
                user_text: &user,
                max_tokens,
                reasoning_effort: self.reasoning_effort,
            },
        )
    }

    /// Ollama runs a `{{ .Prompt }}` passthrough template, so this backend owns
    /// turn structure. Gemma 4 can loop in its thinking channel until it spends
    /// `max_tokens` and returns empty content; the closed channel stops that.
    /// `sanitize_llm_output` removes any marker that still leaks.
    pub fn preprocess_user_text(&self, text: &str) -> String {
        let m = self.model.to_lowercase();
        if m.contains("gemma-4") || m.contains("gemma4") {
            format!("{text}\n<|turn>model\n<|channel>thought\n<channel|>")
        } else {
            text.to_string()
        }
    }
}

impl LlmProvider for OllamaBackend {
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
        "Ollama"
    }
}
