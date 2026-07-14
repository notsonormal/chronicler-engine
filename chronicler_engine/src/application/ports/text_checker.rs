//! [DOC: docs/system/text_check.md]
//! TextChecker port trait and CheckResult DTO

use std::ops::Range;

use crate::domain::model::settings::TextCheckMode;
use crate::error::EngineError;

/// The result of checking a piece of text for spelling and grammar issues.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckResult {
    /// The original text that was checked.
    pub original: String,
    /// The text with all applicable suggestions applied.
    pub corrected: String,
    /// Individual issues found in the text.
    pub issues: Vec<CheckIssue>,
}

/// A single spelling or grammar issue found in text.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckIssue {
    /// Byte range in the original text where the issue occurs.
    pub span: Range<usize>,
    /// Human-readable description of the issue.
    pub message: String,
    /// The suggested replacement text, if any.
    pub suggestion: Option<String>,
    /// The kind of issue (spelling, grammar, etc.).
    pub kind: IssueKind,
}

/// Classification of a text check issue.
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

/// Port trait for text checking backends.
pub trait TextChecker: Send + Sync {
    fn check(
        &self,
        text: &str,
        mode: TextCheckMode,
        ignored_words: &[String],
    ) -> Result<Option<CheckResult>, EngineError>;
}
