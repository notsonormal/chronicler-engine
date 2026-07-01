//! [DOC: docs/system/llm_processing.md]
//! Mock LLM provider for testing

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use crate::error::{EngineError, NarrativeFailure};

use crate::application::ports::llm_provider::{LlmProvider, LlmCallResult};

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
    pub(crate) should_fail: AtomicBool,
    pub(crate) should_return_empty: AtomicBool,
    pub(crate) trigger_narration_should_fail: AtomicBool,
    pub(crate) delay_ms: AtomicU64,
    pub(crate) trigger_delay_ms: AtomicU64,
    pub(crate) per_call_narrations: Vec<String>,
    pub(crate) per_call_prompt_responses: Vec<String>,
    pub(crate) call_index: AtomicUsize,
    /// Set when `complete` narration starts. Test-sync primitive — read with `.load(Ordering::SeqCst)`, do not mutate externally.
    pub narration_started: AtomicBool,
    /// Set when `complete` trigger narration starts. Test-sync primitive — read with `.load(Ordering::SeqCst)`, do not mutate externally.
    pub trigger_started: AtomicBool,
}

impl MockBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fail(mut self) -> Self {
        self.should_fail = AtomicBool::new(true);
        self
    }

    pub fn with_empty_response(mut self) -> Self {
        self.should_return_empty = AtomicBool::new(true);
        self
    }

    pub fn with_trigger_narration_fail(mut self) -> Self {
        self.trigger_narration_should_fail = AtomicBool::new(true);
        self
    }

    pub fn with_delay(mut self, ms: u64) -> Self {
        self.delay_ms = AtomicU64::new(ms);
        self
    }

    pub fn with_trigger_delay(mut self, ms: u64) -> Self {
        self.trigger_delay_ms = AtomicU64::new(ms);
        self
    }

    pub fn with_narrations(mut self, v: Vec<String>) -> Self {
        self.per_call_narrations = v;
        self
    }

    pub fn with_prompt_responses(mut self, v: Vec<String>) -> Self {
        self.per_call_prompt_responses = v;
        self
    }

    fn make_result(
        &self,
        agent_name: &str,
        text: impl Into<String>,
        _system_prompt: &str,
        _user_prompt: &str,
    ) -> LlmCallResult {
        let text = text.into();
        LlmCallResult {
            text: text.clone(),
            raw_request_json: String::new(),
            raw_response_json: format!("{{\"content\":\"{text}\"}}"),
            backend_name: self.name().to_string(),
            model_name: self.model().to_string(),
            agent_name: agent_name.to_string(),
        }
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

impl LlmProvider for MockBackend {
    fn model(&self) -> &str {
        "mock"
    }

    fn name(&self) -> &str {
        "Mock"
    }

    fn complete(
        &self,
        agent_name: &str,
        _system_prompt: &str,
        user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        let is_narration = agent_name == crate::application::ports::llm_provider::AGENT_NARRATOR
            || agent_name == crate::application::ports::llm_provider::AGENT_DIALOGUE;

        if is_narration {
            self.narration_started.store(true, Ordering::SeqCst);
            let delay = self.delay_ms.load(Ordering::SeqCst);
            if delay > 0 {
                std::thread::sleep(std::time::Duration::from_millis(delay));
            }
            self.guard()?;
            if self.should_return_empty.load(Ordering::SeqCst) {
                return Ok(self.make_result(
                    agent_name,
                    String::new(),
                    _system_prompt,
                    user_prompt,
                ));
            }
            if !self.per_call_narrations.is_empty() {
                let idx = self.call_index.fetch_add(1, Ordering::SeqCst);
                return Ok(self.make_result(
                    agent_name,
                    self.per_call_narrations[idx % self.per_call_narrations.len()].clone(),
                    _system_prompt,
                    user_prompt,
                ));
            }
            let input = extract_player_input(user_prompt)
                .unwrap_or_else(|| user_prompt.lines().next().unwrap_or("...").to_string());
            Ok(self.make_result(
                agent_name,
                format!("[MockNarration] {input}"),
                _system_prompt,
                user_prompt,
            ))
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
                    _system_prompt,
                    user_prompt,
                ));
            }
            Ok(self.make_result(
                agent_name,
                format!(
                    "[Continuation: {}]",
                    user_prompt.lines().next().unwrap_or("...")
                ),
                _system_prompt,
                user_prompt,
            ))
        }
    }
}
