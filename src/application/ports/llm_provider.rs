//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! LLM provider port (transport-only)

use crate::error::EngineError;
use crate::domain::model::llm_message::LlmMessage;

pub const AGENT_NARRATOR: &str = "narrator";
pub const AGENT_QUANTIFIER: &str = "quantifier";
pub const AGENT_TRIGGER: &str = "trigger";
pub const AGENT_DIALOGUE: &str = "dialogue";

#[derive(Debug)]
pub struct LlmCallResult {
    pub text: String,
    pub raw_request_json: String,
    pub raw_response_json: String,
    pub backend_name: String,
    pub model_name: String,
    pub agent_name: String,
}

impl LlmCallResult {
    /// Build forensic message from result + original prompts.
    /// Prompts are not echoed in the transport result — caller supplies them.
    pub fn to_message(&self, system_prompt: &str, user_prompt: &str) -> LlmMessage {
        LlmMessage {
            id: 0,
            agent_name: self.agent_name.clone(),
            backend_name: self.backend_name.clone(),
            model_name: self.model_name.clone(),
            system_prompt: system_prompt.to_string(),
            user_prompt: user_prompt.to_string(),
            raw_request_json: self.raw_request_json.clone(),
            raw_response_json: self.raw_response_json.clone(),
            parsed_response: self.text.clone(),
            error_message: None,
            created_at: chrono::Utc::now(),
        }
    }
}

pub trait LlmProvider: Send + Sync {
    fn model(&self) -> &str;
    fn name(&self) -> &str;

    fn complete(
        &self,
        agent_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError>;
}
