//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Token budget management

use crate::domain::model::settings::{DEFAULT_MAX_HISTORY_TOKENS, DEFAULT_MAX_TOKENS};
use crate::error::EngineError;

pub const MAX_CONTEXT_TOKENS: u32 = 32768;

pub const SAFETY_MARGIN_TOKENS: u32 = 256;

pub const MIN_INPUT_BUDGET_TOKENS: u32 = 512;

/// An estimate: each provider has its own chat template.
pub const MESSAGE_OVERHEAD_TOKENS: usize = 4;

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
    /// The Connection's size of the history. The room in the window can be smaller.
    pub max_history_tokens: u32,
    /// The response length the call asks for.
    pub max_tokens: u32,
}

impl TokenBudget {
    pub const fn fallback() -> Self {
        Self {
            max_context_tokens: MAX_CONTEXT_TOKENS,
            max_history_tokens: DEFAULT_MAX_HISTORY_TOKENS,
            max_tokens: DEFAULT_MAX_TOKENS,
        }
    }

    /// One limit for the history space and for the fit of the whole prompt.
    pub fn input_limit(&self) -> usize {
        let available =
            (self.max_context_tokens as usize).saturating_sub(SAFETY_MARGIN_TOKENS as usize);
        let requested = self.max_tokens as usize;

        available
            .saturating_sub(requested.min(available))
            .max(MIN_INPUT_BUDGET_TOKENS as usize)
    }

    /// Leaves out the Player Input and the Guide, so a new Input does not move the
    /// history start.
    pub fn history_space(&self, system_tokens: usize, fixed_user_tokens: usize) -> usize {
        self.input_limit()
            .saturating_sub(system_tokens)
            .saturating_sub(fixed_user_tokens)
            .saturating_sub(history_reserve_tokens(self.max_context_tokens))
            .min(self.max_history_tokens as usize)
    }

    /// Clamps the response length to the room the prompt leaves. The history cut
    /// makes the prompt fit, so this only checks it.
    pub fn response_tokens(
        &self,
        system_tokens: usize,
        tokens_after_system: usize,
    ) -> Result<u32, EngineError> {
        let max_context = self.max_context_tokens as usize;
        let safety_margin = SAFETY_MARGIN_TOKENS as usize;
        let min_input_budget = MIN_INPUT_BUDGET_TOKENS as usize;

        if system_tokens + safety_margin + min_input_budget > max_context {
            return Err(EngineError::ContextOverflow {
                requested: system_tokens,
                max: max_context.saturating_sub(safety_margin + min_input_budget),
            });
        }

        let actual_max_tokens = (self.max_tokens as usize)
            .min(max_context.saturating_sub(system_tokens + tokens_after_system + safety_margin))
            .min(max_context.saturating_sub(system_tokens + min_input_budget + safety_margin))
            .max(1) as u32;

        Ok(actual_max_tokens)
    }
}

pub const CHARS_PER_TOKEN: usize = 4;

pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(CHARS_PER_TOKEN)
}
