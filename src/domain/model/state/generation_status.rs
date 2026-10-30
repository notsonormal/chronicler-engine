//! [DOC: docs/diataxis/reference/game_flow.md]
//! Generation status enums: the step and the status are independent axes; the Job in `application/pipeline/` drives them.

use serde::{Deserialize, Serialize};

use crate::domain::model::agent::Kind;
use crate::error::{EngineError, LlmFailure, NarrationFailure};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum GenerationStatus {
    /// No Job holds the Game, and no run.
    #[default]
    Idle,
    /// A Job or an Autoplay run holds the Game; the Job's `Step` names the sub-stage.
    Busy,
    /// Generation failed; the payload classifies the failure.
    Failed(GenerationFailure),
}

impl GenerationFailureKind {
    /// The 240px status column clamps this sentence to two lines, so any remedy it
    /// names has to be one the dashboard can carry out. The words are the design's
    /// failure table; only a line that names something reads `named`.
    pub fn line(self, named: &str) -> String {
        let named = named.trim_end_matches('.');
        match self {
            Self::Unreachable => "The language model could not be reached.".to_string(),
            Self::PromptTooLong => "The story was too long for the language model.".to_string(),
            Self::UnreadableAnswer => "The language model's answer could not be read.".to_string(),
            Self::TokenBudgetSpent => {
                "Reasoning used the budget. Lower the reasoning effort.".to_string()
            }
            Self::EmptyText => "The language model gave an empty answer.".to_string(),
            Self::SaveFailed => "The new text could not be saved.".to_string(),
            Self::InvalidRoom => format!("The room {named} is not on the map."),
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

/// The one definition of the kinds, so `GenerationFailureKind::ALL` cannot lag the enum.
macro_rules! failure_kinds {
    ($($(#[$doc:meta])* $variant:ident),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum GenerationFailureKind {
            $($(#[$doc])* $variant),+
        }

        impl GenerationFailureKind {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
        }
    };
}

failure_kinds! {
    /// No answer arrived: a timeout, a transport failure, or a non-2xx response.
    Unreachable,
    /// The assembled prompt exceeded the active connection's context budget.
    PromptTooLong,
    /// An answer arrived but could not be used: unparseable, empty, or rejected
    /// by narration post-processing.
    UnreadableAnswer,
    /// The model spent its whole output budget before writing an answer.
    TokenBudgetSpent,
    /// The new text could not be saved.
    SaveFailed,
    /// The room the request ran in is gone from the map.
    InvalidRoom,
    /// A request needed a record the engine cannot read.
    MissingData,
    /// The model answered, and the answer was empty or only whitespace.
    EmptyText,
    /// The text was saved, and the game state could not be updated: the game rules
    /// refused it, or the write of the update failed.
    Contradiction,
    /// The event Narration of a Trigger failed; the raw text holds the failure
    /// under it.
    EventNarrationFailed,
    /// Anything the other kinds do not name.
    Other,
}

/// The records a request reads. The name of a record is what the missing-data
/// line names.
///
/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Record {
    Game,
    Persona,
    Story,
    Message,
    StartState,
    SavedState,
    CurrentState,
    World,
    Settings,
    Preset,
}

impl Record {
    pub fn name(self) -> &'static str {
        match self {
            Self::Game => "the game",
            Self::Persona => "the persona",
            Self::Story => "the story",
            Self::Message => "the message",
            Self::StartState => "the start state",
            Self::SavedState => "the saved state",
            Self::CurrentState => "the current state",
            Self::World => "the world",
            Self::Settings => "the settings",
            Self::Preset => "the preset",
        }
    }
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
    /// the room too.
    pub fn invalid_room(room: impl Into<String>) -> Self {
        let room = room.into();
        Self {
            kind: GenerationFailureKind::InvalidRoom,
            raw: room.clone(),
            named: room,
        }
    }

    /// A record the request reads and the engine holds no row for: the line names
    /// it.
    pub fn missing(record: Record) -> Self {
        Self {
            kind: GenerationFailureKind::MissingData,
            named: record.name().to_string(),
            raw: record.name().to_string(),
        }
    }

    /// A read of a record failed: the line is the one of a missing record, and the
    /// disclosure holds the error behind it.
    pub fn unreadable(record: Record, error: &EngineError) -> Self {
        Self {
            raw: error.to_string(),
            ..Self::missing(record)
        }
    }

    /// A write failed; the disclosure holds the error.
    pub fn save_failed(error: &EngineError) -> Self {
        Self::new(GenerationFailureKind::SaveFailed, error.to_string())
    }

    /// The text is saved, and the game state could not be updated.
    pub fn contradiction(raw: impl Into<String>) -> Self {
        Self::new(GenerationFailureKind::Contradiction, raw)
    }

    /// A failed event generation reports the event line, with the line of the
    /// failure under it in the raw text. The other kinds keep their line.
    pub fn of_generation(self, kind: Kind) -> Self {
        match kind {
            Kind::Event => Self::new(GenerationFailureKind::EventNarrationFailed, self.line()),
            _ => self,
        }
    }

    /// The short line the status column shows: the kind's words, with the part the
    /// kind names.
    pub fn line(&self) -> String {
        self.kind.line(&self.named)
    }

    /// Classifies the errors of a model call and of prompt assembly. A read or a
    /// write names its own line where it fails.
    pub fn from_engine_error(error: &EngineError) -> Self {
        let kind = match error {
            EngineError::Llm(
                LlmFailure::Timeout | LlmFailure::Network { .. } | LlmFailure::Http { .. },
            ) => GenerationFailureKind::Unreachable,
            EngineError::ContextOverflow { .. }
            | EngineError::Narration(NarrationFailure::PromptBuild { .. }) => {
                GenerationFailureKind::PromptTooLong
            }
            EngineError::Llm(LlmFailure::ParseError { .. } | LlmFailure::EmptyResponse { .. })
            | EngineError::Narration(NarrationFailure::Generation { .. }) => {
                GenerationFailureKind::UnreadableAnswer
            }
            EngineError::Llm(LlmFailure::TokenBudgetSpent { .. }) => {
                GenerationFailureKind::TokenBudgetSpent
            }
            _ => GenerationFailureKind::Other,
        };
        Self::new(kind, error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Step {
    /// Narrator LLM writing the Narration.
    Narrating,
    /// Quantifier LLM evaluating which Characters are present, and movement.
    Quantifying,
    /// Event-generation LLM producing derived side effects.
    GeneratingEvent,
    /// Options LLM producing the Options to pick from.
    Options,
}
