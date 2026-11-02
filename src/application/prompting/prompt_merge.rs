//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Prompt merge helper for models that ignore the system role.

use crate::domain::model::prompt_part::{PromptPart, PromptRole};

pub fn merge_single_user_message(parts: &[PromptPart]) -> PromptPart {
    let text = parts
        .iter()
        .filter(|part| !part.text.is_empty())
        .map(|part| match part.role {
            PromptRole::System => format!("[SYSTEM]\n{}", part.text),
            PromptRole::Assistant => format!("[ASSISTANT]\n{}", part.text),
            PromptRole::User => part.text.clone(),
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    PromptPart::user(text)
}
