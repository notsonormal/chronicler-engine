//! [DOC: docs/diataxis/reference/game_flow.md]
//! Text checking and validation

pub mod harper_text_checker;

pub use self::harper_text_checker::HarperTextChecker;
#[cfg(test)]
mod harper_text_checker_tests;

// Re-export types from the port for backwards compatibility
pub use crate::application::ports::text_checker::{CheckIssue, CheckResult, IssueKind};
