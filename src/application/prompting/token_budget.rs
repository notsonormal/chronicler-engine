//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Token budget management

pub const MAX_CONTEXT_TOKENS: u32 = 32768;

pub const MAX_HISTORY_TOKENS: u32 = 16000;

pub const SAFETY_MARGIN_TOKENS: u32 = 256;

pub const MIN_INPUT_BUDGET_TOKENS: u32 = 512;

/// The room the history leaves for the Player Input and the Guide, at most.
pub const HISTORY_RESERVE_TOKENS: u32 = 1024;

/// The reserve is at most this part of the window, so a small window keeps its history.
const HISTORY_RESERVE_WINDOW_PART: u32 = 8;

pub fn history_reserve_tokens(max_context_tokens: u32) -> usize {
    HISTORY_RESERVE_TOKENS.min(max_context_tokens / HISTORY_RESERVE_WINDOW_PART) as usize
}

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

pub const CHARS_PER_TOKEN: usize = 4;

pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(CHARS_PER_TOKEN)
}
