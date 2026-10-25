//! [DOC: docs/diataxis/reference/game_flow.md]
//! TextChecker port trait and CheckResult DTO

use std::ops::Range;

use crate::domain::model::settings::TextCheckMode;
use crate::error::EngineError;

#[derive(Debug, Clone, PartialEq)]
pub struct CheckResult {
    pub original: String,
    pub corrected: String,
    pub issues: Vec<CheckIssue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckIssue {
    pub span: Range<usize>,
    pub message: String,
    pub suggestion: Option<String>,
    pub kind: IssueKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IssueKind {
    /// Word misspelled; suggestion is the corrected spelling.
    Spelling,
    /// Grammar rule violated (subject-verb, tense, agreement).
    Grammar,
    /// Capitalization incorrect for context (proper noun, sentence start).
    Capitalization,
    /// Style preference issue (passive voice, wordiness).
    Style,
    /// Formatting issue (whitespace, punctuation, structure).
    Formatting,
    /// Issue that doesn't fit the other categories; `message` carries details.
    Other,
}

impl std::fmt::Display for IssueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spelling => write!(f, "spell"),
            Self::Grammar => write!(f, "grammar"),
            Self::Capitalization => write!(f, "capitalization"),
            Self::Style => write!(f, "style"),
            Self::Formatting => write!(f, "formatting"),
            Self::Other => write!(f, "other"),
        }
    }
}

pub trait TextChecker: Send + Sync {
    fn check(
        &self,
        text: &str,
        mode: TextCheckMode,
        ignored_words: &[String],
    ) -> Result<Option<CheckResult>, EngineError>;
}
