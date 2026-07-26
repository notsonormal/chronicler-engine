//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]
//! Merge system + user prompts for models that ignore the system role.

/// Merge system + user for models that ignore system role.
pub fn merge_single_user_message(system_prompt: &str, user_text: &str) -> String {
    format!("[SYSTEM]\n{system_prompt}\n\n{user_text}")
}
