use askama::Template;

use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::{AppSettings, LlmProviderConfig};
use crate::adapters::driving::http::settings_fragment::{SettingsTemplate, parse_api_key};

#[test]
fn test_deepseek_returns_deepseek() {
    assert_eq!(LlmBackendType::from("deepseek"), LlmBackendType::DeepSeek);
}

#[test]
fn test_mock_returns_mock() {
    assert_eq!(LlmBackendType::from("mock"), LlmBackendType::Mock);
}

#[test]
fn test_openrouter_returns_openrouter() {
    assert_eq!(
        LlmBackendType::from("openrouter"),
        LlmBackendType::OpenRouter
    );
}

#[test]
fn test_unknown_returns_mock_default() {
    assert_eq!(
        LlmBackendType::from("unknown_backend"),
        LlmBackendType::Mock
    );
    assert_eq!(LlmBackendType::from(""), LlmBackendType::Mock);
}

#[test]
fn test_ollama_returns_ollama() {
    assert_eq!(LlmBackendType::from("ollama"), LlmBackendType::Ollama);
}

#[test]
fn test_parse_api_key_empty_returns_none() {
    assert_eq!(parse_api_key(""), None);
}

#[test]
fn test_parse_api_key_non_empty_returns_some() {
    assert_eq!(parse_api_key("sk-test123"), Some("sk-test123".to_string()));
    assert_eq!(parse_api_key("   "), Some("   ".to_string()));
}

#[test]
fn test_settings_template_renders_connections() {
    let settings = AppSettings {
        connections: vec![
            LlmProviderConfig {
                id: "conn-1".into(),
                name: "Test Narrator".into(),
                provider: LlmBackendType::OpenRouter,
                model: "openai/gpt-4o-mini".into(),
                api_key: Some("sk-test".into()),
                base_url: None,
                single_user_message: false,
                max_tokens: None,
                max_context_tokens: None,
            },
            LlmProviderConfig {
                id: "conn-2".into(),
                name: "Test Quantifier".into(),
                provider: LlmBackendType::Ollama,
                model: "llama3".into(),
                api_key: None,
                base_url: Some("http://localhost:11434".into()),
                single_user_message: false,
                max_tokens: None,
                max_context_tokens: None,
            },
        ],
        narration_connection_id: "conn-1".into(),
        quantifier_connection_id: "conn-2".into(),
        response_length: "flexible".into(),
        ..Default::default()
    };
    let template = SettingsTemplate::from_settings(&settings);

    assert_eq!(template.connections.len(), 2);
    assert!(template.render().unwrap().contains("conn-1"));
    assert!(template.render().unwrap().contains("conn-2"));
}

#[test]
fn test_narrator_badge_renders() {
    let settings = AppSettings {
        connections: vec![
            LlmProviderConfig {
                id: "conn-1".into(),
                name: "Test Narrator".into(),
                provider: LlmBackendType::OpenRouter,
                model: "openai/gpt-4o-mini".into(),
                api_key: Some("sk-test".into()),
                base_url: None,
                single_user_message: false,
                max_tokens: None,
                max_context_tokens: None,
            },
            LlmProviderConfig {
                id: "conn-2".into(),
                name: "Test Quantifier".into(),
                provider: LlmBackendType::Ollama,
                model: "llama3".into(),
                api_key: None,
                base_url: Some("http://localhost:11434".into()),
                single_user_message: false,
                max_tokens: None,
                max_context_tokens: None,
            },
        ],
        narration_connection_id: "conn-1".into(),
        quantifier_connection_id: "conn-2".into(),
        response_length: "flexible".into(),
        ..Default::default()
    };
    let template = SettingsTemplate::from_settings(&settings);
    let html = template.render().unwrap();

    assert!(html.contains(r#"<span class="badge">Narrator</span>"#));
}

#[test]
fn test_quantifier_badge_renders() {
    let settings = AppSettings {
        connections: vec![
            LlmProviderConfig {
                id: "conn-1".into(),
                name: "Test Narrator".into(),
                provider: LlmBackendType::OpenRouter,
                model: "openai/gpt-4o-mini".into(),
                api_key: Some("sk-test".into()),
                base_url: None,
                single_user_message: false,
                max_tokens: None,
                max_context_tokens: None,
            },
            LlmProviderConfig {
                id: "conn-2".into(),
                name: "Test Quantifier".into(),
                provider: LlmBackendType::Ollama,
                model: "llama3".into(),
                api_key: None,
                base_url: Some("http://localhost:11434".into()),
                single_user_message: false,
                max_tokens: None,
                max_context_tokens: None,
            },
        ],
        narration_connection_id: "conn-1".into(),
        quantifier_connection_id: "conn-2".into(),
        response_length: "flexible".into(),
        ..Default::default()
    };
    let template = SettingsTemplate::from_settings(&settings);
    let html = template.render().unwrap();

    assert!(html.contains(r#"<span class="badge quantifier">Quantifier</span>"#));
}
