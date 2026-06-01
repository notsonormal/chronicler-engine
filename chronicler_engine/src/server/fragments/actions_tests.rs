use crate::server::fragments::ActionForm;

#[test]
fn test_action_form_deserialization() {
    let form: ActionForm = serde_json::from_str(r#"{"command": "look"}"#).unwrap();
    assert_eq!(form.command, "look");
}

#[test]
fn test_action_form_empty_command() {
    let form: ActionForm = serde_json::from_str(r#"{"command": ""}"#).unwrap();
    assert!(form.command.is_empty());
}

#[test]
fn test_action_form_with_whitespace_command() {
    let form: ActionForm = serde_json::from_str(r#"{"command": "  look  "}"#).unwrap();
    assert_eq!(form.command, "  look  ");
}

#[test]
fn test_action_form_with_special_characters() {
    let form: ActionForm =
        serde_json::from_str(r#"{"command": "go north & talk to guard"}"#).unwrap();
    assert_eq!(form.command, "go north & talk to guard");
}

#[test]
fn test_action_form_deserialize_unicode() {
    let form: ActionForm = serde_json::from_str(r#"{"command": "こんにちは"}"#).unwrap();
    assert_eq!(form.command, "こんにちは");
}

#[test]
fn test_action_form_roundtrip() {
    let original = ActionForm {
        command: "test command".to_string(),
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: ActionForm = serde_json::from_str(&json).unwrap();
    assert_eq!(original.command, parsed.command);
}
