//! Unit tests for `get_llm_recorder_for` factory function.

use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::domain::model::settings::Connection;
use crate::domain::model::llm_backend::LlmBackendType;
use crate::bootstrap::llm_factory::get_llm_recorder_for;

#[test]
fn mock_backend_path_returns_recorder_with_mock_provider() {
    let connection = Connection {
        id: "test-mock".to_string(),
        name: "Test Mock".to_string(),
        provider: LlmBackendType::Mock,
        model: "mock-model".to_string(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let storage = Arc::new(Storage::new_in_memory());

    let recorder = get_llm_recorder_for(&connection, storage.clone())
        .expect("get_llm_recorder_for should succeed for Mock backend");

    // Verify the recorder was created and has the right provider
    assert_eq!(recorder.provider().name(), "Mock");
    assert_eq!(recorder.provider().model(), "mock");
}

#[test]
fn deepseek_path_returns_recorder() {
    let connection = Connection {
        id: "test-deepseek".to_string(),
        name: "Test DeepSeek".to_string(),
        provider: LlmBackendType::DeepSeek,
        model: "deepseek-chat".to_string(),
        api_key: Some("fake-key".to_string()),
        base_url: Some("https://api.deepseek.com".to_string()),
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let storage = Arc::new(Storage::new_in_memory());

    let recorder = get_llm_recorder_for(&connection, storage.clone());

    // from_connection() doesn't validate at construction time - defers to call time.
    // So this succeeds here.
    assert!(recorder.is_ok());
}

#[test]
fn openrouter_path_returns_recorder() {
    let connection = Connection {
        id: "test-openrouter".to_string(),
        name: "Test OpenRouter".to_string(),
        provider: LlmBackendType::OpenRouter,
        model: "openrouter-model".to_string(),
        api_key: Some("fake-key".to_string()),
        base_url: Some("https://openrouter.ai/api/v1".to_string()),
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let storage = Arc::new(Storage::new_in_memory());

    let recorder = get_llm_recorder_for(&connection, storage.clone());
    assert!(recorder.is_ok());
}

#[test]
fn ollama_path_returns_recorder() {
    let connection = Connection {
        id: "test-ollama".to_string(),
        name: "Test Ollama".to_string(),
        provider: LlmBackendType::Ollama,
        model: "llama3".to_string(),
        base_url: Some("http://localhost:11434".to_string()),
        api_key: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let storage = Arc::new(Storage::new_in_memory());

    let recorder = get_llm_recorder_for(&connection, storage.clone());
    assert!(recorder.is_ok());
}

#[test]
fn deepseek_missing_base_url_still_returns_recorder_defers_error() {
    // from_connection() doesn't actually fail on missing fields at construction.
    // It defers errors to call time (e.g., when complete() is called).
    // This test documents that behavior.
    let connection = Connection {
        id: "test-deepseek-missing".to_string(),
        name: "Test DeepSeek Missing".to_string(),
        provider: LlmBackendType::DeepSeek,
        model: "deepseek-chat".to_string(),
        api_key: None,  // Missing
        base_url: None, // Missing
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let storage = Arc::new(Storage::new_in_memory());

    let recorder = get_llm_recorder_for(&connection, storage.clone());

    // Factory succeeds - error deferred to call time
    let _ = recorder.expect("factory should succeed, error deferred");
}
