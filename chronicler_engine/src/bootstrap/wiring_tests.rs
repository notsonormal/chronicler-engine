//! Unit tests for LLM provider composition through the public wiring API.

use std::sync::{Arc, RwLock};

use crate::adapters::driven::storage::Storage;
use crate::bootstrap::wiring::{WiredApp, build_app_graph};
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::{AppSettings, LlmProviderConfig};

fn settings_with(connection: LlmProviderConfig) -> Arc<RwLock<AppSettings>> {
    let mut settings = AppSettings::default();
    settings.connections = vec![connection];
    settings.narration_connection_id = settings.connections[0].id.clone();
    Arc::new(RwLock::new(settings))
}

fn wired_app(connection: LlmProviderConfig, storage: Arc<Storage>) -> WiredApp {
    build_app_graph(
        settings_with(connection),
        storage,
        Arc::new(Storage::new_in_memory()),
    )
    .expect("build_app_graph should succeed")
}

fn mock_connection() -> LlmProviderConfig {
    LlmProviderConfig {
        id: "test-mock".to_string(),
        name: "Test Mock".to_string(),
        provider: LlmBackendType::Mock,
        model: "mock-model".to_string(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    }
}

#[test]
fn mock_backend_path_returns_recorder_with_mock_provider() {
    let storage = Arc::new(Storage::new_in_memory());

    let wired = wired_app(mock_connection(), storage);
    let recorder = &wired.game_service.llm_recorder;

    assert_eq!(recorder.provider().name(), "Mock");
    assert_eq!(recorder.provider().model(), "mock");
}

#[test]
fn mock_backend_recorder_persists_forensics_to_storage() {
    // Regression guard: the factory must wire `SaveLlmMessageFn` to
    // `Storage::save_llm_message`, not a no-op closure. If the silent fallback
    // is reintroduced, `storage.list_latest_llm_messages(...)` comes back empty.
    let storage = Arc::new(Storage::new_in_memory());

    let wired = wired_app(mock_connection(), Arc::clone(&storage));
    let recorder = &wired.game_service.llm_recorder;

    recorder
        .complete("wiring-test-agent", "sys", "usr", None)
        .expect("recorder.complete should succeed against MockBackend");

    let messages = storage
        .list_latest_llm_messages(10)
        .expect("Storage::list_latest_llm_messages should succeed");
    assert_eq!(
        messages.len(),
        1,
        "expected exactly one LlmMessage persisted to Storage"
    );
    assert_eq!(messages[0].agent_name, "wiring-test-agent");
    assert_eq!(messages[0].backend_name, "Mock");
}

#[test]
fn deepseek_path_returns_recorder() {
    let connection = LlmProviderConfig {
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

    let wired = wired_app(connection, storage);
    let recorder = &wired.game_service.llm_recorder;

    assert_eq!(recorder.provider().name(), "DeepSeek");
}

#[test]
fn openrouter_path_returns_recorder() {
    let connection = LlmProviderConfig {
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

    let wired = wired_app(connection, storage);
    let recorder = &wired.game_service.llm_recorder;

    assert_eq!(recorder.provider().name(), "OpenRouter");
}

#[test]
fn ollama_path_returns_recorder() {
    let connection = LlmProviderConfig {
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

    let wired = wired_app(connection, storage);
    let recorder = &wired.game_service.llm_recorder;

    assert_eq!(recorder.provider().name(), "Ollama");
}

#[test]
fn deepseek_missing_base_url_still_returns_recorder_defers_error() {
    // `from_config()` does not fail on missing fields at construction.
    // Errors defer to call time (e.g., when `complete()` runs). This test
    // documents that the public composition entry point preserves the
    // same deferred-error contract.
    let connection = LlmProviderConfig {
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

    let wired = wired_app(connection, storage);

    assert_eq!(
        wired.game_service.llm_recorder.provider().name(),
        "DeepSeek"
    );
}
