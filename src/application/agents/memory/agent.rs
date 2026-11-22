//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Story memory writer implementation.

use std::sync::Arc;

use crate::application::agents::memory::parser::MemoryEntriesAnswer;
use crate::application::agents::memory::prompt::StoryMemoryPrompt;
use crate::application::agents::trait_def::BeforeNarrationAgent;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::ports::llm_provider::AGENT_STORY_MEMORY;
use crate::domain::model::agent::StoryMemoryHookInput;
use crate::domain::model::generation_parameters::CallSettings;
use crate::domain::model::prompt_part::PromptPart;
use crate::domain::model::memory_entry::MemoryEntry;
use crate::error::EngineError;

const MAX_ATTEMPTS: u32 = 2;

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

    fn attempt(
        &self,
        prompt: &[PromptPart],
        call: &CallSettings,
        answer: &MemoryEntriesAnswer,
    ) -> Result<Vec<MemoryEntry>, EngineError> {
        self.recorder
            .complete_read(AGENT_STORY_MEMORY, prompt, call, |result| {
                answer.read(result)
            })
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
    ) -> Result<Vec<MemoryEntry>, EngineError> {
        let (system_prompt, user_prompt) = StoryMemoryPrompt::new(input).build();
        let prompt = PromptPart::system_and_user(system_prompt, user_prompt);
        let answer = MemoryEntriesAnswer::new(input);
        tracing::info!(
            "[Memory] Calling provider: {} for Messages #{} to #{}",
            self.recorder.provider_label(),
            input.oldest(),
            input.newest()
        );
        let mut outcome = self.attempt(&prompt, call, &answer);
        for attempt in 2..=MAX_ATTEMPTS {
            let Err(error) = &outcome else { break };
            tracing::warn!(
                "[Memory] try {} of {MAX_ATTEMPTS} failed: {error}",
                attempt - 1
            );
            outcome = self.attempt(&prompt, call, &answer);
        }
        outcome
    }
}
