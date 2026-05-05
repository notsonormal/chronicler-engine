use crate::error::EngineError;
use crate::model::character::{NpcCard, PlayerCard};
use crate::model::map::Room;
use crate::model::state::LogEntry;
use crate::model::world::WorldCard;
use crate::narrative::prompt::budget;
use crate::narrative::prompt::budget::estimate_tokens;
use crate::narrative::prompt::types::PromptContext;

/// Reserves a safety margin and minimum input budget, caps `max_tokens` to what
/// actually fits, and drops oldest history entries first if the user text is too long.
///
/// Returns `(fitted_system, fitted_user, actual_max_tokens)`.
/// [DOC: docs/system/prompt_system.md]
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

    // System prompt alone must fit with margin and minimum input budget
    if system_tokens + safety_margin + min_input_budget > max_context {
        return Err(EngineError::ContextOverflow {
            requested: system_tokens,
            max: max_context.saturating_sub(safety_margin + min_input_budget),
        });
    }

    let requested = requested_max_tokens.unwrap_or(budget::MAX_RESPONSE_TOKENS) as usize;

    // Available tokens for input (system + user) after reserving margin and response budget
    let available_for_input = max_context.saturating_sub(safety_margin);
    let max_input_tokens = available_for_input.saturating_sub(requested.min(available_for_input));

    // Ensure we leave at least the minimum input budget
    let max_input_tokens = max_input_tokens.max(min_input_budget);

    let fitted_user = if user_tokens <= max_input_tokens.saturating_sub(system_tokens) {
        user.to_string()
    } else {
        let remaining_user_budget = max_input_tokens.saturating_sub(system_tokens);
        trim_history_to_budget(user, remaining_user_budget)
    };
    let fitted_user_tokens = estimate_tokens(&fitted_user);

    let actual_max_tokens = requested
        .min(max_context.saturating_sub(system_tokens + fitted_user_tokens + safety_margin))
        .min(max_context.saturating_sub(system_tokens + min_input_budget + safety_margin))
        .max(1) as u32;

    Ok((system.to_string(), fitted_user, actual_max_tokens))
}

/// Trim the `<ConversationHistory>` section within `user` by dropping oldest entries
/// first until the total token count is within `target_user_tokens`.
pub(crate) fn trim_history_to_budget(user: &str, target_user_tokens: usize) -> String {
    const HISTORY_OPEN: &str = "<ConversationHistory>\n";
    const HISTORY_CLOSE: &str = "\n</ConversationHistory>";

    let Some(start_idx) = user.find(HISTORY_OPEN) else {
        return user.to_string();
    };
    let Some(end_idx) = user.find(HISTORY_CLOSE) else {
        return user.to_string();
    };

    let prefix = &user[..start_idx + HISTORY_OPEN.len()];
    let suffix = &user[end_idx..];
    let history_content = &user[start_idx + HISTORY_OPEN.len()..end_idx];

    // If already within budget, return as-is
    if estimate_tokens(user) <= target_user_tokens {
        return user.to_string();
    }

    let lines: Vec<&str> = history_content.lines().collect();
    if lines.is_empty() {
        return format!("{prefix}(History truncated to fit context window){suffix}");
    }

    // estimate_tokens(text) <= target  <=>  text.len() <= target * 4
    let target_bytes = target_user_tokens.saturating_mul(4);
    let overhead = prefix.len() + suffix.len();
    let total_line_bytes: usize = lines.iter().map(|l| l.len()).sum();

    let mut dropped_bytes = 0;
    let mut first_kept_idx = lines.len();

    for (drop_count, line) in lines.iter().enumerate() {
        let kept_count = lines.len() - drop_count;
        let kept_newlines = kept_count.saturating_sub(1);
        let kept_bytes = total_line_bytes - dropped_bytes;

        if overhead + kept_bytes + kept_newlines <= target_bytes {
            first_kept_idx = drop_count;
            break;
        }

        dropped_bytes += line.len();
    }

    let trimmed_history = if first_kept_idx >= lines.len() {
        "(History truncated to fit context window)"
    } else {
        &lines[first_kept_idx..].join("\n")
    };

    format!("{prefix}{trimmed_history}{suffix}")
}

/// [DOC: docs/system/prompt_system.md]
pub fn make_prompt_context<'a>(
    world: &'a WorldCard,
    room: &'a Room,
    all_npcs: &'a [NpcCard],
    npcs_in_area: &'a [NpcCard],
    player: &'a PlayerCard,
    user_message: &'a str,
    history: &'a [LogEntry],
) -> PromptContext<'a> {
    PromptContext {
        world,
        room,
        all_npcs,
        npcs_in_area,
        player,
        user_message,
        history,
    }
}
