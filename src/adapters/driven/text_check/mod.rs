//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Text checking and validation

pub mod harper_text_checker;

pub use self::harper_text_checker::HarperTextChecker;
#[cfg(test)]
mod harper_text_checker_tests;
