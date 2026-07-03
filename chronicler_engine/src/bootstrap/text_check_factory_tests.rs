//! Unit tests for `create_text_check_service` factory function.

use crate::bootstrap::text_check_factory::create_text_check_service;
use crate::domain::model::settings::{AppSettings, TextCheckMode};

#[test]
fn production_path_returns_text_check_service() {
    let settings = AppSettings::default();
    let service = create_text_check_service(&settings);

    // Verify service is constructed (not Result - factory is infallible)
    // Test observable behavior: Disabled mode short-circuits
    let result = service
        .check_player_input("Test", TextCheckMode::Disabled, &[])
        .expect("check should succeed");
    assert!(result.is_none());
}

#[test]
fn text_check_mode_disabled_from_settings() {
    let mut settings = AppSettings::default();
    settings.text_check.mode = TextCheckMode::Disabled;

    let service = create_text_check_service(&settings);

    // In Disabled mode, check_player_input always returns None
    let result = service
        .check_player_input("Test with errors", TextCheckMode::Disabled, &[])
        .expect("check should succeed");
    assert!(result.is_none(), "should short-circuit in Disabled mode");
}

// NOTE: `HarperTextChecker::new()` is infallible (returns `Self`, not `Result`).
// There is no construction-time error to propagate. Any errors occur at check time.
// Spell-mode routing + ignored-words wiring are exercised by `harper_text_checker_tests.rs`.
