//! [DOC: docs/diataxis/reference/narrative/narration_system.md]
//! Mock LLM provider utilities.

use crate::domain::model::prompt_part::{PromptPart, PromptRole};

const PLAYER_INPUT: (&str, &str) = ("<PlayerInput>\n", "\n</PlayerInput>");
const LAST_MESSAGE: (&str, &str) = ("<LastMessage>\n", "\n</LastMessage>");

/// Falls back to the last history message because the engine leaves out the Player
/// Input block when that message already holds the same text.
pub(crate) fn extract_player_input(prompt: &[PromptPart]) -> Option<String> {
    let last_user = prompt
        .iter()
        .rev()
        .find(|part| part.role == PromptRole::User)?;
    between(&last_user.text, PLAYER_INPUT).or_else(|| between(&last_user.text, LAST_MESSAGE))
}

pub(crate) fn first_user_line(prompt: &[PromptPart]) -> String {
    prompt
        .iter()
        .find(|part| part.role != PromptRole::System)
        .and_then(|part| part.text.lines().next())
        .unwrap_or("...")
        .to_string()
}

fn between(text: &str, (open, close): (&str, &str)) -> Option<String> {
    let start = text.find(open)? + open.len();
    let end = text[start..].find(close)?;
    Some(text[start..start + end].to_string())
}

pub(crate) fn request_json(prompt: &[PromptPart]) -> String {
    serde_json::json!({ "messages": prompt.iter().map(PromptPart::to_json).collect::<Vec<_>>() })
        .to_string()
}
