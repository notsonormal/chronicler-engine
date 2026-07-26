use std::sync::{Arc, RwLock};
use axum::Form;
use axum::extract::Path;

use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::{AppSettings, LlmProviderConfig, TextCheckMode};
use crate::adapters::driving::http::settings::handlers::{
    add_connection_handler, connection_card_fragment, delete_connection_handler,
    edit_connection_form, edit_connection_handler, save_settings_handler, save_text_check_handler,
    set_narrator_handler, set_quantifier_handler, settings_panel, ConnectionForm, SettingsForm,
    TextCheckForm,
};
use crate::adapters::driving::http::AppState;
use crate::adapters::driven::storage::Storage;
use crate::bootstrap::wiring::build_app_graph_for_tests;
use tokio_util::sync::CancellationToken;

fn make_test_app_state() -> AppState {
    let storage = Arc::new(Storage::new_in_memory());
    let settings = Arc::new(RwLock::new(AppSettings::default()));
    let wired = build_app_graph_for_tests(
        Arc::clone(&settings),
        Arc::clone(&storage),
        Arc::new(Storage::new_in_memory()),
        None,
    )
    .expect("build_app_graph_for_tests should succeed");
    AppState::from_wired(wired, CancellationToken::new())
}

fn make_app_state_with_settings(settings: AppSettings) -> AppState {
    let storage = Arc::new(Storage::new_in_memory());
    let settings = Arc::new(RwLock::new(settings));
    let wired = build_app_graph_for_tests(
        Arc::clone(&settings),
        Arc::clone(&storage),
        Arc::new(Storage::new_in_memory()),
        None,
    )
    .expect("build_app_graph_for_tests should succeed");
    AppState::from_wired(wired, CancellationToken::new())
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
async fn test_save_settings_handler_updates_ids() {
    let app_state = make_test_app_state();
    let form = SettingsForm {
        narration_connection_id: "conn-1".into(),
        quantifier_connection_id: "conn-2".into(),
    };

    let response = save_settings_handler(axum::extract::State(app_state.clone()), Form(form)).await;

    assert!(response.0.contains("Settings saved!"));

    let settings = app_state.settings.read().unwrap();
    assert_eq!(settings.narration_connection_id, "conn-1");
    assert_eq!(settings.quantifier_connection_id, "conn-2");
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

    assert!(response.0.contains("Text check settings saved!"));

    let settings = app_state.settings.read().unwrap();
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

    let response =
        save_text_check_handler(axum::extract::State(app_state.clone()), Form(form)).await;

    assert!(response.0.contains("Text check settings saved!"));

    let settings = app_state.settings.read().unwrap();
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

    let settings = app_state.settings.read().unwrap();
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

    let settings = app_state.settings.read().unwrap();
    assert_eq!(settings.text_check.mode, TextCheckMode::Disabled);
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

    assert!(response.0.contains("<div class=\"settings-panel\">"));

    let settings = app_state.settings.read().unwrap();
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

    let settings = app_state.settings.read().unwrap();
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

    let settings = app_state.settings.read().unwrap();
    let new_conn = settings.connections.last().unwrap();
    assert_eq!(new_conn.base_url, Some("http://localhost:11434".into()));
}

#[tokio::test]
async fn test_connection_card_fragment_returns_card() {
    let mut settings = AppSettings::default();
    let conn = LlmProviderConfig {
        id: "test-conn".into(),
        name: "Test".into(),
        provider: LlmBackendType::OpenRouter,
        model: "model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    };
    settings.connections.push(conn);
    let app_state = make_app_state_with_settings(settings);

    let response =
        connection_card_fragment(axum::extract::State(app_state), Path("test-conn".into())).await;

    assert!(response.0.contains("Test"));
    assert!(response.0.contains("connection-card"));
}

#[tokio::test]
async fn test_connection_card_fragment_not_found() {
    let app_state = make_test_app_state();

    let response =
        connection_card_fragment(axum::extract::State(app_state), Path("missing".into())).await;

    assert!(response.0.contains("LlmProviderConfig not found"));
}

#[tokio::test]
async fn test_edit_connection_form_returns_form() {
    let mut settings = AppSettings::default();
    settings.connections.push(LlmProviderConfig {
        id: "test-conn".into(),
        name: "Test".into(),
        provider: LlmBackendType::DeepSeek,
        model: "deepseek-chat".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    });
    let app_state = make_app_state_with_settings(settings);

    let response =
        edit_connection_form(axum::extract::State(app_state), Path("test-conn".into())).await;

    assert!(response.0.contains("<div class=\"connection-edit-form\">"));
    assert!(response.0.contains("Edit Test"));
}

#[tokio::test]
async fn test_edit_connection_form_not_found() {
    let app_state = make_test_app_state();

    let response =
        edit_connection_form(axum::extract::State(app_state), Path("missing".into())).await;

    assert!(response.0.contains("LlmProviderConfig not found"));
}

#[tokio::test]
async fn test_edit_connection_handler_updates_connection() {
    let mut settings = AppSettings::default();
    settings.connections.push(LlmProviderConfig {
        id: "test-conn".into(),
        name: "Old Name".into(),
        provider: LlmBackendType::OpenRouter,
        model: "old-model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    });
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

    assert!(response.0.contains("New Name"));
    assert!(response.0.contains("connection-card"));
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

    assert!(response.0.contains("LlmProviderConfig not found"));
}

#[tokio::test]
async fn test_delete_connection_handler_removes_connection() {
    let mut settings = AppSettings::default();
    settings.connections.clear();
    settings.connections.push(LlmProviderConfig {
        id: "conn-1".into(),
        name: "First".into(),
        provider: LlmBackendType::OpenRouter,
        model: "model1".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    });
    settings.connections.push(LlmProviderConfig {
        id: "conn-2".into(),
        name: "Second".into(),
        provider: LlmBackendType::DeepSeek,
        model: "model2".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    });
    let app_state = make_app_state_with_settings(settings);

    let response = delete_connection_handler(
        axum::extract::State(app_state.clone()),
        Path("conn-1".into()),
    )
    .await;

    assert!(response.0.is_empty());

    let settings = app_state.settings.read().unwrap();
    assert_eq!(settings.connections.len(), 1);
    assert_eq!(settings.connections[0].id, "conn-2");
}

#[tokio::test]
async fn test_delete_connection_handler_redirects_narrator() {
    let mut settings = AppSettings::default();
    settings.connections.clear();
    settings.narration_connection_id = "conn-1".into();
    settings.connections.push(LlmProviderConfig {
        id: "conn-1".into(),
        name: "First".into(),
        provider: LlmBackendType::OpenRouter,
        model: "model1".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    });
    settings.connections.push(LlmProviderConfig {
        id: "conn-2".into(),
        name: "Second".into(),
        provider: LlmBackendType::DeepSeek,
        model: "model2".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    });
    let app_state = make_app_state_with_settings(settings);

    let _response = delete_connection_handler(
        axum::extract::State(app_state.clone()),
        Path("conn-1".into()),
    )
    .await;

    let settings = app_state.settings.read().unwrap();
    assert_eq!(settings.narration_connection_id, "conn-2");
}

#[tokio::test]
async fn test_delete_connection_handler_not_found() {
    let app_state = make_test_app_state();

    let response =
        delete_connection_handler(axum::extract::State(app_state), Path("missing".into())).await;

    assert!(response.0.contains("LlmProviderConfig not found"));
}

#[tokio::test]
async fn test_delete_connection_handler_cannot_delete_last() {
    let mut settings = AppSettings::default();
    settings.connections.clear();
    settings.connections.push(LlmProviderConfig {
        id: "only-conn".into(),
        name: "Only".into(),
        provider: LlmBackendType::OpenRouter,
        model: "model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    });
    let app_state = make_app_state_with_settings(settings);

    let response =
        delete_connection_handler(axum::extract::State(app_state), Path("only-conn".into())).await;

    assert!(response.0.contains("Cannot delete the last connection"));
}

#[tokio::test]
async fn test_set_narrator_handler_updates_id() {
    let mut settings = AppSettings::default();
    settings.connections.push(LlmProviderConfig {
        id: "conn-1".into(),
        name: "Test".into(),
        provider: LlmBackendType::OpenRouter,
        model: "model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    });
    settings.narration_connection_id = "old-conn".into();
    let app_state = make_app_state_with_settings(settings);

    let response = set_narrator_handler(
        axum::extract::State(app_state.clone()),
        Path("conn-1".into()),
    )
    .await;

    assert!(response.0.contains("<div class=\"settings-panel\">"));

    let settings = app_state.settings.read().unwrap();
    assert_eq!(settings.narration_connection_id, "conn-1");
}

#[tokio::test]
async fn test_set_narrator_handler_not_found() {
    let app_state = make_test_app_state();

    let response =
        set_narrator_handler(axum::extract::State(app_state), Path("missing".into())).await;

    assert!(response.0.contains("LlmProviderConfig not found"));
}

#[tokio::test]
async fn test_set_quantifier_handler_updates_id() {
    let mut settings = AppSettings::default();
    settings.connections.push(LlmProviderConfig {
        id: "conn-1".into(),
        name: "Test".into(),
        provider: LlmBackendType::OpenRouter,
        model: "model".into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    });
    settings.quantifier_connection_id = "old-conn".into();
    let app_state = make_app_state_with_settings(settings);

    let response = set_quantifier_handler(
        axum::extract::State(app_state.clone()),
        Path("conn-1".into()),
    )
    .await;

    assert!(response.0.contains("<div class=\"settings-panel\">"));

    let settings = app_state.settings.read().unwrap();
    assert_eq!(settings.quantifier_connection_id, "conn-1");
}

#[tokio::test]
async fn test_set_quantifier_handler_not_found() {
    let app_state = make_test_app_state();

    let response =
        set_quantifier_handler(axum::extract::State(app_state), Path("missing".into())).await;

    assert!(response.0.contains("LlmProviderConfig not found"));
}
