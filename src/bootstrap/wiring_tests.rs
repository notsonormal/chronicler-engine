//! Unit tests for LLM provider composition through the public wiring API.

use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::bootstrap::wiring::{WiredApp, build_app_graph};
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::{AppSettings, LlmConnection};
use crate::test_support::env_guard::ApiKeyEnvGuard;
use crate::test_support::test_connection;

fn settings_with(connection: LlmConnection) -> AppSettings {
    let mut settings = AppSettings::default();
    settings.connections = vec![connection];
    settings.narration_connection_id = settings.connections[0].id.clone();
    // The default quantifier id is absent from this list, and resolution errors
    // on a dangling reference rather than falling back to Mock.
    settings.quantifier_connection_id = settings.connections[0].id.clone();
    settings
}

fn wired_app(connection: LlmConnection, storage: Arc<Storage>) -> WiredApp {
    storage
        .save_settings(&settings_with(connection))
        .expect("save_settings should succeed");
    build_app_graph(storage, None).expect("build_app_graph should succeed")
}

fn mock_connection() -> LlmConnection {
    LlmConnection {
        model: "mock-model".to_string(),
        ..test_connection("test-mock", LlmBackendType::Mock)
    }
}

#[test]
fn mock_backend_path_returns_recorder_with_mock_provider() {
    let storage = Arc::new(Storage::new_in_memory());

    let wired = wired_app(mock_connection(), storage);
    let recorder = wired.job_deps.narrator.recorder();

    assert_eq!(recorder.provider().expect("resolves").name(), "Mock");
    assert_eq!(
        recorder.provider().expect("resolves").model(),
        "mock-model",
        "a mock connection reports the model it was configured with"
    );
}

#[test]
fn mock_backend_recorder_persists_forensics_to_storage() {
    let storage = Arc::new(Storage::new_in_memory());

    let wired = wired_app(mock_connection(), Arc::clone(&storage));
    let recorder = wired.job_deps.narrator.recorder();

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
    let connection = LlmConnection {
        name: "Test DeepSeek".to_string(),
        model: "deepseek-chat".to_string(),
        api_key: Some("fake-key".to_string()),
        base_url: Some("https://api.deepseek.com".to_string()),
        ..test_connection("test-deepseek", LlmBackendType::DeepSeek)
    };
    let storage = Arc::new(Storage::new_in_memory());

    let wired = wired_app(connection, storage);
    let recorder = wired.job_deps.narrator.recorder();

    assert_eq!(recorder.provider().expect("resolves").name(), "DeepSeek");
}

#[test]
fn openrouter_path_returns_recorder() {
    let connection = LlmConnection {
        name: "Test OpenRouter".to_string(),
        model: "openrouter-model".to_string(),
        api_key: Some("fake-key".to_string()),
        base_url: Some("https://openrouter.ai/api/v1".to_string()),
        ..test_connection("test-openrouter", LlmBackendType::OpenRouter)
    };
    let storage = Arc::new(Storage::new_in_memory());

    let wired = wired_app(connection, storage);
    let recorder = wired.job_deps.narrator.recorder();

    assert_eq!(recorder.provider().expect("resolves").name(), "OpenRouter");
}

#[test]
fn ollama_path_returns_recorder() {
    let connection = LlmConnection {
        name: "Test Ollama".to_string(),
        model: "llama3".to_string(),
        base_url: Some("http://localhost:11434".to_string()),
        ..test_connection("test-ollama", LlmBackendType::Ollama)
    };
    let storage = Arc::new(Storage::new_in_memory());

    let wired = wired_app(connection, storage);
    let recorder = wired.job_deps.narrator.recorder();

    assert_eq!(recorder.provider().expect("resolves").name(), "Ollama");
}

#[test]
fn deepseek_missing_base_url_still_returns_recorder_defers_error() {
    // `from_config()` does not fail on missing fields at construction.
    // Errors defer to call time (e.g., when `complete()` runs). This test
    // documents that the public composition entry point preserves the
    // same deferred-error contract.
    let connection = LlmConnection {
        name: "Test DeepSeek Missing".to_string(),
        model: "deepseek-chat".to_string(),
        ..test_connection("test-deepseek-missing", LlmBackendType::DeepSeek)
    };
    let storage = Arc::new(Storage::new_in_memory());

    let wired = wired_app(connection, storage);

    assert_eq!(
        wired
            .job_deps
            .narrator
            .recorder()
            .provider()
            .expect("resolves")
            .name(),
        "DeepSeek"
    );
}

#[test]
fn openrouter_connection_without_key_reports_missing_api_key() {
    // A keyless OpenRouter connection must fail locally with a message naming the
    // connection rather than the provider's opaque 401 "Missing Authentication header".
    let _guard = ApiKeyEnvGuard::new();
    unsafe {
        std::env::remove_var("OPENROUTER_API_KEY");
    }
    let connection = LlmConnection {
        name: "Keyless OpenRouter".to_string(),
        model: "openrouter-model".to_string(),
        ..test_connection("test-openrouter-keyless", LlmBackendType::OpenRouter)
    };
    let storage = Arc::new(Storage::new_in_memory());

    let wired = wired_app(connection, storage);
    // `dyn LlmProvider` is not `Debug`, so `expect_err` is unavailable here.
    let error = match wired.job_deps.narrator.recorder().provider() {
        Ok(_) => panic!("a keyless OpenRouter connection must not resolve"),
        Err(e) => e,
    };

    let message = error.to_string();
    assert!(
        message.contains("Keyless OpenRouter"),
        "the error must name the connection, got: {message}"
    );
    assert!(
        message.contains("OPENROUTER_API_KEY"),
        "the error must name the environment fallback, got: {message}"
    );
}
