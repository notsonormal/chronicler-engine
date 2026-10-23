//! Tests for the `test_hygiene.rs` guardrails.

use crate::test_hygiene::{check_no_ignored_tests, check_sleep_has_reason};

#[test]
fn test_check_no_ignored_tests_catches_every_ignore_form() {
    let content = "\
#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn bare() {}

    #[test]
    #[ignore = \"flaky\"]
    fn with_reason() {}

    #[test]
    #[cfg_attr(windows, ignore)]
    fn conditional() {}
}
";
    let violations = check_no_ignored_tests("http/foo.rs", content);
    let found: Vec<(usize, bool)> = violations
        .iter()
        .map(|v| (v.line, v.message.contains("tests/llm/")))
        .collect();
    assert_eq!(found, vec![(4, true), (8, true), (12, true)]);
    assert!(violations[0].message.contains("`bare`"));
    assert!(violations[1].message.contains("`with_reason`"));
    assert!(violations[2].message.contains("`conditional`"));
}

#[test]
fn test_check_no_ignored_tests_allows_the_word_outside_an_ignore_attribute() {
    let content = "\
// #[ignore] in a comment
#[test]
#[cfg_attr(feature = \"ignore\", allow(dead_code))]
fn fixture_text() {
    let source = \"#[test]\\n#[ignore]\\nfn x() {}\";
    assert!(!source.is_empty());
}
";
    assert!(check_no_ignored_tests("infrastructure/x.rs", content).is_empty());
}

#[test]
fn test_check_no_ignored_tests_reports_a_file_it_cannot_parse() {
    let violations = check_no_ignored_tests("http/broken.rs", "fn broken( {");
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("could not parse"));
}

#[test]
fn test_check_sleep_has_reason_catches_a_bare_sleep() {
    let content = "\
async fn waits() {
    let ready = false;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    std::thread::sleep(std::time::Duration::from_millis(5));
}
";
    let violations = check_sleep_has_reason("http/foo.rs", content);
    let lines: Vec<usize> = violations.iter().map(|v| v.line).collect();
    assert_eq!(lines, vec![3, 4]);
    assert!(violations[0].message.contains("tests/test_utils/"));
}

#[test]
fn test_check_sleep_has_reason_allows_a_sleep_with_a_comment_above() {
    let content = "\
async fn waits() {
    // Give the submit a chance to reach the stub.
    tokio::time::sleep(SETTLE).await;
    let text = \"tokio::time::sleep(x)\";
    // sleep(1) in a comment
    assert!(!text.is_empty());
}
";
    assert!(check_sleep_has_reason("browser/stub/x.rs", content).is_empty());
}

#[test]
fn test_check_sleep_has_reason_reports_a_file_it_cannot_parse() {
    let violations = check_sleep_has_reason("http/broken.rs", "fn broken( {");
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("could not parse"));
}
