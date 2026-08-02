//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]
//! LLM call orchestrator - owns forensics save + postprocessing

use std::sync::Arc;

use crate::error::EngineError;
use crate::application::ports::llm_provider::{LlmProvider, LlmCallResult};
use crate::application::llm_message::SaveLlmMessageFn;
use crate::application::prompting::sanitize::sanitize_llm_output;

pub struct LlmCallRecorder {
    provider: Arc<dyn LlmProvider>,
    save_fn: SaveLlmMessageFn,
}

impl LlmCallRecorder {
    pub fn new(provider: Arc<dyn LlmProvider>, save_fn: SaveLlmMessageFn) -> Self {
        Self { provider, save_fn }
    }

    pub fn complete(
        &self,
        agent_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        let result = self
            .provider
            .complete(agent_name, system_prompt, user_prompt, max_tokens)?;

        let sanitized_text = sanitize_llm_output(&result.text);

        let mut message = result.to_message(system_prompt, user_prompt);
        message.parsed_response = sanitized_text.clone();
        (*self.save_fn)(&message)?;

        let mut sanitized_result = result;
        sanitized_result.text = sanitized_text;
        Ok(sanitized_result)
    }

    pub fn provider(&self) -> &Arc<dyn LlmProvider> {
        &self.provider
    }
}
