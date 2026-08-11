use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::LlmProviderConfig;
use crate::application::ports::llm_provider::LlmProvider;
use crate::adapters::driven::llm::providers::ollama::OllamaBackend;

#[test]
fn test_ollama_backend_name() {
    let backend = OllamaBackend::default();
    assert_eq!(backend.name(), "Ollama");
}

#[test]
fn test_ollama_from_connection() {
    let conn = LlmProviderConfig {
        id: "ollama-1".into(),
        name: "Local Ollama".into(),
        provider: LlmBackendType::Ollama,
        model: "llama3".into(),
        api_key: None,
        base_url: Some("http://localhost:11434".into()),
        single_user_message: true,
        max_tokens: Some(1024),
        max_context_tokens: Some(4096),
    };

    let backend = OllamaBackend::from_config(&conn);
    assert_eq!(backend.model(), "llama3");
    assert_eq!(backend.name(), "Ollama");
}

#[test]
fn test_ollama_model() {
    let conn = LlmProviderConfig {
        id: "test".into(),
        name: "Test".into(),
        provider: LlmBackendType::Ollama,
        model: "mistral".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };

    let backend = OllamaBackend::from_config(&conn);
    assert_eq!(backend.model(), "mistral");
}

#[test]
fn test_ollama_preprocess_gemma4_suffix() {
    let backend = OllamaBackend::from_config(&LlmProviderConfig {
        id: "ollama-gemma".into(),
        name: "Gemma".into(),
        provider: LlmBackendType::Ollama,
        model: "gemma4:latest".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    });
    let result = backend.preprocess_user_text("User prompt");
    assert!(result.contains("<|turn>model"));
    assert!(result.contains("<|channel>thought"));
}

#[test]
fn test_ollama_preprocess_other_models_unchanged() {
    let backend = OllamaBackend::from_config(&LlmProviderConfig {
        id: "ollama-llama".into(),
        name: "Llama".into(),
        provider: LlmBackendType::Ollama,
        model: "llama3".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    });
    let result = backend.preprocess_user_text("User prompt");
    assert_eq!(result, "User prompt");
}
