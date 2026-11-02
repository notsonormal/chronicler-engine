//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! LLM provider port (transport-only)

use crate::domain::model::agent::Role;
use crate::domain::model::generation_parameters::CallSettings;
use crate::domain::model::llm_message::LlmMessage;
use crate::domain::model::prompt_part::PromptPart;
use crate::error::EngineError;

pub const AGENT_NARRATOR: &str = Role::Narrator.agent_name();
pub const AGENT_QUANTIFIER: &str = Role::Quantifier.agent_name();
pub const AGENT_TRIGGER: &str = Role::Trigger.agent_name();
pub const AGENT_DIALOGUE: &str = "dialogue";
pub const AGENT_OPTIONS: &str = Role::Options.agent_name();

#[derive(Debug)]
pub struct LlmCallResult {
    pub text: String,
    pub raw_request_json: String,
    pub raw_response_json: String,
    pub backend_name: String,
    pub model_name: String,
    pub agent_name: String,
}

/// `raw_request_json` is empty when the failure came before any send.
#[derive(Debug)]
pub struct LlmCallFailure {
    pub error: EngineError,
    pub raw_request_json: String,
}

impl std::fmt::Display for LlmCallFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

impl From<EngineError> for LlmCallFailure {
    fn from(error: EngineError) -> Self {
        Self {
            error,
            raw_request_json: String::new(),
        }
    }
}

impl LlmCallResult {
    /// `raw_request_json` is the one record of what was sent: a new row writes
    /// no `system_prompt` and no `user_prompt`.
    pub fn to_message(&self) -> LlmMessage {
        LlmMessage {
            id: 0,
            agent_name: self.agent_name.clone(),
            backend_name: self.backend_name.clone(),
            model_name: self.model_name.clone(),
            system_prompt: String::new(),
            user_prompt: String::new(),
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
        prompt: &[PromptPart],
        settings: &CallSettings,
    ) -> Result<LlmCallResult, LlmCallFailure>;
}
