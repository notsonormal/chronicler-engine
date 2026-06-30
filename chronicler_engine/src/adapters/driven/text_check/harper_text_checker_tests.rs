#[cfg(test)]
use crate::domain::model::settings::TextCheckMode;
#[cfg(test)]
use crate::adapters::driven::text_check::HarperTextChecker;
#[cfg(test)]
use crate::application::ports::text_checker::TextChecker;

#[test]
fn detects_misspelling() {
    let checker = HarperTextChecker::new(&[]);
    let result = checker
        .check("go to the casle", TextCheckMode::Spell, &[])
        .unwrap();
    assert!(result.is_some());
    let result = result.unwrap();
    assert!(!result.issues.is_empty());
    assert_eq!(result.corrected, "go to the castle");
}

#[test]
fn no_issues_on_clean_text() {
    let checker = HarperTextChecker::new(&[]);
    let result = checker
        .check("go to the castle", TextCheckMode::SpellGrammar, &[])
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn disabled_returns_none() {
    let checker = HarperTextChecker::new(&[]);
    let result = checker
        .check("go to the casle", TextCheckMode::Disabled, &[])
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn ignored_words_are_respected() {
    let checker = HarperTextChecker::new(&["casle".to_string()]);
    let result = checker
        .check("go to the casle", TextCheckMode::Spell, &[])
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn detects_multiple_misspellings() {
    let checker = HarperTextChecker::new(&[]);
    let result = checker
        .check("Yiu igore her and move inside", TextCheckMode::Spell, &[])
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
    let checker = HarperTextChecker::new(&[]);
    let result = checker
        .check("go to the casle and teh towre", TextCheckMode::Spell, &[])
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
    let checker = HarperTextChecker::new(&[]);
    let result = checker
        .check("He dont go to the casle", TextCheckMode::SpellGrammar, &[])
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
