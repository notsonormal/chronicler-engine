//! [DOC: docs/system/text_check.md]
//! Text checking and validation

pub mod check;
pub mod harper_backend;
pub mod types;

pub use self::check::check_player_input;
#[cfg(test)]
mod check_tests;
mod harper_backend_tests;
mod types_tests;

pub use self::types::{CheckIssue, CheckResult, IssueKind};
