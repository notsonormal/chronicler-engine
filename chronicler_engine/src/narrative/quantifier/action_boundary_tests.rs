use crate::narrative::quantifier::core::action_boundary_contains;

fn make_boundary_chars() -> std::collections::HashSet<char> {
    [
        ' ', '.', ',', '!', '?', '\n', '\t', '\r', '\'', '"', ':', ';',
    ]
    .into_iter()
    .collect()
}

#[test]
fn test_action_boundary_exact_match_at_start() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("hello world", "hello", &boundary_chars);
    assert!(result, "Should match at start of text");
}

#[test]
fn test_action_boundary_exact_match_at_end() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("hello world", "world", &boundary_chars);
    assert!(result, "Should match at end of text");
}

#[test]
fn test_action_boundary_match_in_middle() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("hello world here", "world", &boundary_chars);
    assert!(result, "Should match in middle with spaces on both sides");
}

#[test]
fn test_action_boundary_no_match_prefix() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("announcement", "ann", &boundary_chars);
    assert!(
        !result,
        "Should not match when substring is prefix of longer word"
    );
}

#[test]
fn test_action_boundary_no_match_suffix() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("maryann", "ann", &boundary_chars);
    assert!(
        !result,
        "Should not match when substring is suffix of longer word"
    );
}

#[test]
fn test_action_boundary_no_match_mid_word() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("canny", "ann", &boundary_chars);
    assert!(
        !result,
        "Should not match when substring appears but surrounded by non-boundary chars"
    );
}

#[test]
fn test_action_boundary_match_with_comma() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("hello,world", "world", &boundary_chars);
    assert!(result, "Should match with comma as boundary");
}

#[test]
fn test_action_boundary_match_with_period() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("hello.world", "world", &boundary_chars);
    assert!(result, "Should match with period as boundary");
}

#[test]
fn test_action_boundary_match_with_exclamation() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("hello!world", "world", &boundary_chars);
    assert!(result, "Should match with exclamation as boundary");
}

#[test]
fn test_action_boundary_match_with_question_mark() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("hello?world", "world", &boundary_chars);
    assert!(result, "Should match with question mark as boundary");
}

#[test]
fn test_action_boundary_empty_text() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("", "hello", &boundary_chars);
    assert!(!result, "Should not match empty text");
}

#[test]
fn test_action_boundary_empty_substring() {
    let boundary_chars = make_boundary_chars();
    // Empty substring finds at position 0, but char after position 0 is not a boundary
    let result = action_boundary_contains("hello world", "", &boundary_chars);
    assert!(
        !result,
        "Empty substring should not match when followed by non-boundary char"
    );
}

#[test]
fn test_action_boundary_both_empty() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("", "", &boundary_chars);
    assert!(result, "Empty text and substring should match");
}

#[test]
fn test_action_boundary_no_match_not_found() {
    let boundary_chars = make_boundary_chars();
    let result = action_boundary_contains("hello world", "xyz", &boundary_chars);
    assert!(!result, "Should not match when substring not found");
}
