use crate::error::LlmFailure;
use crate::model::settings::Connection;
use crate::narrative::llm::backend::{LlmBackendType, get_llm_backend_for};
use crate::{AppSettings, EngineError};

fn make_settings_with_provider(provider: LlmBackendType) -> AppSettings {
    let conn = Connection::new("test-conn", "Test", provider);
    AppSettings {
        connections: vec![conn],
        narration_connection_id: "test-conn".into(),
        quantifier_connection_id: "test-conn".into(),
        response_length: "flexible".into(),
        ..Default::default()
    }
}

#[test]
fn test_get_llm_backend_for_all_types() {
    let openrouter_settings = make_settings_with_provider(LlmBackendType::OpenRouter);
    assert_eq!(
        get_llm_backend_for(&openrouter_settings.narration_connection(), None,).name(),
        "OpenRouter"
    );

    let mock_settings = make_settings_with_provider(LlmBackendType::Mock);
    assert_eq!(
        get_llm_backend_for(&mock_settings.narration_connection(), None,).name(),
        "Mock"
    );

    let deepseek_settings = make_settings_with_provider(LlmBackendType::DeepSeek);
    assert_eq!(
        get_llm_backend_for(&deepseek_settings.narration_connection(), None,).name(),
        "DeepSeek"
    );

    let ollama_settings = make_settings_with_provider(LlmBackendType::Ollama);
    assert_eq!(
        get_llm_backend_for(&ollama_settings.narration_connection(), None,).name(),
        "Ollama"
    );
}

#[test]
fn test_llm_backend_type_from_env_default() {
    assert_eq!(LlmBackendType::from_env(), LlmBackendType::OpenRouter);
}

#[test]
fn test_llm_empty_response_error_variant() {
    let err = EngineError::Llm(LlmFailure::EmptyResponse);
    assert_eq!(err.to_string(), "LLM error: LLM returned an empty response");
}
