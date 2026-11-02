//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! Mock LLM provider for testing

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

use crate::adapters::driven::utils::mock::{extract_player_input, first_user_line, request_json};
use crate::error::{EngineError, NarrationFailure};

use crate::application::ports::llm_provider::{LlmCallFailure, LlmCallResult, LlmProvider};
use crate::domain::model::generation_parameters::CallSettings;
use crate::domain::model::prompt_part::PromptPart;

#[derive(Default)]
pub struct MockBackend {
    pub(crate) should_fail: AtomicBool,
    pub(crate) fail_first_n: AtomicU32,
    pub(crate) should_return_empty: AtomicBool,
    pub(crate) trigger_narration_should_fail: AtomicBool,
    pub(crate) per_call_narrations: Vec<String>,
    pub(crate) per_call_prompt_responses: Vec<String>,
    pub(crate) call_index: AtomicUsize,
    /// The model this mock stands in for. `None` reports the bare `"mock"`.
    pub(crate) model: Option<String>,
}

impl MockBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }

    pub fn with_fail(mut self) -> Self {
        self.should_fail = AtomicBool::new(true);
        self
    }

    /// Trigger and quantifier calls never consume this failure budget.
    pub fn with_fail_first_n(mut self, n: u32) -> Self {
        self.fail_first_n = AtomicU32::new(n);
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
        prompt: &[PromptPart],
        text: impl Into<String>,
    ) -> LlmCallResult {
        let text = text.into();
        LlmCallResult {
            text: text.clone(),
            raw_request_json: request_json(prompt),
            raw_response_json: format!("{{\"content\":\"{text}\"}}"),
            backend_name: self.name().to_string(),
            model_name: self.model().to_string(),
            agent_name: agent_name.to_string(),
        }
    }

    fn make_error(&self) -> EngineError {
        EngineError::Narration(NarrationFailure::Generation {
            stage: "mock",
            reason: "configured_failure",
        })
    }

    fn guard(&self) -> Result<(), EngineError> {
        if self.fail_first_n.load(Ordering::SeqCst) > 0 {
            self.fail_first_n.fetch_sub(1, Ordering::SeqCst);
            return Err(self.make_error());
        }
        if self.should_fail.load(Ordering::SeqCst) {
            return Err(self.make_error());
        }
        Ok(())
    }
}

impl LlmProvider for MockBackend {
    fn model(&self) -> &str {
        self.model.as_deref().unwrap_or("mock")
    }

    fn name(&self) -> &str {
        "Mock"
    }

    fn complete(
        &self,
        agent_name: &str,
        prompt: &[PromptPart],
        _settings: &CallSettings,
    ) -> Result<LlmCallResult, LlmCallFailure> {
        let is_narration = agent_name == crate::application::ports::llm_provider::AGENT_NARRATOR
            || agent_name == crate::application::ports::llm_provider::AGENT_DIALOGUE;

        if is_narration {
            self.guard().map_err(LlmCallFailure::from)?;
            if self.should_return_empty.load(Ordering::SeqCst) {
                return Ok(self.make_result(agent_name, prompt, String::new()));
            }
            if !self.per_call_narrations.is_empty() {
                let idx = self.call_index.fetch_add(1, Ordering::SeqCst);
                return Ok(self.make_result(
                    agent_name,
                    prompt,
                    self.per_call_narrations[idx % self.per_call_narrations.len()].clone(),
                ));
            }
            let input = extract_player_input(prompt).unwrap_or_else(|| first_user_line(prompt));
            Ok(self.make_result(agent_name, prompt, format!("[MockNarration] {input}")))
        } else {
            if self.should_fail.load(Ordering::SeqCst)
                || self.trigger_narration_should_fail.load(Ordering::SeqCst)
            {
                return Err(EngineError::Narration(NarrationFailure::Generation {
                    stage: "mock_trigger",
                    reason: "configured_failure",
                })
                .into());
            }
            if !self.per_call_prompt_responses.is_empty() {
                let idx = self.call_index.fetch_add(1, Ordering::SeqCst);
                return Ok(self.make_result(
                    agent_name,
                    prompt,
                    self.per_call_prompt_responses[idx % self.per_call_prompt_responses.len()]
                        .clone(),
                ));
            }
            if agent_name == "options" {
                // The options agent needs parseable output in the default
                // seed shape (<suggestion> tags) so browser/E2E runs
                // exercise the real generation path. Seeded responses keep
                // priority for pipeline-level tests.
                return Ok(self.make_result(
                    agent_name,
                    prompt,
                    "<suggestion>Option one from the mock.</suggestion>\n\
                     <suggestion>Option two from the mock.</suggestion>\n\
                     <suggestion>Option three from the mock.</suggestion>",
                ));
            }
            Ok(self.make_result(
                agent_name,
                prompt,
                format!("[Continuation: {}]", first_user_line(prompt)),
            ))
        }
    }
}
