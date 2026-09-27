//! Tests for `handler_helpers.rs` shared handler helpers.

use super::handler_helpers::opt_string;

#[test]
fn test_opt_string_empty_returns_none() {
    assert_eq!(opt_string(""), None);
}

#[test]
fn test_opt_string_non_empty_returns_some() {
    assert_eq!(opt_string("sk-test123"), Some("sk-test123".to_string()));
    assert_eq!(opt_string("   "), Some("   ".to_string()));
}
