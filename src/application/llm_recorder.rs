//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! LLM call orchestrator - owns forensics save + postprocessing

use std::sync::Arc;

use crate::application::ports::llm_provider::{LlmCallResult, LlmProvider};
use crate::application::prompting::sanitize::sanitize_llm_output;
use crate::domain::model::llm_message::LlmMessage;
use crate::error::EngineError;

/// Resolves the provider for one call, so a dashboard change to the narrator or
/// quantifier connection takes effect on the next call without a restart.
pub type ProviderResolver =
    Arc<dyn Fn() -> Result<Arc<dyn LlmProvider>, EngineError> + Send + Sync>;

/// Persists one forensics record; injected so a test can capture or fail a save.
pub type SaveLlmMessageFn = Arc<dyn Fn(&LlmMessage) -> Result<(), EngineError> + Send + Sync>;

pub struct LlmCallRecorder {
    resolve: ProviderResolver,
    save_fn: SaveLlmMessageFn,
}

impl LlmCallRecorder {
    /// Test and fixture constructor: the provider never changes.
    pub fn new(provider: Arc<dyn LlmProvider>, save_fn: SaveLlmMessageFn) -> Self {
        Self {
            resolve: Arc::new(move || Ok(Arc::clone(&provider))),
            save_fn,
        }
    }

    /// The production constructor: `resolve` runs inside `complete`, so a
    /// dangling connection id surfaces as a call error.
    pub fn with_resolver(resolve: ProviderResolver, save_fn: SaveLlmMessageFn) -> Self {
        Self { resolve, save_fn }
    }

    pub fn provider(&self) -> Result<Arc<dyn LlmProvider>, EngineError> {
        (self.resolve)()
    }

    /// One log label, or a placeholder when the connection cannot be resolved.
    pub fn provider_label(&self) -> String {
        match self.provider() {
            Ok(provider) => format!("{} {}", provider.name(), provider.model()),
            Err(e) => format!("<unresolved: {e}>"),
        }
    }

    pub fn complete(
        &self,
        agent_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        let provider = self.provider()?;
        let result = match provider.complete(agent_name, system_prompt, user_prompt, max_tokens) {
            Ok(result) => result,
            Err(error) => {
                let failure = Self::failed_message(
                    agent_name,
                    provider.name(),
                    provider.model(),
                    system_prompt,
                    user_prompt,
                    &error,
                );
                if let Err(save_error) = (*self.save_fn)(&failure) {
                    tracing::warn!(
                        "[LLM] failed to record the failed attempt for agent '{agent_name}': {save_error}"
                    );
                }
                return Err(error);
            }
        };

        let sanitized_text = sanitize_llm_output(&result.text);

        let mut message = result.to_message(system_prompt, user_prompt);
        message.parsed_response = sanitized_text.clone();
        (*self.save_fn)(&message)?;

        let mut sanitized_result = result;
        sanitized_result.text = sanitized_text;
        Ok(sanitized_result)
    }

    /// The transport returns the request payload only on success, so a failed row
    /// blanks `raw_request_json`.
    fn failed_message(
        agent_name: &str,
        backend_name: &str,
        model_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        error: &EngineError,
    ) -> LlmMessage {
        let raw_response_json = error.raw_response_body().unwrap_or_default().to_string();
        LlmMessage {
            id: 0,
            agent_name: agent_name.to_string(),
            backend_name: backend_name.to_string(),
            model_name: model_name.to_string(),
            system_prompt: system_prompt.to_string(),
            user_prompt: user_prompt.to_string(),
            raw_request_json: String::new(),
            raw_response_json,
            parsed_response: String::new(),
            error_message: Some(error.llm_error_string()),
            created_at: chrono::Utc::now(),
        }
    }
}
