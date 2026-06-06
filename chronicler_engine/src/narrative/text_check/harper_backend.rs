//! [DOC: docs/system/text_check.md]

use std::sync::Arc;

use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::{FstDictionary, MutableDictionary};
use harper_core::{Document, MergedDictionary};

use crate::error::EngineError;
use crate::model::settings::TextCheckMode;

use super::types::{CheckIssue, CheckResult, IssueKind};

pub struct HarperBackend {
    dictionary: Arc<MergedDictionary>,
}

impl HarperBackend {
    pub fn new(ignored_words: &[String]) -> Self {
        let mut merged = MergedDictionary::new();
        merged.add_dictionary(FstDictionary::curated());

        if !ignored_words.is_empty() {
            let mut user_dict = MutableDictionary::new();
            for word in ignored_words {
                user_dict.append_word_str(word, harper_core::WordMetadata::default());
            }
            merged.add_dictionary(Arc::new(user_dict));
        }

        Self {
            dictionary: Arc::new(merged),
        }
    }

    pub fn check(
        &self,
        text: &str,
        mode: TextCheckMode,
    ) -> Result<Option<CheckResult>, EngineError> {
        if mode == TextCheckMode::Disabled {
            return Ok(None);
        }

        let document = Document::new_plain_english(text, self.dictionary.as_ref());
        let mut linter = LintGroup::new_curated(self.dictionary.clone());
        linter.config.set_rule_enabled("AvoidCurses", false);

        match mode {
            TextCheckMode::Spell => {
                linter.set_all_rules_to(Some(false));
                linter.config.set_rule_enabled("SpellCheck", true);
            }
            TextCheckMode::Grammar => {
                linter.config.set_rule_enabled("SpellCheck", false);
            }
            TextCheckMode::SpellGrammar => {
                // Default curated config is already what we want.
            }
            TextCheckMode::Disabled => unreachable!(),
        }

        let lints = linter.lint(&document);
        if lints.is_empty() {
            return Ok(None);
        }

        let issues = lints
            .iter()
            .map(|lint| {
                let span_chars = lint.span.start..lint.span.end;
                let span_bytes = char_span_to_byte_span(text, span_chars);
                let suggestion = lint.suggestions.first().and_then(|s| match s {
                    harper_core::linting::Suggestion::ReplaceWith(chars) => {
                        Some(chars.iter().collect())
                    }
                    harper_core::linting::Suggestion::InsertAfter(chars) => {
                        Some(chars.iter().collect())
                    }
                    harper_core::linting::Suggestion::Remove => None,
                });

                CheckIssue {
                    span: span_bytes,
                    message: lint.message.clone(),
                    suggestion,
                    kind: lint_kind_to_issue_kind(lint.lint_kind),
                }
            })
            .collect::<Vec<_>>();

        let corrected = apply_suggestions(text, &lints);

        Ok(Some(CheckResult {
            original: text.to_string(),
            corrected,
            issues,
        }))
    }
}

fn lint_kind_to_issue_kind(kind: harper_core::linting::LintKind) -> IssueKind {
    use harper_core::linting::LintKind as HK;
    match kind {
        HK::Spelling => IssueKind::Spelling,
        HK::Capitalization => IssueKind::Capitalization,
        HK::Formatting => IssueKind::Formatting,
        HK::Repetition => IssueKind::Style,
        HK::Readability => IssueKind::Style,
        HK::Enhancement => IssueKind::Grammar,
        HK::WordChoice => IssueKind::Grammar,
        HK::Style => IssueKind::Style,
        HK::Miscellaneous => IssueKind::Other,
    }
}

fn char_span_to_byte_span(text: &str, char_span: std::ops::Range<usize>) -> std::ops::Range<usize> {
    let mut byte_start = None;
    let mut byte_end = None;

    for (char_idx, (byte_idx, _)) in text.char_indices().enumerate() {
        if char_idx == char_span.start {
            byte_start = Some(byte_idx);
        }
        if char_idx == char_span.end {
            byte_end = Some(byte_idx);
            break;
        }
    }

    let start = byte_start.unwrap_or(0);
    let end = byte_end.unwrap_or(text.len());
    start..end
}

fn apply_suggestions(text: &str, lints: &[harper_core::linting::Lint]) -> String {
    let mut chars: Vec<char> = text.chars().collect();
    let mut sorted_lints: Vec<_> = lints.iter().collect();
    sorted_lints.sort_by_key(|l| std::cmp::Reverse(l.span.start));

    for lint in sorted_lints {
        if let Some(suggestion) = lint.suggestions.first() {
            // Span in the current char vec. Since we're working backwards,
            // the span hasn't shifted yet.
            suggestion.apply(lint.span, &mut chars);
        }
    }

    chars.into_iter().collect()
}
