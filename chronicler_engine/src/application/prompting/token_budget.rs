//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]
//! Token budget management

/// Maximum tokens allocated for the entire context window (fallback default).
pub const MAX_CONTEXT_TOKENS: u32 = 32768;

/// Maximum tokens for history (conversation log).
pub const MAX_HISTORY_TOKENS: u32 = 16000;

/// Maximum tokens for system prompt.
pub const MAX_SYSTEM_TOKENS: u32 = 1024;

/// Maximum tokens for LLM response generation (fallback default).
pub const MAX_RESPONSE_TOKENS: u32 = 2048;

/// Safety margin reserved for token estimation error.
pub const SAFETY_MARGIN_TOKENS: u32 = 256;

/// Minimum tokens reserved for the input side (system + user).
pub const MIN_INPUT_BUDGET_TOKENS: u32 = 512;
pub fn estimate_tokens(text: &str) -> usize {
    // Use div_ceil for cleaner integer division with ceiling
    text.chars().count().div_ceil(4)
}

pub fn truncate_to_budget(text: &str, max_tokens: usize) -> String {
    let max_chars = max_tokens * 4; // Reverse the token estimate

    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    // Keep only the last max_chars characters
    let chars: Vec<char> = text.chars().collect();
    let start_idx = chars.len().saturating_sub(max_chars);
    chars[start_idx..].iter().collect()
}
