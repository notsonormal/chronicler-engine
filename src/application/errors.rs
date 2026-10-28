//! [DOC: docs/diataxis/reference/game_flow.md]
//! ApplicationError — the error envelope of the application services.

use crate::application::generation::gate::Refusal;
use crate::error::EngineError;

pub enum ApplicationError {
    /// Input failed domain validation; payload is the user-facing reason.
    Validation(String),
    /// A refusal of the writer rule, kept as it is so the HTTP layer answers it as
    /// it answers any refusal.
    Refused(Refusal),
    /// Engine-layer failure surfaced through the application service.
    Engine(EngineError),
}

impl ApplicationError {
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Lifts an engine error, keeping a domain refusal on the variant the HTTP
    /// layer renders as a user-facing rejection. Distinct from the `From`
    /// impl, which keeps every engine error on `Engine` for a `?` conversion.
    pub fn from_engine_error(error: EngineError) -> Self {
        match error {
            EngineError::Validation(message) => Self::Validation(message),
            other => Self::Engine(other),
        }
    }

    /// The engine error of a caller that has no use for the envelope: a validation or
    /// a refusal keeps its kind as `EngineError::Validation`.
    pub fn into_engine_error(self) -> EngineError {
        match self {
            Self::Validation(message) => EngineError::Validation(message),
            refused @ Self::Refused(_) => EngineError::Validation(refused.to_string()),
            Self::Engine(error) => error,
        }
    }
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(msg) => write!(f, "{msg}"),
            Self::Refused(refusal) => write!(f, "{}", refusal.line()),
            Self::Engine(e) => write!(f, "{e}"),
        }
    }
}

impl std::fmt::Debug for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for ApplicationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Engine(e) => Some(e),
            _ => None,
        }
    }
}

impl From<EngineError> for ApplicationError {
    fn from(e: EngineError) -> Self {
        Self::Engine(e)
    }
}
