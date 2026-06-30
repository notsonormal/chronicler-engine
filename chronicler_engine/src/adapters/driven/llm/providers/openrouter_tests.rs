use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::Connection;
use crate::application::ports::llm_provider::LlmProvider;
use crate::adapters::driven::llm::providers::openrouter::OpenRouterBackend;

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

    let backend = OpenRouterBackend::from_connection(&conn);
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

    let backend = OpenRouterBackend::from_connection(&conn);
    assert_eq!(backend.model(), "claude-3-opus");
}

// NOTE: save_message tests removed - LlmProvider trait no longer has storage responsibility.
// Message saving is now handled by LlmCallRecorder orchestrator (Phase 2.1).
