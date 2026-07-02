//! Tests for `TextChecker` port trait — polymorphism and dispatch.
//!
//! SCOPE GUARD: This file covers ONLY trait polymorphism. HarperTextChecker
//! happy-path tests already exist at
//! `src/adapters/driven/text_check/harper_text_checker_tests.rs` — do NOT
//! duplicate them here.

use std::sync::Arc;

use crate::application::ports::text_checker::{CheckResult, TextChecker};
use crate::domain::model::settings::TextCheckMode;
use crate::error::EngineError;

/// Stub checker that returns a fixed response.
struct StubCheckerA;

impl TextChecker for StubCheckerA {
    fn check(
        &self,
        _text: &str,
        _mode: TextCheckMode,
        _ignored_words: &[String],
    ) -> Result<Option<CheckResult>, EngineError> {
        Ok(Some(CheckResult {
            original: "A".to_string(),
            corrected: "A".to_string(),
            issues: vec![],
        }))
    }
}

/// Stub checker that returns a different fixed response.
struct StubCheckerB;

impl TextChecker for StubCheckerB {
    fn check(
        &self,
        _text: &str,
        _mode: TextCheckMode,
        _ignored_words: &[String],
    ) -> Result<Option<CheckResult>, EngineError> {
        Ok(Some(CheckResult {
            original: "B".to_string(),
            corrected: "B".to_string(),
            issues: vec![],
        }))
    }
}

/// Stub checker that returns None.
struct StubCheckerNone;

impl TextChecker for StubCheckerNone {
    fn check(
        &self,
        _text: &str,
        _mode: TextCheckMode,
        _ignored_words: &[String],
    ) -> Result<Option<CheckResult>, EngineError> {
        Ok(None)
    }
}

#[test]
fn trait_dispatch_between_impls() {
    // Verify that `dyn TextChecker` dispatches to the correct impl.
    let checker_a: Arc<dyn TextChecker> = Arc::new(StubCheckerA);
    let checker_b: Arc<dyn TextChecker> = Arc::new(StubCheckerB);
    let checker_none: Arc<dyn TextChecker> = Arc::new(StubCheckerNone);

    let result_a = checker_a
        .check("test", TextCheckMode::Spell, &[])
        .expect("should succeed");
    let result_b = checker_b
        .check("test", TextCheckMode::Spell, &[])
        .expect("should succeed");
    let result_none = checker_none
        .check("test", TextCheckMode::Spell, &[])
        .expect("should succeed");

    assert!(result_a.is_some());
    assert_eq!(result_a.as_ref().unwrap().original, "A");

    assert!(result_b.is_some());
    assert_eq!(result_b.as_ref().unwrap().original, "B");

    assert!(result_none.is_none());
}

#[test]
fn method_signature_compiles() {
    // This test exists purely to ensure the trait method signature compiles.
    // If the signature changes, this file will fail to compile.
    fn _takes_checker(_checker: &dyn TextChecker) {}

    let checker: Arc<dyn TextChecker> = Arc::new(StubCheckerA);
    _takes_checker(checker.as_ref());
}
