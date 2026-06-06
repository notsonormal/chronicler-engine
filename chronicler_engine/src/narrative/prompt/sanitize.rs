//! [DOC: docs/system/prompt_system.md]

use once_cell::sync::Lazy;
use regex::Regex;

#[allow(clippy::expect_used)]
pub fn sanitize_for_prompt(input: &str) -> String {
    static INJECTION_PATTERN: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\{\{.+?\}\}").expect("valid regex pattern"));

    INJECTION_PATTERN
        .replace_all(input, "[FILTERED]")
        .to_string()
}
