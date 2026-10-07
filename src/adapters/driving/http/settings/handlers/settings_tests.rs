//! Settings HTTP handler tests.

use std::sync::Arc;
use axum::Form;
use axum::extract::Path;

use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::{AppSettings, LlmProviderConfig, TextCheckMode};
use crate::adapters::driving::http::settings::handlers::{
    add_connection_handler, delete_connection_handler, edit_connection_form,
    edit_connection_handler, new_connection_form, save_text_check_handler, set_narrator_handler,
    set_quantifier_handler, settings_panel, ConnectionForm, RoleForm, TextCheckForm,
};
use crate::adapters::driving::http::AppState;
use crate::adapters::driven::storage::Storage;
use crate::bootstrap::wiring::build_app_graph_for_tests;
use crate::test_support::body_text;

fn make_test_app_state() -> AppState {
    make_app_state_with_settings(AppSettings::default())
}

fn make_app_state_with_settings(settings: AppSettings) -> AppState {
    let storage = Arc::new(Storage::new_in_memory());
    storage
        .save_settings(&settings)
        .expect("save_settings should succeed");
    let wired = build_app_graph_for_tests(Arc::clone(&storage), None)
        .expect("build_app_graph_for_tests should succeed");
    AppState::from_wired(wired)
}

fn mock_connection(id: &str, name: &str) -> LlmProviderConfig {
    LlmProviderConfig {
        id: id.into(),
        name: name.into(),
        provider: LlmBackendType::Mock,
        model: "mock-model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    }
}

#[tokio::test]
async fn test_settings_panel_returns_html() {
    let app_state = make_test_app_state();
    let response = settings_panel(axum::extract::State(app_state)).await;

    assert!(response.0.contains("<div class=\"settings-panel\">"));
    assert!(response.0.contains("Connections"));
    assert!(response.0.contains("Text Check"));
}

#[tokio::test]
async fn test_save_text_check_handler_spell_mode() {
    let app_state = make_test_app_state();
    let form = TextCheckForm {
        check_mode: "spell".into(),
        enable_auto_check: true,
    };

    let response =
        save_text_check_handler(axum::extract::State(app_state.clone()), Form(form)).await;

    assert!(response.0.contains(r#"id="text-check-card"#));
    assert!(response.0.contains("Saved"));

    let settings = app_state.settings().expect("settings read should succeed");
    assert_eq!(settings.text_check.mode, TextCheckMode::Spell);
    assert!(settings.text_check.enable_auto_check);
}

#[tokio::test]
async fn test_save_text_check_handler_grammar_mode() {
    let app_state = make_test_app_state();
    let form = TextCheckForm {
        check_mode: "grammar".into(),
        enable_auto_check: false,
    };

    let _response =
        save_text_check_handler(axum::extract::State(app_state.clone()), Form(form)).await;

    let settings = app_state.settings().expect("settings read should succeed");
    assert_eq!(settings.text_check.mode, TextCheckMode::Grammar);
    assert!(!settings.text_check.enable_auto_check);
}

#[tokio::test]
async fn test_save_text_check_handler_spell_grammar_mode() {
    let app_state = make_test_app_state();
    let form = TextCheckForm {
        check_mode: "spell_grammar".into(),
        enable_auto_check: true,
    };

    let _response =
        save_text_check_handler(axum::extract::State(app_state.clone()), Form(form)).await;

    let settings = app_state.settings().expect("settings read should succeed");
    assert_eq!(settings.text_check.mode, TextCheckMode::SpellGrammar);
    assert!(settings.text_check.enable_auto_check);
}

#[tokio::test]
async fn test_save_text_check_handler_unknown_mode_defaults_to_disabled() {
    let app_state = make_test_app_state();
    let form = TextCheckForm {
        check_mode: "unknown".into(),
        enable_auto_check: false,
    };

    let _response =
        save_text_check_handler(axum::extract::State(app_state.clone()), Form(form)).await;

    let settings = app_state.settings().expect("settings read should succeed");
    assert_eq!(settings.text_check.mode, TextCheckMode::Disabled);
}

#[tokio::test]
async fn test_save_text_check_handler_disabled_clears_and_disables_checkbox() {
    let app_state = make_test_app_state();
    let form = TextCheckForm {
        check_mode: "disabled".into(),
        enable_auto_check: true,
    };

    let response =
        save_text_check_handler(axum::extract::State(app_state.clone()), Form(form)).await;

    let settings = app_state.settings().expect("settings read should succeed");
    assert_eq!(settings.text_check.mode, TextCheckMode::Disabled);
    assert!(!settings.text_check.enable_auto_check);
    assert!(
        !response
            .0
            .contains(r#"name="enable_auto_check" value="true" checked"#),
        "the disabled card must render the check box unchecked: {}",
        response.0
    );
    assert!(
        response.0.contains("disabled"),
        "the disabled card must render the check box disabled: {}",
        response.0
    );
}

#[tokio::test]
async fn test_new_connection_form_returns_the_add_form() {
    let app_state = make_test_app_state();
    let response = new_connection_form(axum::extract::State(app_state)).await;

    assert!(response.0.contains(r#"<h2>Add Connection</h2>"#));
    assert!(response.0.contains(r#"id="conn_name""#));
    assert!(response.0.contains("&#8249; Connections"));
}

#[tokio::test]
async fn test_edit_connection_form_returns_form() {
    let mut settings = AppSettings::default();
    settings
        .connections
        .push(mock_connection("test-conn", "Test"));
    let app_state = make_app_state_with_settings(settings);

    let response =
        edit_connection_form(axum::extract::State(app_state), Path("test-conn".into())).await;

    assert!(response.0.contains(r#"<h2>Edit Test</h2>"#));
    assert!(
        response
            .0
            .contains(r#"hx-post="/connections/test-conn/edit""#)
    );
}

#[tokio::test]
async fn test_edit_connection_form_not_found() {
    let app_state = make_test_app_state();

    let response =
        edit_connection_form(axum::extract::State(app_state), Path("missing".into())).await;

    assert!(response.0.contains("Connection not found"));
}

#[tokio::test]
async fn test_add_connection_handler_adds_connection() {
    let app_state = make_test_app_state();
    let form = ConnectionForm {
        conn_name: "Test LlmProviderConfig".into(),
        conn_provider: "openrouter".into(),
        conn_model: "openai/gpt-4".into(),
        conn_api_key: "sk-test".into(),
        conn_base_url: "".into(),
        single_user_message: false,
    };

    let response =
        add_connection_handler(axum::extract::State(app_state.clone()), Form(form)).await;
    let body = body_text(response).await;

    assert!(body.contains("<div class=\"settings-panel\">"));

    let settings = app_state.settings().expect("settings read should succeed");
    let new_conn = settings.connections.last().unwrap();
    assert_eq!(new_conn.name, "Test LlmProviderConfig");
    assert_eq!(new_conn.provider, LlmBackendType::OpenRouter);
    assert_eq!(new_conn.model, "openai/gpt-4");
    assert_eq!(new_conn.api_key, Some("sk-test".into()));
    assert_eq!(new_conn.base_url, None);
}

#[tokio::test]
async fn test_add_connection_handler_empty_base_url_is_none() {
    let app_state = make_test_app_state();
    let form = ConnectionForm {
        conn_name: "Test".into(),
        conn_provider: "ollama".into(),
        conn_model: "llama3".into(),
        conn_api_key: "".into(),
        conn_base_url: "".into(),
        single_user_message: false,
    };

    let _response =
        add_connection_handler(axum::extract::State(app_state.clone()), Form(form)).await;

    let settings = app_state.settings().expect("settings read should succeed");
    let new_conn = settings.connections.last().unwrap();
    assert_eq!(new_conn.base_url, None);
}

#[tokio::test]
async fn test_add_connection_handler_non_empty_base_url_is_some() {
    let app_state = make_test_app_state();
    let form = ConnectionForm {
        conn_name: "Test".into(),
        conn_provider: "ollama".into(),
        conn_model: "llama3".into(),
        conn_api_key: "".into(),
        conn_base_url: "http://localhost:11434".into(),
        single_user_message: false,
    };

    let _response =
        add_connection_handler(axum::extract::State(app_state.clone()), Form(form)).await;

    let settings = app_state.settings().expect("settings read should succeed");
    let new_conn = settings.connections.last().unwrap();
    assert_eq!(new_conn.base_url, Some("http://localhost:11434".into()));
}

#[tokio::test]
async fn test_edit_connection_handler_updates_connection() {
    let mut settings = AppSettings::default();
    settings
        .connections
        .push(mock_connection("test-conn", "Old Name"));
    let app_state = make_app_state_with_settings(settings);

    let form = ConnectionForm {
        conn_name: "New Name".into(),
        conn_provider: "deepseek".into(),
        conn_model: "new-model".into(),
        conn_api_key: "new-key".into(),
        conn_base_url: "http://new.url".into(),
        single_user_message: true,
    };

    let response = edit_connection_handler(
        axum::extract::State(app_state.clone()),
        Path("test-conn".into()),
        Form(form),
    )
    .await;
    let body = body_text(response).await;

    assert!(body.contains("New Name"));
    assert!(body.contains(r#"<div class="connection-row">"#));
}

#[tokio::test]
async fn test_edit_connection_handler_not_found() {
    let app_state = make_test_app_state();
    let form = ConnectionForm {
        conn_name: "Test".into(),
        conn_provider: "openrouter".into(),
        conn_model: "model".into(),
        conn_api_key: "".into(),
        conn_base_url: "".into(),
        single_user_message: false,
    };

    let response = edit_connection_handler(
        axum::extract::State(app_state),
        Path("missing".into()),
        Form(form),
    )
    .await;
    let body = body_text(response).await;

    assert!(body.contains("Connection not found"));
}

#[tokio::test]
async fn test_delete_connection_handler_removes_connection() {
    let settings = AppSettings {
        connections: vec![
            mock_connection("conn-1", "First"),
            mock_connection("conn-2", "Second"),
        ],
        narration_connection_id: "conn-2".into(),
        quantifier_connection_id: "conn-2".into(),
        ..AppSettings::default()
    };
    let app_state = make_app_state_with_settings(settings);

    let response = delete_connection_handler(
        axum::extract::State(app_state.clone()),
        Path("conn-1".into()),
    )
    .await;

    assert!(response.0.contains(r#"<div class="settings-panel">"#));

    let settings = app_state.settings().expect("settings read should succeed");
    assert_eq!(settings.connections.len(), 1);
    assert_eq!(settings.connections[0].id, "conn-2");
}

#[tokio::test]
async fn test_delete_connection_handler_refuses_while_a_role_uses_it() {
    let settings = AppSettings {
        connections: vec![
            mock_connection("conn-1", "First"),
            mock_connection("conn-2", "Second"),
        ],
        narration_connection_id: "conn-1".into(),
        quantifier_connection_id: "conn-2".into(),
        ..AppSettings::default()
    };
    let app_state = make_app_state_with_settings(settings);

    let response = delete_connection_handler(
        axum::extract::State(app_state.clone()),
        Path("conn-1".into()),
    )
    .await;

    assert!(
        response.0.contains("Narrator uses this connection"),
        "the refusal must name the role: {}",
        response.0
    );
    assert!(
        response.0.contains("role above"),
        "the refusal must point at the role rows: {}",
        response.0
    );

    let settings = app_state.settings().expect("settings read should succeed");
    assert_eq!(settings.connections.len(), 2);
    assert_eq!(settings.narration_connection_id, "conn-1");
}

#[tokio::test]
async fn test_delete_connection_handler_refuses_while_quantifier_uses_it() {
    let settings = AppSettings {
        connections: vec![
            mock_connection("conn-1", "First"),
            mock_connection("conn-2", "Second"),
        ],
        narration_connection_id: "conn-1".into(),
        quantifier_connection_id: "conn-1".into(),
        ..AppSettings::default()
    };
    let app_state = make_app_state_with_settings(settings);

    let response = delete_connection_handler(
        axum::extract::State(app_state.clone()),
        Path("conn-1".into()),
    )
    .await;

    assert!(
        response
            .0
            .contains("Narrator and Quantifier use this connection"),
        "the refusal must name both roles: {}",
        response.0
    );

    let settings = app_state.settings().expect("settings read should succeed");
    assert_eq!(settings.connections.len(), 2);
}

#[tokio::test]
async fn test_delete_connection_handler_not_found() {
    let app_state = make_test_app_state();

    let response =
        delete_connection_handler(axum::extract::State(app_state), Path("missing".into())).await;

    assert!(response.0.contains("Connection not found"));
}

#[tokio::test]
async fn test_delete_connection_handler_cannot_delete_last() {
    let settings = AppSettings {
        connections: vec![mock_connection("only-conn", "Only")],
        narration_connection_id: "only-conn".into(),
        quantifier_connection_id: "only-conn".into(),
        ..AppSettings::default()
    };
    let app_state = make_app_state_with_settings(settings);

    let response =
        delete_connection_handler(axum::extract::State(app_state), Path("only-conn".into())).await;

    assert!(response.0.contains("Cannot delete the last connection"));
}

#[tokio::test]
async fn test_set_narrator_handler_updates_id() {
    let settings = AppSettings {
        connections: vec![
            mock_connection("conn-1", "First"),
            mock_connection("conn-2", "Second"),
        ],
        narration_connection_id: "conn-2".into(),
        quantifier_connection_id: "conn-2".into(),
        ..AppSettings::default()
    };
    let app_state = make_app_state_with_settings(settings);

    let response = set_narrator_handler(
        axum::extract::State(app_state.clone()),
        Form(RoleForm {
            connection_id: "conn-1".into(),
        }),
    )
    .await;

    assert!(response.0.contains("<div class=\"settings-panel\">"));

    let settings = app_state.settings().expect("settings read should succeed");
    assert_eq!(settings.narration_connection_id, "conn-1");
}

#[tokio::test]
async fn test_set_narrator_handler_not_found() {
    let app_state = make_test_app_state();

    let response = set_narrator_handler(
        axum::extract::State(app_state),
        Form(RoleForm {
            connection_id: "missing".into(),
        }),
    )
    .await;

    assert!(response.0.contains("Connection not found"));
}

#[tokio::test]
async fn test_set_quantifier_handler_updates_id() {
    let settings = AppSettings {
        connections: vec![
            mock_connection("conn-1", "First"),
            mock_connection("conn-2", "Second"),
        ],
        narration_connection_id: "conn-2".into(),
        quantifier_connection_id: "conn-2".into(),
        ..AppSettings::default()
    };
    let app_state = make_app_state_with_settings(settings);

    let response = set_quantifier_handler(
        axum::extract::State(app_state.clone()),
        Form(RoleForm {
            connection_id: "conn-1".into(),
        }),
    )
    .await;

    assert!(response.0.contains("<div class=\"settings-panel\">"));

    let settings = app_state.settings().expect("settings read should succeed");
    assert_eq!(settings.quantifier_connection_id, "conn-1");
}

#[tokio::test]
async fn test_set_quantifier_handler_not_found() {
    let app_state = make_test_app_state();

    let response = set_quantifier_handler(
        axum::extract::State(app_state),
        Form(RoleForm {
            connection_id: "missing".into(),
        }),
    )
    .await;

    assert!(response.0.contains("Connection not found"));
}
