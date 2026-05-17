use crate::model::settings::TextCheckMode;
use crate::narrative::text_check::harper_backend::HarperBackend;

#[test]
fn detects_misspelling() {
    let backend = HarperBackend::new(&[]);
    let result = backend
        .check("go to the casle", TextCheckMode::Spell)
        .unwrap();
    assert!(result.is_some());
    let result = result.unwrap();
    assert!(!result.issues.is_empty());
    assert_eq!(result.corrected, "go to the castle");
}

#[test]
fn no_issues_on_clean_text() {
    let backend = HarperBackend::new(&[]);
    let result = backend
        .check("go to the castle", TextCheckMode::SpellGrammar)
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn disabled_returns_none() {
    let backend = HarperBackend::new(&[]);
    let result = backend
        .check("go to the casle", TextCheckMode::Disabled)
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn ignored_words_are_respected() {
    let backend = HarperBackend::new(&["casle".to_string()]);
    let result = backend
        .check("go to the casle", TextCheckMode::Spell)
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn detects_multiple_misspellings() {
    let backend = HarperBackend::new(&[]);
    let result = backend
        .check("Yiu igore her and move inside", TextCheckMode::Spell)
        .unwrap();
    assert!(result.is_some());
    let result = result.unwrap();
    assert!(
        result.issues.len() >= 2,
        "Expected at least 2 issues, got {}: {:?}",
        result.issues.len(),
        result.issues
    );
    assert_eq!(
        result.corrected, "Y's ignore her and move inside",
        "Corrected text was wrong: {}",
        result.corrected
    );
}

#[test]
fn detects_two_unambiguous_misspellings() {
    let backend = HarperBackend::new(&[]);
    let result = backend
        .check("go to the casle and teh towre", TextCheckMode::Spell)
        .unwrap();
    assert!(result.is_some());
    let result = result.unwrap();
    assert_eq!(
        result.issues.len(),
        3,
        "Expected 3 issues: {:?}",
        result.issues
    );
    assert_eq!(
        result.corrected, "go to the castle and ten tore",
        "Corrected text was wrong: {}",
        result.corrected
    );
}

#[test]
fn spellgrammar_finds_spelling_and_grammar_issues() {
    let backend = HarperBackend::new(&[]);
    let result = backend
        .check("He dont go to the casle", TextCheckMode::SpellGrammar)
        .unwrap();
    assert!(result.is_some());
    let result = result.unwrap();
    assert!(
        result.issues.len() >= 2,
        "Expected at least 2 issues, got {}: {:?}",
        result.issues.len(),
        result.issues
    );
    assert!(
        result.corrected.contains("castle"),
        "Corrected text should contain 'castle', got: {}",
        result.corrected
    );
}
