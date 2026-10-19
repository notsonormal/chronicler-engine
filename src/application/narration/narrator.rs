//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! The narrator: the assembly and the model call a Job's narration needs

use std::sync::Arc;

use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::prompting::token_budget::TokenBudget;
use crate::application::prompting::{AssembledPrompt, PromptAssembler, PromptContext};
use crate::domain::model::agent::Role;
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::state::generation_status::{GenerationFailure, GenerationFailureKind};
use crate::error::EngineError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Narrated {
    pub text: String,
    pub backend_name: String,
    pub model_name: String,
}

pub struct Narrator {
    assembler: PromptAssembler,
    recorder: Arc<LlmCallRecorder>,
}

impl std::fmt::Debug for Narrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Narrator")
            .field("provider", &self.recorder.provider_label())
            .finish_non_exhaustive()
    }
}

impl Narrator {
    pub fn new(assembler: PromptAssembler, recorder: Arc<LlmCallRecorder>) -> Self {
        Self {
            assembler,
            recorder,
        }
    }

    pub fn recorder(&self) -> &Arc<LlmCallRecorder> {
        &self.recorder
    }

    /// The provider and model the next narration would use.
    pub fn backend_info(&self) -> (String, String) {
        match self.recorder.provider() {
            Ok(p) => (p.name().to_string(), p.model().to_string()),
            Err(e) => (format!("<unresolved: {e}>"), "<unresolved>".to_string()),
        }
    }

    /// The prompt of one narration without the model call; the trigger context
    /// keeps it for the event narration to re-use.
    pub fn assemble(
        &self,
        context: &PromptContext,
        preset: &PromptPreset,
        response_length: &str,
        budget: TokenBudget,
    ) -> Result<AssembledPrompt, EngineError> {
        self.assembler.assemble(
            context,
            preset,
            &context.world.global_rules,
            Some(response_length),
            budget,
        )
    }

    /// `role` is the role whose health the call feeds: the narrator's own for a
    /// turn, and the Trigger's for an event Narration.
    pub fn narrate(
        &self,
        role: Role,
        context: &PromptContext,
        preset: &PromptPreset,
        response_length: &str,
        budget: TokenBudget,
    ) -> Result<Narrated, GenerationFailure> {
        let assembled = self
            .assemble(context, preset, response_length, budget)
            .map_err(|e| GenerationFailure::from_engine_error(&e))?;

        tracing::info!("Narration LLM call (agent={})", role.agent_name());
        let narration_result = self
            .recorder
            .complete(
                role.agent_name(),
                &assembled.system_prompt,
                &assembled.user_prompt,
                Some(assembled.max_tokens),
            )
            .map_err(|e| GenerationFailure::from_engine_error(&e))?;
        tracing::info!("Pipeline ✓ Narration complete");

        let text = narration_result.text;
        if text.trim().is_empty() {
            return Err(GenerationFailure::new(
                GenerationFailureKind::EmptyText,
                "the answer was empty",
            ));
        }

        Ok(Narrated {
            text,
            backend_name: narration_result.backend_name,
            model_name: narration_result.model_name,
        })
    }
}
