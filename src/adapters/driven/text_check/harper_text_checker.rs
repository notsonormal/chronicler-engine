//! [DOC: docs/diataxis/reference/game_flow.md]
//! Harper text check adapter implementing TextChecker port

use std::sync::Arc;

use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::{FstDictionary, MutableDictionary};
use harper_core::{Document, MergedDictionary};

use crate::adapters::driven::utils::harper::{
    apply_suggestions, char_span_to_byte_span, lint_kind_to_issue_kind,
};
use crate::error::EngineError;
use crate::domain::model::settings::TextCheckMode;
use crate::application::ports::text_checker::{CheckIssue, CheckResult, TextChecker};

pub struct HarperTextChecker {
    ignored_words: Vec<String>,
    dictionary: std::sync::OnceLock<Arc<MergedDictionary>>,
}

impl HarperTextChecker {
    pub fn new(ignored_words: &[String]) -> Self {
        Self {
            ignored_words: ignored_words.to_vec(),
            dictionary: std::sync::OnceLock::new(),
        }
    }

    fn merged(&self) -> Arc<MergedDictionary> {
        self.dictionary
            .get_or_init(|| {
                let mut merged = MergedDictionary::new();
                merged.add_dictionary(FstDictionary::curated());
                if !self.ignored_words.is_empty() {
                    let mut user_dict = MutableDictionary::new();
                    for word in &self.ignored_words {
                        user_dict.append_word_str(word, harper_core::WordMetadata::default());
                    }
                    merged.add_dictionary(Arc::new(user_dict));
                }
                Arc::new(merged)
            })
            .clone()
    }
}

impl TextChecker for HarperTextChecker {
    fn check(
        &self,
        text: &str,
        mode: TextCheckMode,
        _ignored_words: &[String],
    ) -> Result<Option<CheckResult>, EngineError> {
        if mode == TextCheckMode::Disabled {
            return Ok(None);
        }

        let dictionary = self.merged();
        let document = Document::new_plain_english(text, dictionary.as_ref());
        let mut linter = LintGroup::new_curated(dictionary);
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
