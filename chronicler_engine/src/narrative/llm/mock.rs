use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::error::{EngineError, NarrativeFailure};
use crate::model::character::NpcCard;

use super::backend::LlmBackend;

#[derive(Default)]
pub struct MockBackend {
    /// If true, all narration methods return `Err(EngineError::Narrative(NarrativeFailure::Generation { stage: "mock", reason: "configured_failure" }))`.
    pub should_fail: AtomicBool,
    /// If true, `narrate_action` returns `Ok("")` â€” simulates an empty LLM response.
    pub should_return_empty: AtomicBool,
    /// If true, `narrate_action_from_prompt` (trigger narration) returns `Err`.
    pub trigger_narration_should_fail: AtomicBool,
    /// Milliseconds to sleep in `narrate_action` to simulate a slow LLM.
    pub delay_ms: AtomicU64,
}

impl MockBackend {
    /// Backend that fails all narration calls.
    pub fn failing() -> Self {
        Self {
            should_fail: AtomicBool::new(true),
            ..Default::default()
        }
    }

    /// Backend whose `narrate_action` returns an empty string â€” simulates `LlmEmptyResponse`.
    pub fn with_empty_response() -> Self {
        Self {
            should_return_empty: AtomicBool::new(true),
            ..Default::default()
        }
    }

    /// Backend whose trigger narration (`narrate_action_from_prompt`) fails.
    pub fn with_failing_trigger_narration() -> Self {
        Self {
            trigger_narration_should_fail: AtomicBool::new(true),
            ..Default::default()
        }
    }

    /// Backend with a configurable delay in `narrate_action`.
    pub fn with_delay(ms: u64) -> Self {
        Self {
            delay_ms: AtomicU64::new(ms),
            ..Default::default()
        }
    }
}

impl LlmBackend for MockBackend {
    fn generate_dialogue(
        &self,
        context: &crate::narrative::prompt::PromptContext,
        _npc: &NpcCard,
    ) -> Result<String, EngineError> {
        if self.should_fail.load(Ordering::SeqCst) {
            return Err(EngineError::Narrative(NarrativeFailure::Generation {
                stage: "mock",
                reason: "configured_failure",
            }));
        }
        let user_input = context.user_message;
        if user_input.is_empty() {
            Ok("[MockGenerated] Standard greeting.".to_string())
        } else {
            Ok(format!("[MockGenerated] Replying to: {user_input}"))
        }
    }

    fn narrate_action(
        &self,
        context: &crate::narrative::prompt::PromptContext,
    ) -> Result<String, EngineError> {
        let delay = self.delay_ms.load(Ordering::SeqCst);
        if delay > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
        if self.should_fail.load(Ordering::SeqCst) {
            return Err(EngineError::Narrative(NarrativeFailure::Generation {
                stage: "mock",
                reason: "configured_failure",
            }));
        }
        if self.should_return_empty.load(Ordering::SeqCst) {
            return Ok(String::new());
        }
        Ok(format!("[MockNarration] {}", context.user_message))
    }

    fn narrate_arrival(
        &self,
        context: &crate::narrative::prompt::PromptContext,
    ) -> Result<String, EngineError> {
        if self.should_fail.load(Ordering::SeqCst) {
            return Err(EngineError::Narrative(NarrativeFailure::Generation {
                stage: "mock",
                reason: "configured_failure",
            }));
        }
        Ok(format!(
            "[MockArrival] You enter the {}.",
            context.room.name
        ))
    }

    fn narrate_continuation(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
        trigger_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<String, EngineError> {
        if self.should_fail.load(Ordering::SeqCst) {
            return Err(EngineError::Narrative(NarrativeFailure::Generation {
                stage: "mock",
                reason: "configured_failure",
            }));
        }
        Ok(format!("[Trigger: {trigger_prompt}]"))
    }

    fn narrate_action_from_prompt(
        &self,
        _system_prompt: &str,
        user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<String, EngineError> {
        if self.should_fail.load(Ordering::SeqCst)
            || self.trigger_narration_should_fail.load(Ordering::SeqCst)
        {
            return Err(EngineError::Narrative(NarrativeFailure::Generation {
                stage: "mock_trigger",
                reason: "configured_failure",
            }));
        }
        Ok(format!(
            "[Continuation: {}]",
            user_prompt.lines().next().unwrap_or("...")
        ))
    }

    fn name(&self) -> &str {
        "Mock"
    }
}
