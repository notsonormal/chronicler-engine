use crate::narrative::llm::backend::LlmBackend;
use crate::narrative::llm::mock::MockBackend;

#[test]
fn test_mock_backend_name() {
    let backend = MockBackend::default();
    assert_eq!(backend.name(), "Mock");
}

#[test]
fn test_mock_complete() {
    let backend = MockBackend::default();
    let result = backend.complete("test", "system prompt", "user action", None);
    assert!(result.is_ok());
    let response = result.unwrap().text;
    assert!(response.contains("action") || response.contains("Continuation"));
}

#[test]
fn test_mock_complete_multiline() {
    let backend = MockBackend::default();
    let result = backend.complete(
        "test",
        "system prompt\nwith multiple lines",
        "user prompt\nalso multiline",
        None,
    );
    assert!(result.is_ok());
    assert!(result.unwrap().text.contains("user prompt"));
}

#[test]
fn test_mock_complete_empty() {
    let backend = MockBackend::default();
    let result = backend.complete("test", "", "", None);
    assert!(result.is_ok());
    assert!(result.unwrap().text.contains("..."));
}

#[test]
fn test_mock_complete_per_call_responses() {
    let backend = MockBackend::default().with_prompt_responses(vec![
        r#"{"npcs_in_room": ["carla"], "movement": {"type": null}}"#.to_string(),
        r#"{"npcs_in_room": [], "movement": {"type": "entering", "destination": "kitchen"}}"#
            .to_string(),
    ]);
    let result1 = backend.complete("quantifier", "sys", "user", None);
    assert!(result1.is_ok());
    assert!(result1.unwrap().text.contains("carla"));

    let result2 = backend.complete("quantifier", "sys", "user", None);
    assert!(result2.is_ok());
    assert!(result2.unwrap().text.contains("kitchen"));

    let result3 = backend.complete("quantifier", "sys", "user", None);
    assert!(result3.is_ok());
    assert!(result3.unwrap().text.contains("carla"));
}

#[test]
fn test_mock_complete_very_long_input() {
    let backend = MockBackend::default();
    let long_system = "You are a game master. ".repeat(50);
    let long_user = "The player performs an action. ".repeat(50);
    let result = backend.complete("test", &long_system, &long_user, None);
    assert!(result.is_ok());
    assert!(result.unwrap().text.contains("The player performs"));
}

#[test]
fn test_mock_with_failing_trigger_narration() {
    let backend = MockBackend::default().with_trigger_narration_fail();
    let narrate_result = backend.complete(
        crate::narrative::llm::backend::AGENT_NARRATOR,
        "sys",
        "user",
        None,
    );
    assert!(
        narrate_result.is_ok(),
        "narrator complete should succeed even with trigger_narration_should_fail set"
    );
    let trigger_result = backend.complete("trigger", "sys", "user", None);
    assert!(
        trigger_result.is_err(),
        "trigger complete should fail when trigger_narration_should_fail is set"
    );
    assert!(
        trigger_result
            .unwrap_err()
            .to_string()
            .contains("mock_trigger"),
        "Error message should identify this as a trigger narration failure"
    );
}

#[test]
fn test_mock_backend_logs_to_storage() {
    use crate::adapters::driven::storage::Storage;
    use std::sync::Arc;
    let storage = Arc::new(Storage::new_in_memory());
    let backend = MockBackend::new(Some(Arc::clone(&storage)));

    let result = backend.complete("narrator", "sys", "user action", None);
    assert!(result.is_ok());

    let messages = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].agent_name, "narrator");
    assert_eq!(messages[0].backend_name, "Mock");
    assert_eq!(messages[0].model_name, "mock");
}

#[test]
fn test_mock_backend_logs_multiple_calls() {
    use crate::adapters::driven::storage::Storage;
    use std::sync::Arc;
    let storage = Arc::new(Storage::new_in_memory());
    let backend = MockBackend::new(Some(Arc::clone(&storage)));

    let _ = backend.complete("narrator", "sys", "first", None);
    let _ = backend.complete("trigger", "sys", "second", None);
    let _ = backend.complete("narrator", "sys", "third", None);

    let messages = storage.list_latest_llm_messages(50).unwrap();
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].agent_name, "narrator");
    assert_eq!(messages[1].agent_name, "trigger");
    assert_eq!(messages[2].agent_name, "narrator");
}

#[test]
fn test_mock_backend_builders_compose() {
    let backend = MockBackend::default()
        .with_empty_response()
        .with_prompt_responses(vec!["response1".to_string()]);

    let narrate = backend.complete(
        crate::narrative::llm::backend::AGENT_NARRATOR,
        "sys",
        "user",
        None,
    );
    assert!(narrate.is_ok());
    assert!(
        narrate.unwrap().text.is_empty(),
        "with_empty_response should win on narrator path"
    );

    let trigger = backend.complete("trigger", "sys", "user", None);
    assert!(trigger.is_ok());
    assert_eq!(
        trigger.unwrap().text,
        "response1",
        "with_prompt_responses should survive chaining and be returned on trigger path"
    );
}
