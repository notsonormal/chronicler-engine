//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Token budget management

pub const MAX_CONTEXT_TOKENS: u32 = 32768;

pub const MAX_HISTORY_TOKENS: u32 = 16000;

pub const MAX_RESPONSE_TOKENS: u32 = 2048;

pub const SAFETY_MARGIN_TOKENS: u32 = 256;

pub const MIN_INPUT_BUDGET_TOKENS: u32 = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TokenBudget {
    pub max_context_tokens: u32,
    pub max_tokens: Option<u32>,
}

impl TokenBudget {
    pub const fn fallback() -> Self {
        Self {
            max_context_tokens: MAX_CONTEXT_TOKENS,
            max_tokens: None,
        }
    }
}
pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

pub fn truncate_to_budget(text: &str, max_tokens: usize) -> String {
    // Inverse of estimate_tokens' four characters per token.
    let max_chars = max_tokens * 4;

    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    let chars: Vec<char> = text.chars().collect();
    let start_idx = chars.len().saturating_sub(max_chars);
    chars[start_idx..].iter().collect()
}
