use std::ops::Range;

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
    Spelling,
    Grammar,
    Capitalization,
    Style,
    Formatting,
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
