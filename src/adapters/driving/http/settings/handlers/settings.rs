//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Settings handlers

use axum::body::Body;
use axum::{Form, extract::State, response::Html, response::IntoResponse, response::Response};

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::builders::connections::{
    connection_card_html, connection_edit_form_html,
};
use crate::adapters::driving::http::settings::templates::settings::SettingsTemplate;
use crate::adapters::driving::http::utils::error::{error_response, render_error};
use crate::adapters::driving::http::utils::handler_helpers::{
    generate_storage_id, opt_string, render_template,
};
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::{LlmProviderConfig, TextCheckMode};
use crate::error::EngineError;

pub async fn settings_panel(State(app_state): State<AppState>) -> Html<String> {
    match app_state.settings() {
        Ok(settings) => render_template(SettingsTemplate::from_settings(&settings)),
        Err(e) => Html(render_error(&e.to_string())),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct SettingsForm {
    pub narration_connection_id: String,
    pub quantifier_connection_id: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ConnectionForm {
    pub conn_name: String,
    pub conn_provider: String,
    pub conn_model: String,
    pub conn_api_key: String,
    pub conn_base_url: String,
    #[serde(default)]
    pub single_user_message: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct TextCheckForm {
    pub check_mode: String,
    #[serde(default)]
    pub enable_auto_check: bool,
}

pub async fn save_settings_handler(
    State(app_state): State<AppState>,
    Form(form): Form<SettingsForm>,
) -> Html<String> {
    let outcome = app_state.settings_service.update_settings(|settings| {
        settings.narration_connection_id = form.narration_connection_id;
        settings.quantifier_connection_id = form.quantifier_connection_id;
        settings.narration_connection()?;
        settings.quantifier_connection()?;
        Ok(())
    });

    match outcome {
        Ok(()) => Html("Settings saved!".to_string()),
        Err(e) => Html(render_error(&e.to_string())),
    }
}

pub async fn save_text_check_handler(
    State(app_state): State<AppState>,
    Form(form): Form<TextCheckForm>,
) -> Html<String> {
    let outcome = app_state.settings_service.update_settings(|settings| {
        settings.text_check.mode = match form.check_mode.as_str() {
            "spell" => TextCheckMode::Spell,
            "grammar" => TextCheckMode::Grammar,
            "spell_grammar" => TextCheckMode::SpellGrammar,
            _ => TextCheckMode::Disabled,
        };
        settings.text_check.enable_auto_check = form.enable_auto_check;
        Ok(())
    });

    match outcome {
        Ok(()) => Html("Text check settings saved!".to_string()),
        Err(e) => Html(render_error(&e.to_string())),
    }
}

pub async fn add_connection_handler(
    State(app_state): State<AppState>,
    Form(form): Form<ConnectionForm>,
) -> Response<Body> {
    let provider = match form.conn_provider.as_str().parse::<LlmBackendType>() {
        Ok(p) => p,
        Err(e) => return Html(render_error(&e.to_string())).into_response(),
    };

    let id = generate_storage_id("conn");

    let connection = LlmProviderConfig {
        id,
        name: form.conn_name,
        provider,
        model: form.conn_model,
        api_key: opt_string(&form.conn_api_key),
        base_url: opt_string(&form.conn_base_url),
        single_user_message: form.single_user_message,
        max_tokens: None,
        max_context_tokens: None,
    };

    match app_state.settings_service.add_connection(connection) {
        Ok(updated) => render_template(SettingsTemplate::from_settings(&updated)).into_response(),
        Err(e) => error_response(e, "Error"),
    }
}

pub async fn connection_card_fragment(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let settings = match app_state.settings() {
        Ok(s) => s,
        Err(e) => return Html(render_error(&e.to_string())),
    };

    let conn = match settings.find_connection(&id) {
        Some(c) => c.clone(),
        None => return Html(render_error("Connection not found")),
    };

    Html(connection_card_html(
        &conn,
        settings.narration_connection_id == id,
        settings.quantifier_connection_id == id,
    ))
}

pub async fn edit_connection_form(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let settings = match app_state.settings() {
        Ok(s) => s,
        Err(e) => return Html(render_error(&e.to_string())),
    };

    let conn = match settings.find_connection(&id) {
        Some(c) => c.clone(),
        None => return Html(render_error("Connection not found")),
    };

    Html(connection_edit_form_html(&conn))
}

pub async fn edit_connection_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Form(form): Form<ConnectionForm>,
) -> Response<Body> {
    let provider = match form.conn_provider.as_str().parse::<LlmBackendType>() {
        Ok(p) => p,
        Err(e) => return Html(render_error(&e.to_string())).into_response(),
    };

    let api_key = opt_string(&form.conn_api_key);
    let base_url = opt_string(&form.conn_base_url);
    let outcome = app_state
        .settings_service
        .update_connection(&id, move |connection| {
            connection.name = form.conn_name;
            connection.provider = provider;
            connection.model = form.conn_model;
            connection.api_key = api_key;
            connection.base_url = base_url;
            connection.single_user_message = form.single_user_message;
        });

    match outcome {
        Ok((connection, is_narrator, is_quantifier)) => Html(connection_card_html(
            &connection,
            is_narrator,
            is_quantifier,
        ))
        .into_response(),
        Err(e) => error_response(e, "Error"),
    }
}

pub async fn delete_connection_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let outcome = app_state.settings_service.update_settings(|settings| {
        let Some(idx) = settings.connections.iter().position(|c| c.id == id) else {
            return Err(EngineError::Config("Connection not found".to_string()));
        };

        if settings.connections.len() <= 1 {
            return Err(EngineError::Config(
                "Cannot delete the last connection".to_string(),
            ));
        }

        settings.connections.remove(idx);

        if settings.narration_connection_id == id {
            settings.narration_connection_id = settings.connections[0].id.clone();
        }
        if settings.quantifier_connection_id == id {
            settings.quantifier_connection_id = settings.connections[0].id.clone();
        }
        Ok(())
    });

    match outcome {
        Ok(()) => Html(String::new()),
        Err(e) => Html(render_error(&e.to_string())),
    }
}

pub async fn set_narrator_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let outcome = app_state.settings_service.update_settings(|settings| {
        if settings.find_connection(&id).is_none() {
            return Err(EngineError::Config("Connection not found".to_string()));
        }
        settings.narration_connection_id = id.clone();
        Ok(settings.clone())
    });

    match outcome {
        Ok(updated) => render_template(SettingsTemplate::from_settings(&updated)),
        Err(e) => Html(render_error(&e.to_string())),
    }
}

pub async fn set_quantifier_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let outcome = app_state.settings_service.update_settings(|settings| {
        if settings.find_connection(&id).is_none() {
            return Err(EngineError::Config("Connection not found".to_string()));
        }
        settings.quantifier_connection_id = id.clone();
        Ok(settings.clone())
    });

    match outcome {
        Ok(updated) => render_template(SettingsTemplate::from_settings(&updated)),
        Err(e) => Html(render_error(&e.to_string())),
    }
}
