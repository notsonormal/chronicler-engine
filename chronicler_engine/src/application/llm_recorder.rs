//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]
//! LLM call orchestrator - owns forensics save + postprocessing

use std::sync::Arc;

use crate::error::EngineError;
use crate::application::ports::llm_provider::{LlmProvider, LlmCallResult};
use crate::application::ports::llm_message_repository::LlmMessageRepository;
use crate::application::llm_sanitizer::sanitize_llm_output;

pub struct LlmCallRecorder {
    provider: Arc<dyn LlmProvider>,
    forensics: Arc<dyn LlmMessageRepository>,
}

impl LlmCallRecorder {
    pub fn new(provider: Arc<dyn LlmProvider>, forensics: Arc<dyn LlmMessageRepository>) -> Self {
        Self {
            provider,
            forensics,
        }
    }

    pub fn complete(
        &self,
        agent_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        // 1. Call the provider (pure transport)
        let result = self
            .provider
            .complete(agent_name, system_prompt, user_prompt, max_tokens)?;

        // 2. Postprocess response text (sanitization)
        let sanitized_text = sanitize_llm_output(&result.text);

        // 3. Build message with prompts from args + sanitized text, save to forensics
        let mut message = result.to_message(system_prompt, user_prompt);
        message.parsed_response = sanitized_text.clone();
        self.forensics.save_llm_message(&message)?;

        // 4. Return result with sanitized text
        let mut sanitized_result = result;
        sanitized_result.text = sanitized_text;
        Ok(sanitized_result)
    }

    pub fn provider(&self) -> &Arc<dyn LlmProvider> {
        &self.provider
    }
}
