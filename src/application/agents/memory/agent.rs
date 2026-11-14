//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Story memory writer implementation.

use std::sync::Arc;

use crate::application::agents::trait_def::BeforeNarrationAgent;
use crate::application::agents::memory::prompt::StoryMemoryPrompt;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::ports::llm_provider::{LlmCallResult, AGENT_STORY_MEMORY};
use crate::application::prompting::token_budget::STORY_MEMORY_CHARS;
use crate::domain::model::agent::StoryMemoryHookInput;
use crate::domain::model::generation_parameters::CallSettings;
use crate::domain::model::prompt_part::PromptPart;
use crate::domain::model::story_memory::StoryMemory;
use crate::error::{EngineError, LlmFailure};

pub struct StoryMemoryWriter {
    recorder: Arc<LlmCallRecorder>,
}

impl std::fmt::Debug for StoryMemoryWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoryMemoryWriter").finish_non_exhaustive()
    }
}

impl StoryMemoryWriter {
    pub fn new(recorder: Arc<LlmCallRecorder>) -> Self {
        Self { recorder }
    }

    /// A cut text loses the newest events, which sit at its end, so an answer that
    /// does not fit fails whole and the previous story memory stays.
    fn check_answer(answer: &LlmCallResult) -> Result<(), EngineError> {
        let text = answer.text.trim();
        if text.is_empty() {
            return Err(LlmFailure::EmptyResponse {
                raw_response: answer.raw_response_json.clone(),
                finish_reason: None,
            }
            .into());
        }
        if answer.stopped_at_limit || text.chars().count() > STORY_MEMORY_CHARS {
            return Err(LlmFailure::AnswerTooLong {
                raw_response: answer.raw_response_json.clone(),
                stopped_at_limit: answer.stopped_at_limit,
            }
            .into());
        }
        Ok(())
    }
}

impl BeforeNarrationAgent for StoryMemoryWriter {
    fn name(&self) -> &str {
        AGENT_STORY_MEMORY
    }

    fn run(
        &self,
        input: &StoryMemoryHookInput,
        call: &CallSettings,
    ) -> Result<StoryMemory, EngineError> {
        let (system_prompt, user_prompt) = StoryMemoryPrompt::new(input, call.max_tokens).build();
        let prompt = PromptPart::system_and_user(system_prompt, user_prompt);
        tracing::info!(
            "[Memory] Calling provider: {} for entries up to #{}",
            self.recorder.provider_label(),
            input.newest()
        );
        let answer = self.recorder.complete_checked(
            AGENT_STORY_MEMORY,
            &prompt,
            call,
            Self::check_answer,
        )?;
        Ok(StoryMemory {
            text: answer.text.trim().to_string(),
            covers_up_to: input.newest(),
        })
    }
}
