//! [DOC: docs/system/llm_processing.md]
//! LLM input/output sanitization

use once_cell::sync::Lazy;
use regex::Regex;

/// Strip leaked thinking/reasoning artifacts from LLM output.
#[allow(clippy::expect_used)]
pub fn sanitize_llm_output(text: &str) -> String {
    static RE_LEADING_CHANNEL: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^\s*<channel\|>").expect("valid regex"));
    static RE_THOUGHT_BLOCK: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?s)<thought>.*?</thought>").expect("valid regex"));
    static RE_CHANNEL_THOUGHT: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?s)<\|channel>thought.*?<channel\|>").expect("valid regex"));
    static RE_TURN_MARKERS: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"<\|turn>model|<turn\|>|<\|turn>").expect("valid regex"));

    let result = RE_LEADING_CHANNEL.replace(text, "");
    let result = RE_THOUGHT_BLOCK.replace_all(&result, "");
    let result = RE_CHANNEL_THOUGHT.replace_all(&result, "");
    let result = RE_TURN_MARKERS.replace_all(&result, "");

    result
        .lines()
        .map(|line| line.trim_start())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}
