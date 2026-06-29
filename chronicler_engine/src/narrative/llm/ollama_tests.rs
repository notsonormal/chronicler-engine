use std::sync::Arc;

use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::Connection;
use crate::narrative::llm::backend::LlmBackend;
use crate::narrative::llm::ollama::OllamaBackend;
use crate::storage::Storage;

#[test]
fn test_ollama_backend_name() {
    let backend = OllamaBackend::default();
    assert_eq!(backend.name(), "Ollama");
}

#[test]
fn test_ollama_from_connection() {
    let conn = Connection {
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

    let backend = OllamaBackend::from_connection(&conn, None);
    assert_eq!(backend.model(), "llama3");
    assert_eq!(backend.name(), "Ollama");
}

#[test]
fn test_ollama_model() {
    let conn = Connection {
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

    let backend = OllamaBackend::from_connection(&conn, None);
    assert_eq!(backend.model(), "mistral");
}

#[test]
fn test_ollama_save_message_with_storage() {
    let storage = Arc::new(Storage::new_in_memory());
    let conn = Connection {
        id: "test".into(),
        name: "Test".into(),
        provider: LlmBackendType::Ollama,
        model: "llama3".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let backend = OllamaBackend::from_connection(&conn, Some(Arc::clone(&storage)));

    let msg = crate::domain::model::llm_message::LlmMessageBuilder::new()
        .agent_name("test")
        .backend_name("Ollama")
        .model_name("llama3")
        .system_prompt("sys")
        .user_prompt("user")
        .raw_request_json("req")
        .raw_response_json("res")
        .parsed_response("hello")
        .error_message(None::<String>)
        .build();

    backend.save_message(&msg);

    let messages = storage.list_latest_llm_messages(10).unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].agent_name, "test");
}

#[test]
fn test_ollama_save_message_without_storage() {
    let backend = OllamaBackend::default();

    let msg = crate::domain::model::llm_message::LlmMessageBuilder::new()
        .agent_name("test")
        .backend_name("Ollama")
        .model_name("llama3")
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

#[test]
fn test_ollama_preprocess_gemma4_suffix() {
    let backend = OllamaBackend::from_connection(
        &Connection {
            id: "ollama-gemma".into(),
            name: "Gemma".into(),
            provider: crate::domain::model::llm_backend::LlmBackendType::Ollama,
            model: "gemma4:latest".into(),
            api_key: None,
            base_url: None,
            single_user_message: false,
            max_tokens: None,
            max_context_tokens: None,
        },
        None,
    );
    let result = backend.preprocess_user_text("User prompt");
    assert!(result.contains("<|turn>model"));
    assert!(result.contains("<|channel>thought"));
    assert!(result.contains("<channel|>"));
}

#[test]
fn test_ollama_preprocess_gemma_dash_suffix() {
    let backend = OllamaBackend::from_connection(
        &Connection {
            id: "ollama-gemma".into(),
            name: "Gemma".into(),
            provider: crate::domain::model::llm_backend::LlmBackendType::Ollama,
            model: "mradermacher/gemma-4-26b".into(),
            api_key: None,
            base_url: None,
            single_user_message: false,
            max_tokens: None,
            max_context_tokens: None,
        },
        None,
    );
    let result = backend.preprocess_user_text("User prompt");
    assert!(result.contains("<|turn>model"));
}

#[test]
fn test_ollama_preprocess_no_suffix_for_other_models() {
    let backend = OllamaBackend::from_connection(
        &Connection {
            id: "ollama-llama".into(),
            name: "Llama".into(),
            provider: crate::domain::model::llm_backend::LlmBackendType::Ollama,
            model: "llama3:8b".into(),
            api_key: None,
            base_url: None,
            single_user_message: false,
            max_tokens: None,
            max_context_tokens: None,
        },
        None,
    );
    let input = "User prompt";
    let result = backend.preprocess_user_text(input);
    assert_eq!(result, input);
}
