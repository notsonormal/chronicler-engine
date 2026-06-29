use crate::application::narrative_prompt::budget;
use crate::application::narrative_prompt::budget::{estimate_tokens, truncate_to_budget};

#[test]
fn test_token_budgets() {
    assert_eq!(budget::MAX_CONTEXT_TOKENS, 32768);
    assert_eq!(budget::MAX_HISTORY_TOKENS, 16000);
    assert_eq!(budget::MAX_SYSTEM_TOKENS, 1024);
    assert_eq!(budget::SAFETY_MARGIN_TOKENS, 256);
    assert_eq!(budget::MIN_INPUT_BUDGET_TOKENS, 512);
}

#[test]
fn test_estimate_tokens_empty() {
    assert_eq!(estimate_tokens(""), 0);
}

#[test]
fn test_estimate_tokens_single_char() {
    assert_eq!(estimate_tokens("a"), 1);
}

#[test]
fn test_estimate_tokens_exact_four() {
    assert_eq!(estimate_tokens("abcd"), 1);
}

#[test]
fn test_estimate_tokens_five_chars() {
    assert_eq!(estimate_tokens("abcde"), 2);
}

#[test]
fn test_estimate_tokens_many_chars() {
    let text = "This is a longer text string with many characters.";
    let tokens = estimate_tokens(text);
    assert_eq!(tokens, 13);
}

#[test]
fn test_truncate_to_budget_no_truncate_needed() {
    let text = "Short text";
    let result = truncate_to_budget(text, 10);
    assert_eq!(result, "Short text");
}

#[test]
fn test_truncate_to_budget_exact_fit() {
    let text = "abcd";
    let result = truncate_to_budget(text, 1);
    assert_eq!(result, "abcd");
}

#[test]
fn test_truncate_to_budget_truncate() {
    let text = "1234567890";
    let result = truncate_to_budget(text, 2);
    assert_eq!(result, "34567890");
}

#[test]
fn test_truncate_to_budget_preserves_recent() {
    let text = "The quick brown fox jumps over the lazy dog.";
    let result = truncate_to_budget(text, 5);
    assert!(result.ends_with("the lazy dog."));
}

#[test]
fn test_truncate_to_budget_zero_tokens() {
    let text = "Some text";
    let result = truncate_to_budget(text, 0);
    assert_eq!(result, "");
}
