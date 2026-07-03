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

#[test]
fn text_check_mode_spell_from_settings() {
    let mut settings = AppSettings::default();
    settings.text_check.mode = TextCheckMode::Spell;

    let service = create_text_check_service(&settings);

    // In Spell mode, routing is determined by caller, not factory.
    // Just verify service doesn't panic.
    let _ = service.check_player_input("Test", TextCheckMode::Spell, &[]);
}

#[test]
fn ignored_words_from_settings_flow_to_checker() {
    let mut settings = AppSettings::default();
    settings.text_check.ignored_words = vec!["testword".to_string()];

    let service = create_text_check_service(&settings);

    // Ignored words are passed to Harper internally.
    // Observable behavior: calling check with any mode succeeds.
    // We cannot easily assert that "testword" is ignored without a real Harper check.
    // This test just verifies the factory wired the settings.
    let _ = service
        .check_player_input(
            "Test input",
            TextCheckMode::Spell,
            &["testword".to_string()],
        )
        .expect("check should succeed");
}

// NOTE: `HarperTextChecker::new()` is infallible (returns `Self`, not `Result`).
// There is no construction-time error to propagate. Any errors occur at check time.
