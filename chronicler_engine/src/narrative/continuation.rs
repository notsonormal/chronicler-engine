//! [DOC: docs/system/triggers.md]

use crate::error::{EngineError, Result};
use crate::narrative::prompt::{PromptContext, estimate_tokens, truncate_to_budget};

/// Maximum tokens for the first narration in continuation prompts.
/// We reserve some budget for the room context, trigger text, and prompt overhead.
const MAX_FIRST_NARRATION_TOKENS: usize = 2048;

/// [DOC: docs/system/triggers.md]
pub fn build_continuation_prompt(
    context: &PromptContext,
    first_narration: &str,
    trigger_text: &str,
) -> Result<(String, String)> {
    // Truncate first narration to fit within token budget
    let truncated_narration = truncate_first_narration(first_narration, MAX_FIRST_NARRATION_TOKENS);

    let system_prompt = build_system_instruction();
    let room_context = build_room_context(context);

    let user_prompt = format!(
        "Previous narration:\n{truncated_narration}\n\n\
         Current location: {room_context}\n\n\
         Trigger event: {trigger_text}\n\n\
         Continue the scene from where the previous narration left off.",
    );

    // Final token budget check
    let total_tokens = estimate_tokens(&system_prompt) + estimate_tokens(&user_prompt);
    if total_tokens > crate::narrative::prompt::budget::MAX_CONTEXT_TOKENS as usize {
        return Err(EngineError::ContextOverflow {
            requested: total_tokens,
            max: crate::narrative::prompt::budget::MAX_CONTEXT_TOKENS as usize,
        });
    }

    Ok((system_prompt, user_prompt))
}

/// Build the system prompt instruction for continuation narration.
fn build_system_instruction() -> String {
    String::from(
        r#"You must continue the scene from the previous narration.
 
Incorporate the following trigger event naturally into the narrative. Do NOT repeat or contradict what was already described.
 
Keep your response concise — focus on the trigger event and immediate reactions. Write in the style of literary fiction prose."#,
    )
}

/// [DOC: docs/system/llm_processing.md]
fn truncate_first_narration(narration: &str, max_tokens: usize) -> String {
    let truncated = truncate_to_budget(narration, max_tokens);

    if truncated.len() < narration.len() {
        // Add ellipsis to indicate truncation
        if let Some(last_newline) = truncated.rfind('\n') {
            // Don't cut in the middle of a paragraph
            format!(
                "{}...\n\n[Previous scene continues...]",
                &truncated[..last_newline]
            )
        } else if let Some(last_space) = truncated.rfind(' ') {
            format!("{}...", &truncated[..last_space])
        } else {
            format!("{truncated}...")
        }
    } else {
        truncated
    }
}

/// Build a compact room context string for the continuation prompt.
fn build_room_context(context: &PromptContext) -> String {
    let mut parts = Vec::new();

    // Room name and description
    parts.push(format!("Room: {}", context.room.name));
    if !context.room.description.is_empty() {
        // Truncate description to first 200 chars for context
        let desc = if context.room.description.len() > 200 {
            format!("{}...", &context.room.description[..200])
        } else {
            context.room.description.clone()
        };
        parts.push(format!("Description: {desc}"));
    }

    // NPCs present in the area
    if !context.npcs_in_area.is_empty() {
        let npc_names: Vec<&str> = context
            .npcs_in_area
            .iter()
            .map(|npc| npc.sheet.name.as_str())
            .collect();
        parts.push(format!("Present: {}", npc_names.join(", ")));
    }

    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::character::{CharacterSheet, NpcCard, PlayerCard};
    use crate::model::map::Room;
    use crate::model::state::LogEntry;
    use crate::model::world::WorldCard;
    use std::collections::HashMap;

    fn create_test_context() -> PromptContext<'static> {
        let world = Box::leak(Box::new(WorldCard {
            name: "Test World".to_string(),
            description: "A test world.".to_string(),
            global_rules: vec![],
            ..Default::default()
        }));

        let room = Box::leak(Box::new(Room {
            id: "room_1".to_string(),
            name: "Grand Hall".to_string(),
            description: "A spacious hall with tall ceilings and dusty chandeliers.".to_string(),
            exits: HashMap::new(),
            items: vec![],
            npcs: vec![],
            image_path: None,
            navigation_description: None,
        }));

        let npc = NpcCard {
            id: "gabriella".to_string(),
            sheet: CharacterSheet {
                name: "Gabriella".to_string(),
                description: "A mysterious woman.".to_string(),
                personality: "Guarded".to_string(),
                scenario: "Waiting in the shadows.".to_string(),
                example_dialogue: String::new(),
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
            triggers: vec![],
        };
        let npcs_in_area = Box::leak(Box::new(vec![npc.clone()]));
        let all_npcs = Box::leak(Box::new(vec![npc]));

        let player = Box::leak(Box::new(PlayerCard {
            sheet: CharacterSheet {
                name: "Hero".to_string(),
                description: "The protagonist.".to_string(),
                personality: "Brave".to_string(),
                scenario: "Exploring.".to_string(),
                example_dialogue: String::new(),
                profile_image: None,
                headshot_image: None,
            },
            inventory: vec![],
        }));

        let user_message = Box::leak(Box::new(String::new()));
        let history: &[LogEntry] = &[];

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

    #[test]
    fn test_build_continuation_prompt_returns_non_empty_prompts() {
        let context = create_test_context();
        let first_narration = "You enter the grand hall.";
        let trigger_text = "Gabriella steps forward from the shadows.";

        let result = build_continuation_prompt(&context, first_narration, trigger_text);

        assert!(result.is_ok());
        let (system_prompt, user_prompt) = result.unwrap();
        assert!(!system_prompt.is_empty());
        assert!(!user_prompt.is_empty());
    }

    #[test]
    fn test_first_narration_included_in_prompt() {
        let context = create_test_context();
        let first_narration = "You enter the grand hall and look around.";
        let trigger_text = "Gabriella steps forward.";

        let result = build_continuation_prompt(&context, first_narration, trigger_text);

        assert!(result.is_ok());
        let (_, user_prompt) = result.unwrap();
        assert!(user_prompt.contains(first_narration));
    }

    #[test]
    fn test_trigger_text_included_in_prompt() {
        let context = create_test_context();
        let first_narration = "You enter the hall.";
        let trigger_text = "Gabriella steps forward from the shadows.";

        let result = build_continuation_prompt(&context, first_narration, trigger_text);

        assert!(result.is_ok());
        let (_, user_prompt) = result.unwrap();
        assert!(user_prompt.contains(trigger_text));
    }

    #[test]
    fn test_long_narration_is_truncated() {
        let context = create_test_context();
        // Create a narration that exceeds the 2048 token budget (~8000 chars)
        let long_narration =
            "The grand hall stretches before you, its vaulted ceiling lost in shadow. ".repeat(200);
        let trigger_text = "Gabriella speaks your name.";

        let result = build_continuation_prompt(&context, &long_narration, trigger_text);

        assert!(result.is_ok());
        let (_, user_prompt) = result.unwrap();
        // The prompt should be truncated but still contain the trigger text
        assert!(user_prompt.contains(trigger_text));
        // The narration should have ellipsis or indication of truncation
        assert!(user_prompt.contains("...") || user_prompt.contains("[Previous scene continues]"));
    }

    #[test]
    fn test_room_context_included() {
        let context = create_test_context();
        let first_narration = "You enter the hall.";
        let trigger_text = "Something happens.";

        let result = build_continuation_prompt(&context, first_narration, trigger_text);

        assert!(result.is_ok());
        let (_, user_prompt) = result.unwrap();
        assert!(user_prompt.contains("Grand Hall"));
    }

    #[test]
    fn test_system_prompt_contains_continue_instruction() {
        let context = create_test_context();
        let first_narration = "You enter.";
        let trigger_text = "Something happens.";

        let result = build_continuation_prompt(&context, first_narration, trigger_text);

        assert!(result.is_ok());
        let (system_prompt, _) = result.unwrap();
        assert!(system_prompt.to_lowercase().contains("continue"));
        assert!(system_prompt.to_lowercase().contains("previous"));
    }

    #[test]
    fn test_system_prompt_forbids_repetition() {
        let context = create_test_context();
        let first_narration = "You enter.";
        let trigger_text = "Something happens.";

        let result = build_continuation_prompt(&context, first_narration, trigger_text);

        assert!(result.is_ok());
        let (system_prompt, _) = result.unwrap();
        assert!(
            system_prompt.to_lowercase().contains("not repeat")
                || system_prompt.to_lowercase().contains("don't repeat")
                || system_prompt.to_lowercase().contains("repeat")
        );
    }

    #[test]
    fn test_npcs_in_area_listed_in_context() {
        let context = create_test_context();
        let first_narration = "You enter.";
        let trigger_text = "Something happens.";

        let result = build_continuation_prompt(&context, first_narration, trigger_text);

        assert!(result.is_ok());
        let (_, user_prompt) = result.unwrap();
        assert!(user_prompt.contains("Gabriella"));
    }
}
