//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Prompt context fitting — message budget enforcement.

use crate::domain::model::settings::DEFAULT_MAX_TOKENS;
use crate::error::EngineError;
use crate::application::prompting::token_budget as budget;
use crate::application::prompting::token_budget::estimate_tokens;

/// One limit for the history space and for the fit of the whole prompt.
pub fn input_limit(max_context_tokens: u32, requested_max_tokens: Option<u32>) -> usize {
    let available =
        (max_context_tokens as usize).saturating_sub(budget::SAFETY_MARGIN_TOKENS as usize);
    let requested = requested_max_tokens.unwrap_or(DEFAULT_MAX_TOKENS) as usize;

    available
        .saturating_sub(requested.min(available))
        .max(budget::MIN_INPUT_BUDGET_TOKENS as usize)
}

/// Leaves out the Player Input and the Guide, so a new Input does not move the
/// history start.
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

/// Clamps the response length to the room the prompt leaves. The history cut makes
/// the prompt fit, so this only checks it.
pub fn fit_messages_to_context(
    system: &str,
    user: &str,
    max_context_tokens: u32,
    requested_max_tokens: Option<u32>,
) -> Result<u32, EngineError> {
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

    let requested = requested_max_tokens.unwrap_or(DEFAULT_MAX_TOKENS) as usize;

    let actual_max_tokens = requested
        .min(max_context.saturating_sub(system_tokens + user_tokens + safety_margin))
        .min(max_context.saturating_sub(system_tokens + min_input_budget + safety_margin))
        .max(1) as u32;

    Ok(actual_max_tokens)
}
