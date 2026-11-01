//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Prompt context fitting — message budget enforcement.

use crate::error::EngineError;
use crate::application::prompting::token_budget as budget;
use crate::application::prompting::token_budget::estimate_tokens;

/// The most tokens a call may send as input: the window, less the safety margin,
/// less the response length the call asks for. One rule for the history space
/// and for the fit of the whole prompt.
pub fn input_limit(max_context_tokens: u32, requested_max_tokens: Option<u32>) -> usize {
    let available =
        (max_context_tokens as usize).saturating_sub(budget::SAFETY_MARGIN_TOKENS as usize);
    let requested = requested_max_tokens.unwrap_or(budget::MAX_RESPONSE_TOKENS) as usize;

    available
        .saturating_sub(requested.min(available))
        .max(budget::MIN_INPUT_BUDGET_TOKENS as usize)
}

/// The tokens the history entries may use. It depends on the window, the response
/// length and the parts before the history, and on no Input or Guide, so it holds
/// still between two calls of one kind.
pub fn history_space(
    max_context_tokens: u32,
    requested_max_tokens: Option<u32>,
    system_tokens: usize,
    fixed_user_tokens: usize,
) -> usize {
    input_limit(max_context_tokens, requested_max_tokens)
        .saturating_sub(system_tokens)
        .saturating_sub(fixed_user_tokens)
        .saturating_sub(budget::history_reserve_tokens(max_context_tokens))
        .min(budget::MAX_HISTORY_TOKENS as usize)
}

/// Checks that the system half fits the window, and clamps the response length to
/// the room the prompt leaves. It never trims: the history cut makes the prompt fit.
pub fn fit_messages_to_context(
    system: &str,
    user: &str,
    max_context_tokens: u32,
    requested_max_tokens: Option<u32>,
) -> Result<(String, String, u32), EngineError> {
    let system_tokens = estimate_tokens(system);
    let user_tokens = estimate_tokens(user);
    let max_context = max_context_tokens as usize;
    let safety_margin = budget::SAFETY_MARGIN_TOKENS as usize;
    let min_input_budget = budget::MIN_INPUT_BUDGET_TOKENS as usize;

    if system_tokens + safety_margin + min_input_budget > max_context {
        return Err(EngineError::ContextOverflow {
            requested: system_tokens,
            max: max_context.saturating_sub(safety_margin + min_input_budget),
        });
    }

    let requested = requested_max_tokens.unwrap_or(budget::MAX_RESPONSE_TOKENS) as usize;

    let actual_max_tokens = requested
        .min(max_context.saturating_sub(system_tokens + user_tokens + safety_margin))
        .min(max_context.saturating_sub(system_tokens + min_input_budget + safety_margin))
        .max(1) as u32;

    Ok((system.to_string(), user.to_string(), actual_max_tokens))
}
