//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! The narrator: the assembly and the model call a Job's narration needs

use std::sync::Arc;

use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::prompting::token_budget::TokenBudget;
use crate::application::prompting::{AssembledPrompt, CharacterContext, PromptAssembler, PromptContext};
use crate::domain::model::character::PersonaCard;
use crate::domain::model::generation::Generation;
use crate::domain::model::generation_parameters::CallSettings;
use crate::domain::model::map::Room;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::{NarrativePerspective, NarrativeTense};
use crate::domain::model::state::generation_status::{GenerationFailure, GenerationFailureKind};
use crate::domain::model::state::message_types::{MessageType, StoryRow};
use crate::domain::model::tail::Tail;
use crate::domain::model::world::WorldCard;
use crate::error::EngineError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Narrated {
    pub text: String,
    pub backend_name: String,
    pub model_name: String,
}

/// The values one narration reads. The narrate step loads them; the narrator holds
/// no storage.
pub struct NarrationValues<'a> {
    pub world: &'a WorldCard,
    /// Resolved posture: the Game's own, else the World's.
    pub narrative_perspective: NarrativePerspective,
    pub narrative_tense: NarrativeTense,
    pub room: &'a Room,
    pub characters: CharacterContext<'a>,
    pub persona: &'a PersonaCard,
    /// The entries the narration follows.
    pub history: &'a [StoryRow],
    /// The text of the request's Input, when it carries one.
    pub input: Option<&'a str>,
    pub tail: &'a Tail,
}

pub struct Narrator {
    assembler: PromptAssembler,
    recorder: Arc<LlmCallRecorder>,
    impersonate_recorder: Arc<LlmCallRecorder>,
}

impl std::fmt::Debug for Narrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Narrator")
            .field("provider", &self.recorder.provider_label())
            .finish_non_exhaustive()
    }
}

impl Narrator {
    pub fn new(
        assembler: PromptAssembler,
        recorder: Arc<LlmCallRecorder>,
        impersonate_recorder: Arc<LlmCallRecorder>,
    ) -> Self {
        Self {
            assembler,
            recorder,
            impersonate_recorder,
        }
    }

    /// Only the wiring tests call it: they drive the recorder that the wiring
    /// built, and check a provider that failed to build, which
    /// `backend_info` does not show.
    pub fn recorder(&self) -> &Arc<LlmCallRecorder> {
        &self.recorder
    }

    fn recorder_for(&self, preset_type: PresetType) -> &Arc<LlmCallRecorder> {
        match preset_type {
            PresetType::Impersonate => &self.impersonate_recorder,
            _ => &self.recorder,
        }
    }

    /// The provider and model the next narration would use.
    pub fn backend_info(&self) -> (String, String) {
        match self.recorder.provider() {
            Ok(p) => (p.name().to_string(), p.model().to_string()),
            Err(e) => (format!("<unresolved: {e}>"), "<unresolved>".to_string()),
        }
    }

    fn assemble(
        &self,
        context: &PromptContext,
        preset: &PromptPreset,
        budget: TokenBudget,
    ) -> Result<AssembledPrompt, EngineError> {
        self.assembler
            .assemble(context, preset, &context.world.global_rules, budget)
    }

    /// The prompt rules of each kind live here: the user message, the guide and the
    /// Impersonate flag. The role is the one whose health the call feeds: the
    /// narrator's own for a generation, and the Trigger's for an event Narration.
    pub fn narrate(
        &self,
        generation: &Generation,
        values: &NarrationValues,
        preset: &PromptPreset,
        call: &CallSettings,
        max_context_tokens: u32,
    ) -> Result<Narrated, GenerationFailure> {
        let role = generation.kind().role();
        let user_message = Self::user_message(generation, values);
        let context = PromptContext::new(
            values.world,
            values.narrative_perspective,
            values.narrative_tense,
            values.room,
            values.characters,
            values.persona,
            &user_message,
            values.history,
        );
        let context = match generation {
            Generation::Impersonate { .. } => context.with_impersonate(true),
            Generation::Narration { guide } => context.with_guide(guide.clone()),
            Generation::InputRetry | Generation::Event(_) => context,
        };
        let budget = TokenBudget {
            max_context_tokens,
            max_tokens: call.max_tokens,
        };
        let assembled = self
            .assemble(&context, preset, budget)
            .map_err(|e| GenerationFailure::from_engine_error(&e))?;

        tracing::info!("Narration LLM call (agent={})", role.agent_name());
        let narration_result = self
            .recorder_for(generation.preset_type())
            .complete(
                role.agent_name(),
                &assembled.parts,
                &CallSettings {
                    max_tokens: assembled.max_tokens,
                    ..call.clone()
                },
            )
            .map_err(|e| GenerationFailure::from_engine_error(&e))?;
        tracing::info!("Narration complete");

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

    /// The Player's direction for an Impersonate, the Input's own text for the Retry
    /// of a typed Input, the Trigger's continuation prompt for an event, and otherwise the
    /// Input the new text follows: the request's own text, or the Input that ends
    /// the window.
    fn user_message(generation: &Generation, values: &NarrationValues) -> String {
        match generation {
            Generation::Impersonate { direction } => direction.clone().unwrap_or_default(),
            Generation::InputRetry => values.tail.last.text().to_string(),
            Generation::Event(record) => {
                Self::continuation_user_message(values.history, &record.direction)
            }
            Generation::Narration { .. } => match values.input {
                Some(text) if !text.is_empty() => text.to_string(),
                _ => Self::previous_input(values.history)
                    .unwrap_or_default()
                    .to_string(),
            },
        }
    }

    fn continuation_user_message(history: &[StoryRow], direction: &str) -> String {
        let previous = history
            .last()
            .map(|entry| entry.text.as_str())
            .unwrap_or("");
        format!(
            "Previous narration:\n{previous}\n\nTrigger direction: {direction}\n\n\
             Continue the scene naturally, incorporating the trigger event into the narrative. \
             Do NOT repeat or contradict what was already described. Build naturally on the existing scene."
        )
    }

    fn previous_input(history: &[StoryRow]) -> Option<&str> {
        history
            .last()
            .filter(|entry| matches!(entry.message_type, MessageType::Input))
            .map(|entry| entry.text.as_str())
    }
}
