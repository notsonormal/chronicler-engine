use crate::model::llm_backend::LlmBackendType;
use crate::model::settings::Connection;
use super::fragments::{connection_card_html, connection_edit_form_html};

#[test]
fn test_connection_card_html_with_both_badges() {
    let conn = Connection {
        id: "test-conn".into(),
        name: "Test Connection".into(),
        provider: LlmBackendType::OpenRouter,
        model: "test-model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let html = connection_card_html(&conn, true, true);

    assert!(html.contains("Test Connection"));
    assert!(html.contains("<span class=\"badge\">Narrator</span>"));
    assert!(html.contains("<span class=\"badge quantifier\">Quantifier</span>"));
    assert!(!html.contains("Set as Narrator"));
    assert!(!html.contains("Set as Quantifier"));
    assert!(html.contains("Edit</button>"));
    assert!(html.contains("Delete</button>"));
}

#[test]
fn test_connection_card_html_with_narrator_only() {
    let conn = Connection {
        id: "test-conn".into(),
        name: "Test".into(),
        provider: LlmBackendType::DeepSeek,
        model: "model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let html = connection_card_html(&conn, true, false);

    assert!(html.contains("<span class=\"badge\">Narrator</span>"));
    assert!(!html.contains("Quantifier</span>"));
    assert!(html.contains("Set as Quantifier"));
}

#[test]
fn test_connection_card_html_with_quantifier_only() {
    let conn = Connection {
        id: "test-conn".into(),
        name: "Test".into(),
        provider: LlmBackendType::Ollama,
        model: "model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let html = connection_card_html(&conn, false, true);

    assert!(!html.contains("Narrator</span>"));
    assert!(html.contains("<span class=\"badge quantifier\">Quantifier</span>"));
    assert!(html.contains("Set as Narrator"));
}

#[test]
fn test_connection_card_html_with_no_badges() {
    let conn = Connection {
        id: "test-conn".into(),
        name: "Test".into(),
        provider: LlmBackendType::Mock,
        model: "model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let html = connection_card_html(&conn, false, false);

    // When neither badge is set, there's no badge content but card-badges div still exists
    assert!(!html.contains("badge\">")); // No badge element with closing quote
    assert!(html.contains("Set as Narrator"));
    assert!(html.contains("Set as Quantifier"));
}

#[test]
fn test_connection_card_html_contains_connection_id() {
    let conn = Connection {
        id: "conn-123".into(),
        name: "Test".into(),
        provider: LlmBackendType::OpenRouter,
        model: "model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let html = connection_card_html(&conn, false, false);

    assert!(html.contains("hx-get=\"/fragment/connections/conn-123/edit\""));
    assert!(html.contains("hx-post=\"/connections/conn-123/delete\""));
    assert!(html.contains("hx-post=\"/connections/conn-123/set-narrator\""));
    assert!(html.contains("hx-post=\"/connections/conn-123/set-quantifier\""));
}

#[test]
fn test_connection_card_html_structure() {
    let conn = Connection {
        id: "test".into(),
        name: "Test".into(),
        provider: LlmBackendType::OpenRouter,
        model: "model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let html = connection_card_html(&conn, false, false);

    assert!(html.contains(r#"<div class="connection-card">"#));
    assert!(html.contains(r#"<div class="card-header">"#));
    assert!(html.contains(r#"<div class="card-details">"#));
    assert!(html.contains(r#"<div class="card-actions">"#));
}

#[test]
fn test_connection_edit_form_html_renders_form() {
    let conn = Connection {
        id: "test-conn".into(),
        name: "My Connection".into(),
        provider: LlmBackendType::OpenRouter,
        model: "anthropic/claude-3".into(),
        api_key: Some("sk-test".into()),
        base_url: Some("https://api.example.com".into()),
        single_user_message: true,
        max_tokens: None,
        max_context_tokens: None,
    };
    let html = connection_edit_form_html(&conn);

    assert!(html.contains(r#"<div class="connection-edit-form">"#));
    assert!(html.contains(r#"Edit My Connection"#));
    assert!(html.contains(r#"name="conn_name" value="My Connection""#));
    assert!(html.contains(r#"name="conn_model" value="anthropic/claude-3""#));
    assert!(html.contains(r#"name="conn_api_key" value="sk-test""#));
    assert!(html.contains(r#"name="conn_base_url" value="https://api.example.com""#));
    assert!(html.contains(r#"checked"#));
}

#[test]
fn test_connection_edit_form_html_correct_provider_selected() {
    let conn = Connection {
        id: "test".into(),
        name: "Test".into(),
        provider: LlmBackendType::DeepSeek,
        model: "model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let html = connection_edit_form_html(&conn);

    assert!(html.contains(r#"<option value="deepseek" selected>DeepSeek</option>"#));
}

#[test]
fn test_connection_edit_form_html_empty_api_key() {
    let conn = Connection {
        id: "test".into(),
        name: "Test".into(),
        provider: LlmBackendType::OpenRouter,
        model: "model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let html = connection_edit_form_html(&conn);

    assert!(html.contains(r#"name="conn_api_key" value="" placeholder="(optional)""#));
}

#[test]
fn test_connection_edit_form_html_non_empty_api_key() {
    let conn = Connection {
        id: "test".into(),
        name: "Test".into(),
        provider: LlmBackendType::OpenRouter,
        model: "model".into(),
        api_key: Some("secret-key".into()),
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let html = connection_edit_form_html(&conn);

    assert!(html.contains(r#"name="conn_api_key" value="secret-key""#));
}

#[test]
fn test_connection_edit_form_html_empty_base_url() {
    let conn = Connection {
        id: "test".into(),
        name: "Test".into(),
        provider: LlmBackendType::OpenRouter,
        model: "model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let html = connection_edit_form_html(&conn);

    assert!(html.contains(r#"name="conn_base_url" value="" placeholder="(optional)""#));
}

#[test]
fn test_connection_edit_form_html_non_empty_base_url() {
    let conn = Connection {
        id: "test".into(),
        name: "Test".into(),
        provider: LlmBackendType::OpenRouter,
        model: "model".into(),
        api_key: None,
        base_url: Some("http://localhost:11434".into()),
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let html = connection_edit_form_html(&conn);

    assert!(html.contains(r#"name="conn_base_url" value="http://localhost:11434""#));
}

#[test]
fn test_connection_edit_form_html_single_user_message_checked() {
    let conn = Connection {
        id: "test".into(),
        name: "Test".into(),
        provider: LlmBackendType::OpenRouter,
        model: "model".into(),
        api_key: None,
        base_url: None,
        single_user_message: true,
        max_tokens: None,
        max_context_tokens: None,
    };
    let html = connection_edit_form_html(&conn);

    assert!(
        html.contains(
            r#"<input type="checkbox" name="single_user_message" value="true" checked />"#
        )
    );
}

#[test]
fn test_connection_edit_form_html_single_user_message_unchecked() {
    let conn = Connection {
        id: "test".into(),
        name: "Test".into(),
        provider: LlmBackendType::OpenRouter,
        model: "model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let html = connection_edit_form_html(&conn);

    // When unchecked, the checkbox doesn't have 'checked' attribute
    assert!(html.contains(r#"name="single_user_message""#));
    assert!(!html.contains("checked"));
}

#[test]
fn test_connection_edit_form_html_escapes_name() {
    let conn = Connection {
        id: "test".into(),
        name: "Test <script>alert('xss')</script>".into(),
        provider: LlmBackendType::OpenRouter,
        model: "model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    let html = connection_edit_form_html(&conn);

    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}
