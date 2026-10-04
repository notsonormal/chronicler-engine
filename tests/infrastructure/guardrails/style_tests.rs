//! Tests for `style.rs` guardrail.

use crate::style::*;

#[test]
fn check_import_ordering_flags_crate_before_std() {
    let violations = check_import_ordering(
        "src/example.rs",
        "use crate::thing::Thing;\nuse std::collections::HashMap;\n",
    );
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("Import ordering violation"));
}

#[test]
fn check_import_ordering_allows_std_external_crate_order() {
    let violations = check_import_ordering(
        "src/example.rs",
        "use std::collections::HashMap;\nuse serde::Serialize;\nuse crate::thing::Thing;\n",
    );
    assert!(violations.is_empty());
}

#[test]
fn check_template_raw_strings_flags_single_hash_delimiter() {
    let violations = check_template_raw_strings(
        "src/adapters/driving/http/games/templates/games.rs",
        "let markup = r#\"<div hx-target=\"#id\"></div>\"#;\n",
    );
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("r##"));
}

#[test]
fn check_template_raw_strings_allows_double_hash_delimiter() {
    let violations = check_template_raw_strings(
        "src/adapters/driving/http/games/templates/games.rs",
        "let markup = r##\"<div hx-target=\"#id\"></div>\"##;\n",
    );
    assert!(violations.is_empty());
}

#[test]
fn check_separator_comments_flags_visual_divider() {
    let violations =
        check_separator_comments("src/example.rs", "// === Section ===\nlet value = 1;\n");
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("Separator comment"));
}

#[test]
fn check_separator_comments_allows_plain_comment() {
    let violations = check_separator_comments(
        "src/example.rs",
        "// Explains the next step.\nlet value = 1;\n",
    );
    assert!(violations.is_empty());
}

#[test]
fn check_long_comment_runs_flags_five_consecutive_comments() {
    let content = "// one\n// two\n// three\n// four\n// five\nlet value = 1;\n";
    let violations = check_long_comment_runs("src/example.rs", content);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains("Long comment run"));
}

#[test]
fn check_long_comment_runs_allows_four_consecutive_comments() {
    let content = "// one\n// two\n// three\n// four\nlet value = 1;\n";
    let violations = check_long_comment_runs("src/example.rs", content);
    assert!(violations.is_empty());
}

#[test]
fn check_single_letter_vars_flags_in_long_function() {
    let content = "fn long_function() {\n    let first = 1;\n    let second = 2;\n    \
                   let third = 3;\n    let fourth = 4;\n    let fifth = 5;\n    \
                   let sixth = 6;\n    let seventh = 7;\n    let eighth = 8;\n    \
                   let ninth = 9;\n    let tenth = 10;\n    let q = 11;\n}\n";
    let violations = check_single_letter_vars("src/example.rs", content);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].message.contains('q'));
}

#[test]
fn check_single_letter_vars_allows_short_function() {
    let content = "fn short_function() {\n    let q = 1;\n    let doubled = q + 1;\n}\n";
    let violations = check_single_letter_vars("src/example.rs", content);
    assert!(violations.is_empty());
}
