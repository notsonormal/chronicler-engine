//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Mock LLM provider utilities.

const PLAYER_INPUT_OPEN: &str = "<PlayerInput>\n";
const PLAYER_INPUT_CLOSE: &str = "\n</PlayerInput>";

pub(crate) fn extract_player_input(user_prompt: &str) -> Option<String> {
    let start = user_prompt.find(PLAYER_INPUT_OPEN)?;
    let content_start = start + PLAYER_INPUT_OPEN.len();
    let end = user_prompt[content_start..].find(PLAYER_INPUT_CLOSE)?;
    Some(user_prompt[content_start..content_start + end].to_string())
}
