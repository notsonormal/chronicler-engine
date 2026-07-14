//! [DOC: docs/system/game_flow.md]
//! ApplicationError + ProcessActionResult — error envelope and action-result tri-state.

use crate::error::EngineError;

pub enum ApplicationError {
    /// Input failed domain validation; payload is the user-facing reason.
    Validation(String),
    /// Engine-layer failure surfaced through the application service.
    Engine(EngineError),
    /// Server shutting down; reject new work.
    ShuttingDown,
    /// Another generation already in progress for this game; caller must retry or back off.
    ConcurrentGeneration,
}

impl ApplicationError {
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
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
            Self::ShuttingDown => write!(f, "Server is shutting down"),
            Self::ConcurrentGeneration => write!(f, "Generation in progress"),
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

#[derive(Debug)]
pub enum ProcessActionResult {
    /// Generation task spawned and registered.
    Started,
    /// Rejected: another generation holds the slot for this game.
    ConcurrentGeneration,
    /// Rejected: server is shutting down.
    ShuttingDown,
}
