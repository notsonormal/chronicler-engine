use crate::application::prompting::prompt_merge::merge_single_user_message;
use crate::domain::model::prompt_part::{PromptPart, PromptRole};

#[test]
fn test_merge_single_user_message_format() {
    let merged = merge_single_user_message(&PromptPart::system_and_user(
        "system content",
        "user content",
    ));
    assert_eq!(merged.role, PromptRole::User);
    assert!(merged.text.starts_with("[SYSTEM]\n"));
    let system_pos = merged.text.find("system content").unwrap();
    let user_pos = merged.text.find("user content").unwrap();
    assert!(system_pos < user_pos);
    assert_eq!(merged.text, "[SYSTEM]\nsystem content\n\nuser content");
}

#[test]
fn test_merge_single_user_message_preserves_multiline() {
    let merged = merge_single_user_message(&PromptPart::system_and_user(
        "Line 1\nLine 2",
        "User Line 1\nUser Line 2",
    ));
    assert!(merged.text.contains("Line 1\nLine 2"));
    assert!(merged.text.contains("User Line 1\nUser Line 2"));
}
