//! [DOC: docs/diataxis/reference/game_flow.md]
//! Generation status enums and input buffer — phase/status are independent axes; live state machine lives in `application/pipeline/` (`core.rs` orchestration, `phases.rs` implementations).

use serde::{Deserialize, Serialize};

use crate::error::{EngineError, LlmFailure, NarrativeFailure};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum GenerationStatus {
    /// No generation running; input buffer is empty or consumed.
    #[default]
    Idle,
    /// Generation in progress; `phase` tracks sub-stage.
    Generating,
    /// Generation failed; the payload classifies the failure.
    Error(GenerationFailure),
}

impl GenerationStatus {
    pub fn is_generating(&self) -> bool {
        matches!(self, Self::Generating)
    }

    pub fn failure(&self) -> Option<&GenerationFailure> {
        match self {
            Self::Error(failure) => Some(failure),
            _ => None,
        }
    }
}

/// What went wrong in a failed generation, coarse enough for the status
/// display to name it in one line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenerationFailureKind {
    /// No answer arrived: a timeout, a transport failure, or a non-2xx response.
    Unreachable,
    /// The assembled prompt exceeded the active connection's context budget.
    PromptTooLong,
    /// An answer arrived but could not be used: unparseable, empty, or rejected
    /// by narration post-processing.
    UnreadableAnswer,
    /// The turn could not be persisted.
    SaveFailed,
    /// The room the turn ran in is gone from the map.
    SceneMissing,
    /// The game's active preset is not in the library it names.
    PresetMissing,
    /// Anything the other kinds do not name.
    Other,
}

/// A failed generation: `kind` drives the one-line status sentence and `raw`
/// is the classified error text behind the details disclosure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerationFailure {
    pub kind: GenerationFailureKind,
    pub raw: String,
}

impl GenerationFailure {
    /// A failure the call site names itself, for a condition with no typed
    /// `EngineError` behind it.
    pub fn new(kind: GenerationFailureKind, raw: impl Into<String>) -> Self {
        Self {
            kind,
            raw: raw.into(),
        }
    }

    /// Classifies a typed engine error so no call site matches on message text.
    pub fn from_engine_error(error: &EngineError) -> Self {
        let kind = match error {
            EngineError::Llm(
                LlmFailure::Timeout | LlmFailure::Network { .. } | LlmFailure::Http { .. },
            ) => GenerationFailureKind::Unreachable,
            EngineError::ContextOverflow { .. }
            | EngineError::Narrative(NarrativeFailure::PromptBuild { .. }) => {
                GenerationFailureKind::PromptTooLong
            }
            EngineError::Llm(LlmFailure::ParseError { .. } | LlmFailure::EmptyResponse)
            | EngineError::Narrative(NarrativeFailure::Generation { .. }) => {
                GenerationFailureKind::UnreadableAnswer
            }
            EngineError::Database(_) | EngineError::Io(_) => GenerationFailureKind::SaveFailed,
            EngineError::RoomNotFound(_) => GenerationFailureKind::SceneMissing,
            EngineError::PresetNotFound(_) => GenerationFailureKind::PresetMissing,
            _ => GenerationFailureKind::Other,
        };
        Self {
            kind,
            raw: error.to_string(),
        }
    }

    /// Prefixes `raw` with the pipeline step that failed, so the disclosure
    /// keeps the context the classified error alone does not carry.
    pub fn with_context(mut self, context: impl std::fmt::Display) -> Self {
        self.raw = format!("{context}: {}", self.raw);
        self
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum GenerationPhase {
    /// Narrator LLM generating the scene response.
    #[default]
    Narrating,
    /// Quantifier LLM evaluating NPC presence and movement.
    Quantifying,
    /// Event-generation LLM producing derived side effects.
    GeneratingEvent,
    /// Options LLM producing the pickable option set.
    Options,
}

impl GenerationPhase {
    pub fn as_endpoint_str(&self) -> &'static str {
        match self {
            Self::Narrating => "narrating",
            Self::Quantifying => "quantifying",
            Self::GeneratingEvent => "generating-event",
            Self::Options => "options",
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputBuffer {
    pub input: String,
    pub status: GenerationStatus,
    pub phase: GenerationPhase,
}
