//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Settings handlers

use axum::body::Body;
use axum::{Form, extract::State, response::Html, response::IntoResponse, response::Response};

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::settings::templates::settings::{
    ConnectionFormTemplate, SettingsTemplate, TextCheckCardTemplate,
};
use crate::adapters::driving::http::utils::error::{error_response, render_error};
use crate::adapters::driving::http::utils::handler_helpers::{
    generate_storage_id, opt_string, render_template,
};
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::{AppSettings, LlmProviderConfig, TextCheckMode};
use crate::error::EngineError;

pub async fn settings_panel(State(app_state): State<AppState>) -> Html<String> {
    render_settings_panel(&app_state, None)
}

/// A refusal keeps the Connections sub-tab in place, so its message rides inside the panel rather than replacing it.
fn render_settings_panel(app_state: &AppState, error: Option<&str>) -> Html<String> {
    match app_state.settings() {
        Ok(settings) => match app_state.game_view_query.role_health() {
            Ok(roles) => render_template(SettingsTemplate::from_settings(&settings, &roles, error)),
            Err(e) => Html(render_error(&e.to_string())),
        },
        Err(e) => Html(render_error(&e.to_string())),
    }
}

/// The `EngineError` payload without its layer prefix.
fn refusal_message(error: &EngineError) -> String {
    match error {
        EngineError::Validation(message) | EngineError::Config(message) => message.clone(),
        other => other.to_string(),
    }
}

fn roles_using(settings: &AppSettings, connection_id: &str) -> Vec<&'static str> {
    let mut roles = Vec::new();
    if settings.narration_connection_id == connection_id {
        roles.push("Narrator");
    }
    if settings.quantifier_connection_id == connection_id {
        roles.push("Quantifier");
    }
    roles
}

fn role_in_use_message(roles: &[&str]) -> String {
    match roles {
        [one] => {
            format!("{one} uses this connection. Change the {one} role above before deleting it.")
        }
        [first, second] => format!(
            "{first} and {second} use this connection. Change those roles above before deleting it."
        ),
        _ => "A role uses this connection. Change that role above before deleting it.".to_string(),
    }
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

#[derive(Debug, serde::Deserialize)]
pub struct RoleForm {
    pub connection_id: String,
}

pub async fn save_text_check_handler(
    State(app_state): State<AppState>,
    Form(form): Form<TextCheckForm>,
) -> Html<String> {
    let mode = match form.check_mode.as_str() {
        "spell" => TextCheckMode::Spell,
        "grammar" => TextCheckMode::Grammar,
        "spell_grammar" => TextCheckMode::SpellGrammar,
        _ => TextCheckMode::Disabled,
    };

    let outcome = app_state.settings_service.update_settings(|settings| {
        settings
            .text_check
            .set_mode_and_auto_check(mode, form.enable_auto_check);
        Ok(settings.clone())
    });

    match outcome {
        Ok(updated) => render_template(TextCheckCardTemplate::from_settings(&updated, "Saved")),
        Err(e) => Html(render_error(&e.to_string())),
    }
}

pub async fn new_connection_form(State(_app_state): State<AppState>) -> Html<String> {
    render_template(ConnectionFormTemplate::new(None))
}

pub async fn edit_connection_form(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let settings = match app_state.settings() {
        Ok(s) => s,
        Err(e) => return Html(render_error(&e.to_string())),
    };

    match settings.find_connection(&id) {
        Some(conn) => render_template(ConnectionFormTemplate::new(Some(conn))),
        None => Html(render_error("Connection not found")),
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
        Ok(_) => render_settings_panel(&app_state, None).into_response(),
        Err(e) => error_response(e, "Error"),
    }
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
        Ok(_) => render_settings_panel(&app_state, None).into_response(),
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

        let roles = roles_using(settings, &id);
        if !roles.is_empty() {
            return Err(EngineError::Validation(role_in_use_message(&roles)));
        }

        settings.connections.remove(idx);
        Ok(())
    });

    match outcome {
        Ok(()) => render_settings_panel(&app_state, None),
        Err(e) => render_settings_panel(&app_state, Some(&refusal_message(&e))),
    }
}

async fn set_role(
    app_state: &AppState,
    connection_id: &str,
    assign: impl FnOnce(&mut AppSettings, &str),
) -> Html<String> {
    let outcome = app_state.settings_service.update_settings(|settings| {
        if settings.find_connection(connection_id).is_none() {
            return Err(EngineError::Config("Connection not found".to_string()));
        }
        assign(settings, connection_id);
        Ok(())
    });

    match outcome {
        Ok(()) => render_settings_panel(app_state, None),
        Err(e) => render_settings_panel(app_state, Some(&refusal_message(&e))),
    }
}

pub async fn set_narrator_handler(
    State(app_state): State<AppState>,
    Form(form): Form<RoleForm>,
) -> Html<String> {
    set_role(&app_state, &form.connection_id, |settings, id| {
        settings.narration_connection_id = id.to_string();
    })
    .await
}

pub async fn set_quantifier_handler(
    State(app_state): State<AppState>,
    Form(form): Form<RoleForm>,
) -> Html<String> {
    set_role(&app_state, &form.connection_id, |settings, id| {
        settings.quantifier_connection_id = id.to_string();
    })
    .await
}
