//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Harper text-check adapter boundary helpers.

use crate::application::ports::text_checker::IssueKind;

pub(crate) fn lint_kind_to_issue_kind(kind: harper_core::linting::LintKind) -> IssueKind {
    use harper_core::linting::LintKind as HK;
    match kind {
        HK::Spelling => IssueKind::Spelling,
        HK::Capitalization => IssueKind::Capitalization,
        HK::Formatting => IssueKind::Formatting,
        HK::Repetition => IssueKind::Style,
        HK::Readability => IssueKind::Style,
        HK::Enhancement => IssueKind::Grammar,
        HK::WordChoice => IssueKind::Grammar,
        HK::Style => IssueKind::Style,
        HK::Miscellaneous => IssueKind::Other,
    }
}

pub(crate) fn char_span_to_byte_span(
    text: &str,
    char_span: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    let mut byte_start = None;
    let mut byte_end = None;

    for (char_idx, (byte_idx, _)) in text.char_indices().enumerate() {
        if char_idx == char_span.start {
            byte_start = Some(byte_idx);
        }
        if char_idx == char_span.end {
            byte_end = Some(byte_idx);
            break;
        }
    }

    let start = byte_start.unwrap_or(0);
    let end = byte_end.unwrap_or(text.len());
    start..end
}

pub(crate) fn apply_suggestions(text: &str, lints: &[harper_core::linting::Lint]) -> String {
    let mut chars: Vec<char> = text.chars().collect();
    let mut sorted_lints: Vec<_> = lints.iter().collect();
    sorted_lints.sort_by_key(|l| std::cmp::Reverse(l.span.start));

    for lint in sorted_lints {
        let Some(suggestion) = lint.suggestions.first() else {
            continue;
        };
        // Lint spans are char-index ranges into the ORIGINAL text. A stale
        // lint (computed against a different text version) can land outside
        // `chars` and panic inside `suggestion.apply`. Skip the offending one
        // and keep applying the rest.
        if lint.span.start > lint.span.end || lint.span.end > chars.len() {
            tracing::warn!(
                "skipping harper suggestion: span {:?} out of bounds for text of {} chars",
                lint.span,
                chars.len(),
            );
            continue;
        }
        suggestion.apply(lint.span, &mut chars);
    }

    chars.into_iter().collect()
}
