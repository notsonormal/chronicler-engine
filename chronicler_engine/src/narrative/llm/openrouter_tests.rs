use std::sync::Arc;

use crate::model::llm_backend::LlmBackendType;
use crate::model::settings::Connection;
use crate::narrative::llm::backend::LlmBackend;
use crate::narrative::llm::openrouter::OpenRouterBackend;
use crate::storage::llm_message_storage::{InMemoryLlmMessageStorage, LlmMessageStorage};

#[test]
fn test_openrouter_backend_name() {
    let backend = OpenRouterBackend::default();
    assert_eq!(backend.name(), "OpenRouter");
}

#[test]
fn test_openrouter_from_connection() {
    let conn = Connection {
        id: "or-1".into(),
        name: "OpenRouter".into(),
        provider: LlmBackendType::OpenRouter,
        model: "gpt-4o".into(),
        api_key: Some("sk-test".into()),
        base_url: None,
        single_user_message: false,
        max_tokens: Some(2048),
        max_context_tokens: Some(32768),
    };

    let backend = OpenRouterBackend::from_connection(&conn, None, None);
    assert_eq!(backend.model(), "gpt-4o");
    assert_eq!(backend.name(), "OpenRouter");
}

#[test]
fn test_openrouter_model() {
    let conn = Connection {
        id: "test".into(),
        name: "Test".into(),
        provider: LlmBackendType::OpenRouter,
        model: "claude-3-opus".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };

    let backend = OpenRouterBackend::from_connection(&conn, None, None);
    assert_eq!(backend.model(), "claude-3-opus");
}

#[test]
fn test_openrouter_save_message_with_storage() {
    let storage = Arc::new(InMemoryLlmMessageStorage::new());
    let conn = Connection {
        id: "test".into(),
        name: "Test".into(),
        provider: LlmBackendType::OpenRouter,
        model: "gpt-4".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let backend = OpenRouterBackend::from_connection(
        &conn,
        Some(Arc::clone(&storage) as Arc<dyn LlmMessageStorage>),
        None,
    );

    let msg = crate::model::llm_message::LlmMessageBuilder::new()
        .agent_name("test")
        .backend_name("OpenRouter")
        .model_name("gpt-4")
        .system_prompt("sys")
        .user_prompt("user")
        .raw_request_json("req")
        .raw_response_json("res")
        .parsed_response("hello")
        .error_message(None::<String>)
        .build();

    backend.save_message(&msg);

    let messages = storage.list_latest(10).unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].agent_name, "test");
}

#[test]
fn test_openrouter_save_message_without_storage() {
    let backend = OpenRouterBackend::default();

    let msg = crate::model::llm_message::LlmMessageBuilder::new()
        .agent_name("test")
        .backend_name("OpenRouter")
        .model_name("gpt-4")
        .system_prompt("sys")
        .user_prompt("user")
        .raw_request_json("req")
        .raw_response_json("res")
        .parsed_response("hello")
        .error_message(None::<String>)
        .build();

    // Should not panic when storage is None
    backend.save_message(&msg);
}
