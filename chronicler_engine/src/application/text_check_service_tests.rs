//! Tests for `TextCheckService` orchestrator.

use std::sync::{Arc, Mutex};

use crate::application::ports::text_checker::{CheckIssue, CheckResult, TextChecker};
use crate::application::text_check_service::TextCheckService;
use crate::domain::model::settings::TextCheckMode;
use crate::error::EngineError;

/// Stub checker that records calls and returns a configurable result.
struct StubChecker {
    response: Mutex<Option<Result<Option<CheckResult>, EngineError>>>,
    call_count: Mutex<usize>,
    last_mode: Mutex<Option<TextCheckMode>>,
}

impl StubChecker {
    fn with_ok_response(check_result: Option<CheckResult>) -> Self {
        Self {
            response: Mutex::new(Some(Ok(check_result))),
            call_count: Mutex::new(0),
            last_mode: Mutex::new(None),
        }
    }

    fn with_error_response(err: EngineError) -> Self {
        Self {
            response: Mutex::new(Some(Err(err))),
            call_count: Mutex::new(0),
            last_mode: Mutex::new(None),
        }
    }

    fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    fn last_mode(&self) -> Option<TextCheckMode> {
        self.last_mode.lock().unwrap().clone()
    }
}

impl TextChecker for StubChecker {
    fn check(
        &self,
        _text: &str,
        mode: TextCheckMode,
        _ignored_words: &[String],
    ) -> Result<Option<CheckResult>, EngineError> {
        *self.call_count.lock().unwrap() += 1;
        *self.last_mode.lock().unwrap() = Some(mode);
        self.response.lock().unwrap().take().unwrap_or(Ok(None))
    }
}

#[test]
fn happy_path_issues_found() {
    let checker: Arc<StubChecker> = Arc::new(StubChecker::with_ok_response(Some(CheckResult {
        original: "Hello".to_string(),
        corrected: "Hello".to_string(),
        issues: vec![CheckIssue {
            span: 0..5,
            message: "Test issue".to_string(),
            suggestion: None,
            kind: crate::application::ports::text_checker::IssueKind::Spelling,
        }],
    })));

    let service = TextCheckService::new(checker);
    let result = service
        .check_player_input("Hello", TextCheckMode::Spell, &[])
        .expect("should succeed");

    assert!(result.is_some());
    let check_result = result.unwrap();
    assert_eq!(check_result.issues.len(), 1);
}

#[test]
fn disabled_mode_returns_none() {
    let checker: Arc<StubChecker> = Arc::new(StubChecker::with_ok_response(Some(CheckResult {
        original: "Hello".to_string(),
        corrected: "Hello".to_string(),
        issues: vec![],
    })));
    let service = TextCheckService::new(checker);

    let result = service
        .check_player_input("Hello", TextCheckMode::Disabled, &[])
        .expect("should succeed");

    assert!(result.is_none());
}

#[test]
fn spell_mode_routes_to_checker() {
    let checker: Arc<StubChecker> = Arc::new(StubChecker::with_ok_response(None));
    let service = TextCheckService::new(checker.clone());

    let ignored: Vec<String> = vec!["ignored".to_string()];
    service
        .check_player_input("Test", TextCheckMode::Spell, &ignored)
        .expect("should succeed");

    assert_eq!(checker.call_count(), 1);
    assert_eq!(checker.last_mode(), Some(TextCheckMode::Spell));
}

#[test]
fn grammar_mode_routes_to_checker() {
    let checker: Arc<StubChecker> = Arc::new(StubChecker::with_ok_response(None));
    let service = TextCheckService::new(checker.clone());

    service
        .check_player_input("Test", TextCheckMode::Grammar, &[])
        .expect("should succeed");

    assert_eq!(checker.call_count(), 1);
    assert_eq!(checker.last_mode(), Some(TextCheckMode::Grammar));
}

#[test]
fn spell_grammar_mode_routes_to_checker() {
    let checker: Arc<StubChecker> = Arc::new(StubChecker::with_ok_response(None));
    let service = TextCheckService::new(checker.clone());

    service
        .check_player_input("Test", TextCheckMode::SpellGrammar, &[])
        .expect("should succeed");

    assert_eq!(checker.call_count(), 1);
    assert_eq!(checker.last_mode(), Some(TextCheckMode::SpellGrammar));
}

#[test]
fn empty_input_routes_to_checker() {
    let checker: Arc<StubChecker> = Arc::new(StubChecker::with_ok_response(None));
    let service = TextCheckService::new(checker.clone());

    service
        .check_player_input("", TextCheckMode::Spell, &[])
        .expect("should succeed");

    assert_eq!(checker.call_count(), 1);
}

#[test]
fn checker_error_propagates() {
    let err = EngineError::Io("checker failed".to_string());
    let checker: Arc<StubChecker> = Arc::new(StubChecker::with_error_response(err));
    let service = TextCheckService::new(checker);

    let result = service.check_player_input("Hello", TextCheckMode::Spell, &[]);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), EngineError::Io(_)));
}

