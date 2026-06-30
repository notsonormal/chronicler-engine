//! [DOC: docs/system/text_check.md]
//! Text checking and validation

pub mod harper_text_checker;
pub mod types;

pub use self::harper_text_checker::HarperTextChecker;
#[cfg(test)]
mod harper_text_checker_tests;
// types_tests removed - types tests moved to port

// Re-export types from the port for backwards compatibility
pub use crate::application::ports::text_checker::{CheckIssue, CheckResult, IssueKind};
