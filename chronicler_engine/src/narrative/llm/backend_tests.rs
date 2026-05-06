use crate::narrative::llm::backend::merge_single_user_message;

#[test]
fn test_merge_single_user_message_format() {
    let merged = merge_single_user_message("system content", "user content");
    assert!(merged.starts_with("[SYSTEM]\n"));
    assert!(merged.contains("system content"));
    assert!(merged.contains("user content"));
    // System content should come before user content
    let system_pos = merged.find("system content").unwrap();
    let user_pos = merged.find("user content").unwrap();
    assert!(system_pos < user_pos);
}

#[test]
fn test_merge_single_user_message_preserves_multiline() {
    let system = "Line 1\nLine 2";
    let user = "User Line 1\nUser Line 2";
    let merged = merge_single_user_message(system, user);
    assert!(merged.contains("Line 1\nLine 2"));
    assert!(merged.contains("User Line 1\nUser Line 2"));
}

#[test]
fn test_merge_single_user_message_empty_system() {
    let merged = merge_single_user_message("", "user content");
    assert_eq!(merged, "[SYSTEM]\n\n\nuser content");
}

#[test]
fn test_merge_single_user_message_empty_user() {
    let merged = merge_single_user_message("system content", "");
    assert_eq!(merged, "[SYSTEM]\nsystem content\n\n");
}

#[test]
fn test_merge_single_user_message_both_empty() {
    let merged = merge_single_user_message("", "");
    assert_eq!(merged, "[SYSTEM]\n\n\n");
}
