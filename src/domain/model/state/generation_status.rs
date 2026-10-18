//! [DOC: docs/diataxis/reference/game_flow.md]
//! Generation status enums and input buffer: the step and the status are independent axes; the pipeline in `application/pipeline/` drives them.

use serde::{Deserialize, Serialize};

use crate::error::{EngineError, LlmFailure, NarrativeFailure};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum GenerationStatus {
    /// No generation running; input buffer is empty or consumed.
    #[default]
    Idle,
    /// Generation in progress; the input buffer's `Step` names the sub-stage.
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

impl GenerationFailureKind {
    /// The words are the design's failure table; only a line that names something
    /// reads `named`.
    pub fn line(self, named: &str) -> String {
        let named = named.trim_end_matches('.');
        match self {
            Self::Unreachable => "The language model could not be reached.".to_string(),
            Self::PromptTooLong => "The story was too long for the language model.".to_string(),
            Self::UnreadableAnswer => "The language model's answer could not be read.".to_string(),
            Self::TokenBudgetSpent => {
                "The model used its budget on reasoning. Try another model.".to_string()
            }
            Self::EmptyText => "The language model gave an empty answer.".to_string(),
            Self::SaveFailed => "The new text could not be saved.".to_string(),
            Self::SceneMissing => format!("The room {named} is not on the map."),
            Self::PresetMissing => "The active prompt preset is missing.".to_string(),
            Self::MissingData => {
                format!("The data for this request is missing: {named}.")
            }
            Self::Contradiction => {
                "The text was saved, but the game state could not be updated.".to_string()
            }
            Self::EventNarrationFailed => "The event narration failed.".to_string(),
            Self::Other => "The engine failed with an internal error.".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenerationFailureKind {
    /// No answer arrived: a timeout, a transport failure, or a non-2xx response.
    Unreachable,
    /// The assembled prompt exceeded the active connection's context budget.
    PromptTooLong,
    /// An answer arrived but could not be used: unparseable, empty, or rejected
    /// by narration post-processing.
    UnreadableAnswer,
    /// The model spent its whole output budget before writing an answer.
    TokenBudgetSpent,
    /// The turn could not be persisted.
    SaveFailed,
    /// The room the turn ran in is gone from the map.
    SceneMissing,
    /// The game's active preset is not in the library it names.
    PresetMissing,
    /// A request needed a record the engine cannot read.
    MissingData,
    /// The model answered, and the answer was empty or only whitespace.
    EmptyText,
    /// The text was saved, and the game rules could not update the state.
    Contradiction,
    /// The event Narration of a Trigger failed; the raw text holds the failure
    /// under it.
    EventNarrationFailed,
    /// Anything the other kinds do not name.
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerationFailure {
    pub kind: GenerationFailureKind,
    /// What the line names, when the kind's line names something: the room the
    /// failure happened in, or the record the request needs. A kind whose line is
    /// fixed words leaves it empty.
    #[serde(default)]
    pub named: String,
    /// The raw text the failure's disclosure holds.
    pub raw: String,
}

impl GenerationFailure {
    /// A failure whose line is the kind's own words, with the raw text behind it.
    pub fn new(kind: GenerationFailureKind, raw: impl Into<String>) -> Self {
        Self {
            kind,
            named: String::new(),
            raw: raw.into(),
        }
    }

    /// A room the map does not hold: the line names it, and the disclosure holds
    /// the error behind it when the engine has one to show.
    pub fn missing_room(room: impl Into<String>, raw: Option<String>) -> Self {
        let room = room.into();
        Self {
            kind: GenerationFailureKind::SceneMissing,
            raw: raw.unwrap_or_else(|| room.clone()),
            named: room,
        }
    }

    /// A record the request needs: the line names it, and the disclosure holds the
    /// error behind it when the engine has one to show.
    pub fn missing_record(name: impl Into<String>, raw: Option<String>) -> Self {
        let name = name.into();
        Self {
            kind: GenerationFailureKind::MissingData,
            raw: raw.unwrap_or_else(|| name.clone()),
            named: name,
        }
    }

    /// The short line the status column shows: the kind's words, with the part the
    /// kind names.
    pub fn line(&self) -> String {
        self.kind.line(&self.named)
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
            EngineError::Llm(LlmFailure::ParseError { .. } | LlmFailure::EmptyResponse { .. })
            | EngineError::Narrative(NarrativeFailure::Generation { .. }) => {
                GenerationFailureKind::UnreadableAnswer
            }
            EngineError::Llm(LlmFailure::TokenBudgetSpent { .. }) => {
                GenerationFailureKind::TokenBudgetSpent
            }
            EngineError::Database(_) | EngineError::Io(_) => GenerationFailureKind::SaveFailed,
            // The line the kind renders names the room, so the room and the error
            // behind it are the failure's two parts.
            EngineError::RoomNotFound(room) => {
                return Self::missing_room(room.clone(), Some(error.to_string()));
            }
            EngineError::PresetNotFound(_) => GenerationFailureKind::PresetMissing,
            _ => GenerationFailureKind::Other,
        };
        Self::new(kind, error.to_string())
    }

    /// Keeps the failing step in the disclosure, which the classified error alone
    /// does not carry.
    pub fn with_context(mut self, context: impl std::fmt::Display) -> Self {
        self.raw = format!("{context}: {}", self.raw);
        self
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum Step {
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

impl Step {
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
    pub step: Step,
}
