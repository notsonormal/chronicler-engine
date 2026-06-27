//! [DOC: docs/system/llm_processing.md]
//! Mock LLM provider for testing

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use crate::error::{EngineError, NarrativeFailure};
use crate::storage::Storage;

use super::backend::{LlmBackend, LlmCallResult};

fn extract_player_input(user_prompt: &str) -> Option<String> {
    const OPEN: &str = "<PlayerInput>\n";
    const CLOSE: &str = "\n</PlayerInput>";
    let start = user_prompt.find(OPEN)?;
    let content_start = start + OPEN.len();
    let end = user_prompt[content_start..].find(CLOSE)?;
    Some(user_prompt[content_start..content_start + end].to_string())
}

#[derive(Default)]
pub struct MockBackend {
    pub should_fail: AtomicBool,
    pub should_return_empty: AtomicBool,
    pub trigger_narration_should_fail: AtomicBool,
    pub delay_ms: AtomicU64,
    pub trigger_delay_ms: AtomicU64,
    pub per_call_narrations: Vec<String>,
    pub per_call_prompt_responses: Vec<String>,
    pub call_index: AtomicUsize,
    pub storage: Option<Arc<Storage>>,
    /// Set to `true` when `complete` is entered (useful for tests to detect pipeline start).
    pub narration_started: AtomicBool,
    /// Set to `true` when `complete` is entered with trigger agent (useful for tests to detect trigger start).
    pub trigger_started: AtomicBool,
}

impl MockBackend {
    pub fn new(storage: Option<Arc<Storage>>) -> Self {
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

    pub fn with_trigger_delay(ms: u64) -> Self {
        Self {
            trigger_delay_ms: AtomicU64::new(ms),
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

    fn complete(
        &self,
        agent_name: &str,
        _system_prompt: &str,
        user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        let is_narration = agent_name == super::backend::AGENT_NARRATOR
            || agent_name == super::backend::AGENT_DIALOGUE;

        if is_narration {
            self.narration_started.store(true, Ordering::SeqCst);
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
            let input = extract_player_input(user_prompt)
                .unwrap_or_else(|| user_prompt.lines().next().unwrap_or("...").to_string());
            Ok(self.make_result(agent_name, format!("[MockNarration] {input}")))
        } else {
            self.trigger_started.store(true, Ordering::SeqCst);
            let delay = self.trigger_delay_ms.load(Ordering::SeqCst);
            if delay > 0 {
                std::thread::sleep(std::time::Duration::from_millis(delay));
            }
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
                    self.per_call_prompt_responses[idx % self.per_call_prompt_responses.len()]
                        .clone(),
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
    }

    fn name(&self) -> &str {
        "Mock"
    }

    fn save_message(&self, message: &crate::model::llm_message::LlmMessage) {
        if let Some(storage) = &self.storage {
            let _ = storage.save_llm_message(message);
        }
    }
}
