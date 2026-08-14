use chrono::Utc;

use crate::domain::model::llm_message::LlmMessage;
use crate::adapters::driven::storage::models::llm_message::DbLlmMessage;

#[test]
fn test_llm_message_roundtrip() {
    let original = LlmMessage {
        id: 5,
        agent_name: "narrator".to_string(),
        backend_name: "ollama".to_string(),
        model_name: "llama3".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        raw_request_json: "{}".to_string(),
        raw_response_json: "{}".to_string(),
        parsed_response: "hi".to_string(),
        error_message: None,
        created_at: Utc::now(),
    };
    let db = DbLlmMessage::from(&original);
    let back = LlmMessage::try_from(&db).unwrap();

    assert_eq!(original.id, back.id);
    assert_eq!(original.agent_name, back.agent_name);
    assert_eq!(original.backend_name, back.backend_name);
    assert_eq!(original.model_name, back.model_name);
    assert_eq!(original.system_prompt, back.system_prompt);
    assert_eq!(original.user_prompt, back.user_prompt);
    assert_eq!(original.raw_request_json, back.raw_request_json);
    assert_eq!(original.raw_response_json, back.raw_response_json);
    assert_eq!(original.parsed_response, back.parsed_response);
    assert_eq!(original.error_message, back.error_message);
    assert_eq!(original.created_at, back.created_at);
}

#[test]
fn test_llm_message_with_error_roundtrip() {
    let original = LlmMessage {
        id: 0,
        agent_name: "quantifier".to_string(),
        backend_name: "openrouter".to_string(),
        model_name: "gpt-4".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        raw_request_json: "{}".to_string(),
        raw_response_json: "{}".to_string(),
        parsed_response: "".to_string(),
        error_message: Some("timeout".to_string()),
        created_at: Utc::now(),
    };
    let db = DbLlmMessage::from(&original);
    let back = LlmMessage::try_from(&db).unwrap();

    assert_eq!(back.error_message, Some("timeout".to_string()));
}
