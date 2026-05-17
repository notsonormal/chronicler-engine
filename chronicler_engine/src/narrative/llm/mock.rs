use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use crate::error::{EngineError, NarrativeFailure};
use crate::model::character::NpcCard;
use crate::storage::llm_message_storage::LlmMessageStorage;

use super::backend::{LlmBackend, LlmCallResult};

#[derive(Default)]
pub struct MockBackend {
    /// If true, all narration methods return `Err(EngineError::Narrative(NarrativeFailure::Generation { stage: "mock", reason: "configured_failure" }))`.
    pub should_fail: AtomicBool,
    /// If true, `narrate_action` returns `Ok("")` - simulates an empty LLM response.
    pub should_return_empty: AtomicBool,
    /// If true, `complete` (trigger narration) returns `Err`.
    pub trigger_narration_should_fail: AtomicBool,
    /// Milliseconds to sleep in `narrate_action` to simulate a slow LLM.
    pub delay_ms: AtomicU64,
    /// Return different narration text per call (rotates).
    pub per_call_narrations: Vec<String>,
    /// Return different prompt responses per call (rotates). Used for quantifier/testing.
    pub per_call_prompt_responses: Vec<String>,
    pub call_index: AtomicUsize,
    pub storage: Option<Arc<dyn LlmMessageStorage>>,
}

impl MockBackend {
    pub fn new(storage: Option<Arc<dyn LlmMessageStorage>>) -> Self {
        Self {
            storage,
            ..Default::default()
        }
    }

    pub fn failing() -> Self {
        Self {
            should_fail: AtomicBool::new(true),
            ..Default::default()
        }
    }

    pub fn with_empty_response() -> Self {
        Self {
            should_return_empty: AtomicBool::new(true),
            ..Default::default()
        }
    }

    pub fn with_failing_trigger_narration() -> Self {
        Self {
            trigger_narration_should_fail: AtomicBool::new(true),
            ..Default::default()
        }
    }

    pub fn with_delay(ms: u64) -> Self {
        Self {
            delay_ms: AtomicU64::new(ms),
            ..Default::default()
        }
    }

    fn make_result(&self, agent_name: &str, text: impl Into<String>) -> LlmCallResult {
        let text = text.into();
        let result = LlmCallResult {
            text: text.clone(),
            system_prompt: String::new(),
            user_prompt: String::new(),
            raw_request_json: String::new(),
            raw_response_json: format!("{{\"content\":\"{text}\"}}"),
            backend_name: self.name().to_string(),
            model_name: self.model().to_string(),
            agent_name: agent_name.to_string(),
        };
        self.save_message(&result.to_message());
        result
    }

    fn make_error(&self) -> EngineError {
        EngineError::Narrative(NarrativeFailure::Generation {
            stage: "mock",
            reason: "configured_failure",
        })
    }

    fn guard(&self) -> Result<(), EngineError> {
        if self.should_fail.load(Ordering::SeqCst) {
            return Err(self.make_error());
        }
        Ok(())
    }
}

impl LlmBackend for MockBackend {
    fn model(&self) -> &str {
        "mock"
    }

    fn generate_dialogue(
        &self,
        agent_name: &str,
        context: &crate::narrative::prompt::PromptContext,
        _npc: &NpcCard,
    ) -> Result<LlmCallResult, EngineError> {
        self.guard()?;
        let user_input = context.user_message;
        if user_input.is_empty() {
            Ok(self.make_result(agent_name, "[MockGenerated] Standard greeting."))
        } else {
            Ok(self.make_result(
                agent_name,
                format!("[MockGenerated] Replying to: {user_input}"),
            ))
        }
    }

    fn narrate_action(
        &self,
        agent_name: &str,
        context: &crate::narrative::prompt::PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        let delay = self.delay_ms.load(Ordering::SeqCst);
        if delay > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
        self.guard()?;
        if self.should_return_empty.load(Ordering::SeqCst) {
            return Ok(self.make_result(agent_name, String::new()));
        }
        if !self.per_call_narrations.is_empty() {
            let idx = self.call_index.fetch_add(1, Ordering::SeqCst);
            return Ok(self.make_result(
                agent_name,
                self.per_call_narrations[idx % self.per_call_narrations.len()].clone(),
            ));
        }
        Ok(self.make_result(
            agent_name,
            format!("[MockNarration] {}", context.user_message),
        ))
    }

    fn narrate_arrival(
        &self,
        agent_name: &str,
        context: &crate::narrative::prompt::PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        self.guard()?;
        Ok(self.make_result(
            agent_name,
            format!("[MockArrival] You enter the {}.", context.room.name),
        ))
    }

    fn narrate_continuation(
        &self,
        agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        trigger_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        self.guard()?;
        Ok(self.make_result(agent_name, format!("[Trigger: {trigger_prompt}]")))
    }

    fn complete(
        &self,
        agent_name: &str,
        _system_prompt: &str,
        user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        if self.should_fail.load(Ordering::SeqCst)
            || self.trigger_narration_should_fail.load(Ordering::SeqCst)
        {
            return Err(EngineError::Narrative(NarrativeFailure::Generation {
                stage: "mock_trigger",
                reason: "configured_failure",
            }));
        }
        if !self.per_call_prompt_responses.is_empty() {
            let idx = self.call_index.fetch_add(1, Ordering::SeqCst);
            return Ok(self.make_result(
                agent_name,
                self.per_call_prompt_responses[idx % self.per_call_prompt_responses.len()].clone(),
            ));
        }
        Ok(self.make_result(
            agent_name,
            format!(
                "[Continuation: {}]",
                user_prompt.lines().next().unwrap_or("...")
            ),
        ))
    }

    fn name(&self) -> &str {
        "Mock"
    }

    fn save_message(&self, message: &crate::model::llm_message::LlmMessage) {
        if let Some(storage) = &self.storage {
            let _ = storage.save(message);
        }
    }
}
