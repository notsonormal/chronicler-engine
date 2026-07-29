//! Tests for `harper.rs` text-check adapter boundary helpers.

use harper_core::linting::{Lint, Suggestion};
use harper_core::Span;

use crate::adapters::driven::utils::harper::{
    apply_suggestions, char_span_to_byte_span, lint_kind_to_issue_kind,
};
use crate::application::ports::text_checker::IssueKind;

#[test]
fn lint_kind_to_issue_kind_maps_each_variant() {
    use harper_core::linting::LintKind as HK;
    assert_eq!(lint_kind_to_issue_kind(HK::Spelling), IssueKind::Spelling);
    assert_eq!(
        lint_kind_to_issue_kind(HK::Capitalization),
        IssueKind::Capitalization
    );
    assert_eq!(
        lint_kind_to_issue_kind(HK::Formatting),
        IssueKind::Formatting
    );
    assert_eq!(lint_kind_to_issue_kind(HK::Repetition), IssueKind::Style);
    assert_eq!(lint_kind_to_issue_kind(HK::Readability), IssueKind::Style);
    assert_eq!(lint_kind_to_issue_kind(HK::Enhancement), IssueKind::Grammar);
    assert_eq!(lint_kind_to_issue_kind(HK::WordChoice), IssueKind::Grammar);
    assert_eq!(lint_kind_to_issue_kind(HK::Style), IssueKind::Style);
    assert_eq!(lint_kind_to_issue_kind(HK::Miscellaneous), IssueKind::Other);
}

#[test]
fn char_span_to_byte_span_ascii_identity() {
    assert_eq!(char_span_to_byte_span("hello", 1..4), 1..4);
}

#[test]
fn char_span_to_byte_span_multibyte_boundary() {
    // 'é' is 2 bytes; 'a' is 1.
    // char indices: 0='a', 1='é', 2='b', 3='c'
    // byte indices: 0='a', 1-2='é', 3='b', 4='c'
    assert_eq!(char_span_to_byte_span("aébc", 1..3), 1..4);
    assert_eq!(char_span_to_byte_span("aébc", 0..1), 0..1);
}

#[test]
fn char_span_to_byte_span_end_beyond_text_defaults_to_text_len() {
    // char end not found → defaults to text.len()
    assert_eq!(char_span_to_byte_span("ab", 0..5), 0..2);
}

fn lint_with(span: Span, suggestion: Suggestion) -> Lint {
    Lint {
        span,
        suggestions: vec![suggestion],
        ..Lint::default()
    }
}

#[test]
fn apply_suggestions_replaces_text_in_span() {
    let lints = [lint_with(
        Span::new(0, 5),
        Suggestion::ReplaceWith("world".chars().collect()),
    )];
    assert_eq!(apply_suggestions("hello", &lints), "world");
}

#[test]
fn apply_suggestions_applies_non_overlapping_lints() {
    // Reverse-sort by span.start means later-span-first application; for
    // non-overlapping spans either order yields the same result.
    let lints = [
        lint_with(Span::new(0, 1), Suggestion::ReplaceWith(vec!['H'])),
        lint_with(Span::new(6, 7), Suggestion::ReplaceWith(vec!['W'])),
    ];
    assert_eq!(apply_suggestions("hello world", &lints), "Hello World");
}

#[test]
fn apply_suggestions_skips_out_of_bounds_spans() {
    // Stale-lint guard: span.end > chars.len() should be skipped, not panic.
    let lints = [
        lint_with(
            Span::new(0, 5),
            Suggestion::ReplaceWith("world".chars().collect()),
        ),
        lint_with(
            Span::new(10, 100),
            Suggestion::ReplaceWith("X".chars().collect()),
        ),
    ];
    assert_eq!(apply_suggestions("hello", &lints), "world");
}
