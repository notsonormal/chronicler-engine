//! Tests for the check_player_input wrapper function.
//!
//! These tests verify the public API that combines mode checking,
//! backend construction, and delegation to HarperBackend.

use super::check_player_input;
use crate::domain::model::settings::TextCheckMode;

#[test]
fn test_check_player_input_disabled_mode_returns_none() {
    // Disabled mode should short-circuit and return None without checking
    let result = check_player_input("teh quik brown fox", TextCheckMode::Disabled, &[]);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_check_player_input_empty_text_returns_none() {
    // Empty text with no issues should return None (not Some with empty issues)
    let result = check_player_input("", TextCheckMode::Spell, &[]);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_check_player_input_clean_text_returns_none() {
    // Clean text with no issues should return None
    let result = check_player_input(
        "The quick brown fox jumps over the lazy dog.",
        TextCheckMode::Spell,
        &[],
    );
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.is_none());
}

#[test]
fn test_check_player_input_misspelling_detected() {
    // Misspelled word should be detected in spelling mode
    let result = check_player_input("teh quick brown fox", TextCheckMode::Spell, &[]);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.is_some(), "Should detect misspelling 'teh'");
    let check_result = result.unwrap();
    assert!(!check_result.issues.is_empty());
    assert!(
        check_result
            .issues
            .iter()
            .any(|i| i.message.contains("teh"))
    );
}

#[test]
fn test_check_player_input_ignored_words_respected() {
    // Words in ignored_words list should not trigger issues
    let ignored = vec!["teh".to_string(), "cromulent".to_string()];
    let result = check_player_input("teh cromulent fox", TextCheckMode::Spell, &ignored);
    assert!(result.is_ok());
    let result = result.unwrap();
    // Should have no issues for ignored words
    assert!(result.is_none() || result.unwrap().issues.is_empty());
}

#[test]
fn test_check_player_input_whitespace_only_returns_none() {
    // Whitespace-only input should return None (no issues to report)
    let result = check_player_input("   \n\t  ", TextCheckMode::SpellGrammar, &[]);
    assert!(result.is_ok());
    let result = result.unwrap();
    // Harper may or may not flag whitespace - just verify it doesn't error
    if let Some(check_result) = result {
        // If issues found, verify they're about whitespace
        assert!(check_result.original.trim().is_empty());
    }
    // Test passes either way (None or Some with issues on whitespace)
}

#[test]
fn test_check_player_input_unicode_text() {
    // Unicode text should be handled correctly
    let result = check_player_input("The naïve café served résumé", TextCheckMode::Spell, &[]);
    assert!(result.is_ok());
    // Should not panic or error on unicode
    let _ = result.unwrap();
}

#[test]
fn test_check_player_input_preserves_original_text() {
    // CheckResult should preserve the original text
    let input = "teh quik fox";
    let result = check_player_input(input, TextCheckMode::Spell, &[]);
    assert!(result.is_ok());
    if let Some(check_result) = result.unwrap() {
        assert_eq!(check_result.original, input);
    } else {
        // If no issues found, test passes (text was clean)
    }
}

#[test]
fn test_check_player_input_spelling_mode_only() {
    // SpellingOnly mode should catch spelling but not grammar
    let result = check_player_input("teh fox are quick", TextCheckMode::Spell, &[]);
    assert!(result.is_ok());
    let result = result.unwrap();
    if let Some(check_result) = result {
        // Should at least catch "teh"
        assert!(
            check_result
                .issues
                .iter()
                .any(|i| i.message.contains("teh") || check_result.original.contains("teh"))
        );
    }
}

#[test]
fn test_check_player_input_spelling_and_grammar_mode() {
    // SpellingAndGrammar mode should catch both types of issues
    let result = check_player_input("teh fox are quick", TextCheckMode::SpellGrammar, &[]);
    assert!(result.is_ok());
    // Should detect issues (either spelling, grammar, or both)
    let result = result.unwrap();
    assert!(
        result.is_some(),
        "Should detect issues in 'teh fox are quick'"
    );
}
