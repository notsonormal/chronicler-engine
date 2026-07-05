//! Tests for `TextCheckService` orchestrator.

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use crate::application::ports::text_checker::{CheckIssue, CheckResult, TextChecker};
use crate::application::text_check_service::TextCheckService;
use crate::domain::model::settings::TextCheckMode;
use crate::error::EngineError;

/// Records call count and last mode; returns queued responses in order.
struct StubChecker {
    responses: Mutex<VecDeque<Result<Option<CheckResult>, EngineError>>>,
    call_count: Mutex<usize>,
    last_mode: Mutex<Option<TextCheckMode>>,
}

impl StubChecker {
    fn with_ok_response(check_result: Option<CheckResult>) -> Self {
        Self {
            responses: Mutex::new(VecDeque::from([Ok(check_result)])),
            call_count: Mutex::new(0),
            last_mode: Mutex::new(None),
        }
    }

    fn with_error_response(err: EngineError) -> Self {
        Self {
            responses: Mutex::new(VecDeque::from([Err(err)])),
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
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or(Ok(None))
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
fn active_modes_route_to_checker_with_expected_mode() {
    let cases = [
        TextCheckMode::Spell,
        TextCheckMode::Grammar,
        TextCheckMode::SpellGrammar,
    ];
    for mode in &cases {
        let checker: Arc<StubChecker> = Arc::new(StubChecker::with_ok_response(None));
        let service = TextCheckService::new(checker.clone());

        service
            .check_player_input("Test", mode.clone(), &[])
            .expect("should succeed");

        assert_eq!(
            checker.call_count(),
            1,
            "mode {mode:?} should call checker once"
        );
        assert_eq!(
            checker.last_mode(),
            Some(mode.clone()),
            "mode {mode:?} should be forwarded"
        );
    }
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
