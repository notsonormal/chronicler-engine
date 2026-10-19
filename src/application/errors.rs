//! [DOC: docs/diataxis/reference/game_flow.md]
//! ApplicationError — the error envelope of the application services.

use crate::error::{EngineError, InternalError};

pub enum ApplicationError {
    /// Input failed domain validation; payload is the user-facing reason.
    Validation(String),
    /// Engine-layer failure surfaced through the application service.
    Engine(EngineError),
}

impl ApplicationError {
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Engine(EngineError::Internal(InternalError::new(msg)))
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

    /// Returns true for validation errors and `EngineError::WorldHasGames`.
    /// `WorldHasGames` surfaces as user-actionable (e.g. cannot delete world with active games).
    pub fn is_user_displayable(&self) -> bool {
        matches!(
            self,
            Self::Validation(_) | Self::Engine(EngineError::WorldHasGames { .. })
        )
    }
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(msg) => write!(f, "{msg}"),
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
