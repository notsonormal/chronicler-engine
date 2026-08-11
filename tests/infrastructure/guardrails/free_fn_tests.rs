//! Tests for `free_fn.rs` guardrail.

use crate::free_fn::*;

#[test]
fn test_check_free_fn_location_catches_violation() {
    let violations = check_free_fn_location(
        "src/narrative/parser.rs",
        "pub fn parse_input(s: &str) -> &str { s }\n",
    );
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("parse_input"));
    assert!(violations[0].message.contains("mappers, utils, builders"));
}

#[test]
fn test_check_free_fn_location_flags_async_fn() {
    let violations =
        check_free_fn_location("src/narrative/parser.rs", "pub async fn load_async() {}\n");
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("load_async"));
}

#[test]
fn test_check_free_fn_location_flags_multiple_fns() {
    let violations =
        check_free_fn_location("src/narrative/parser.rs", "pub fn a() {}\npub fn b() {}\n");
    assert_eq!(violations.len(), 2);
}

#[test]
fn test_check_free_fn_location_allows_utils_folder() {
    let violations =
        check_free_fn_location("src/narrative/utils/helper.rs", "pub fn helper() {}\n");
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_free_fn_location_allows_mappers_folder() {
    let violations = check_free_fn_location("src/narrative/mappers/map.rs", "pub fn map() {}\n");
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_free_fn_location_skips_tests_files() {
    let violations =
        check_free_fn_location("src/narrative/parser_tests.rs", "pub fn helper() {}\n");
    assert_eq!(violations.len(), 0);
}

#[test]
fn test_check_free_fn_location_skips_main_rs() {
    let violations = check_free_fn_location("src/main.rs", "pub fn helper() {}\n");
    assert_eq!(violations.len(), 0);
}
