//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Options output parsing — accepts both seeded prompt shapes.

use once_cell::sync::Lazy;
use regex::Regex;

/// Regex patterns here are compile-time constants; an invalid pattern is a
/// programmer error, not a runtime condition.
#[allow(clippy::expect_used)]
fn compile(pattern: &str) -> Regex {
    Regex::new(pattern).expect("valid regex")
}

/// `<suggestion>...</suggestion>` tags (the default seed's shape).
static RE_SUGGESTION_TAG: Lazy<Regex> =
    Lazy::new(|| compile(r"(?is)<suggestion>(.*?)</suggestion>"));
/// Numbered list lines (`1. text`) (the Roadway seed's shape).
static RE_NUMBERED_LINE: Lazy<Regex> = Lazy::new(|| compile(r"(?m)^\s*\d+[.)]\s+(.+?)\s*$"));
/// `Suggestion N: text` (the CYOA extension's documented fallback).
static RE_SUGGESTION_PREFIX: Lazy<Regex> =
    Lazy::new(|| compile(r"(?im)suggestion\s+\d+\s*:\s*(.+?)\s*$"));

/// Parse an LLM response into at most `count` option strings.
///
/// Strategies run in priority order — tags, numbered list, `Suggestion N:`
/// prefix — and the first strategy that yields any item wins, so one
/// response never mixes shapes.
pub fn parse_options(response: &str, count: u32) -> Vec<String> {
    let take = count.max(1) as usize;
    for regex in [&RE_SUGGESTION_TAG, &RE_NUMBERED_LINE, &RE_SUGGESTION_PREFIX] {
        let items: Vec<String> = regex
            .captures_iter(response)
            .filter_map(|captures| captures.get(1).map(|m| m.as_str().trim().to_string()))
            .filter(|item| !item.is_empty())
            .take(take)
            .collect();
        if !items.is_empty() {
            return items;
        }
    }
    Vec::new()
}
