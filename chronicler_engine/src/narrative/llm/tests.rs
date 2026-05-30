use crate::error::LlmFailure;
use crate::model::settings::Connection;
use crate::narrative::{
    llm::backend::{LlmBackendType, get_llm_backend_for, LlmCallResult, merge_single_user_message},
    llm_client::ChatCompletionResult,
};
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
    // Clear any environment variable to test the default
    unsafe {
        std::env::remove_var("LLM_BACKEND");
    }
    assert_eq!(LlmBackendType::from_env(), LlmBackendType::OpenRouter);
}

#[test]
fn test_llm_empty_response_error_variant() {
    let err = EngineError::Llm(LlmFailure::EmptyResponse);
    assert_eq!(err.to_string(), "LLM error: LLM returned an empty response");
}

#[test]
fn test_llm_call_result_from_chat_result() {
    let chat = ChatCompletionResult {
        text: "response text".to_string(),
        system_prompt: "system prompt".to_string(),
        user_prompt: "user prompt".to_string(),
        raw_request_json: r#"{"request": "json"}"#.to_string(),
        raw_response_json: r#"{"response": "json"}"#.to_string(),
    };

    let result = LlmCallResult::from_chat_result("narrator", "OpenRouter", "model-name", chat);

    assert_eq!(result.text, "response text");
    assert_eq!(result.system_prompt, "system prompt");
    assert_eq!(result.user_prompt, "user prompt");
    assert_eq!(result.raw_request_json, r#"{"request": "json"}"#);
    assert_eq!(result.raw_response_json, r#"{"response": "json"}"#);
    assert_eq!(result.backend_name, "OpenRouter");
    assert_eq!(result.model_name, "model-name");
    assert_eq!(result.agent_name, "narrator");
}

#[test]
fn test_llm_call_result_to_message() {
    let result = LlmCallResult {
        text: "response".to_string(),
        system_prompt: "system".to_string(),
        user_prompt: "user".to_string(),
        raw_request_json: r#"{"req": "json"}"#.to_string(),
        raw_response_json: r#"{"resp": "json"}"#.to_string(),
        backend_name: "OpenRouter".to_string(),
        model_name: "model".to_string(),
        agent_name: "narrator".to_string(),
    };

    let message = result.to_message();

    assert_eq!(message.agent_name, "narrator");
    assert_eq!(message.backend_name, "OpenRouter");
    assert_eq!(message.model_name, "model");
    assert_eq!(message.system_prompt, "system");
    assert_eq!(message.user_prompt, "user");
    assert_eq!(message.parsed_response, "response");
    assert_eq!(message.raw_request_json, r#"{"req": "json"}"#);
    assert_eq!(message.raw_response_json, r#"{"resp": "json"}"#);
    assert!(message.error_message.is_none());
    assert!(message.created_at > chrono::Utc::now() - chrono::Duration::seconds(1));
}

#[test]
fn test_merge_single_user_message() {
    let result = merge_single_user_message("system prompt", "user text");
    assert_eq!(result, "[SYSTEM]\nsystem prompt\n\nuser text");
}

#[test]
fn test_merge_single_user_message_with_newlines() {
    let result = merge_single_user_message("system\nwith\nnewlines", "user\ntext");
    assert_eq!(result, "[SYSTEM]\nsystem\nwith\nnewlines\n\nuser\ntext");
}

#[test]
fn test_merge_single_user_message_empty_system() {
    let result = merge_single_user_message("", "user text");
    assert_eq!(result, "[SYSTEM]\n\n\nuser text");
}

#[test]
fn test_merge_single_user_message_empty_user() {
    let result = merge_single_user_message("system prompt", "");
    assert_eq!(result, "[SYSTEM]\nsystem prompt\n\n");
}
