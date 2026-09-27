//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Settings handlers

use axum::{Form, extract::State, response::Html};

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::builders::connections::{
    connection_card_html, connection_edit_form_html,
};
use crate::adapters::driving::http::settings::templates::settings::SettingsTemplate;
use crate::adapters::driving::http::utils::error::render_error;
use crate::adapters::driving::http::utils::handler_helpers::{opt_string, render_template};
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
) -> Html<String> {
    let provider = match form.conn_provider.as_str().parse::<LlmBackendType>() {
        Ok(p) => p,
        Err(e) => return Html(render_error(&e.to_string())),
    };

    let id = format!(
        "conn-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

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

    // Return a snapshot so the panel renders outside the storage lock.
    let outcome = app_state.settings_service.update_settings(|settings| {
        settings.connections.push(connection.clone());
        Ok(settings.clone())
    });

    match outcome {
        Ok(updated) => render_template(SettingsTemplate::from_settings(&updated)),
        Err(e) => Html(render_error(&e.to_string())),
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
        None => return Html(render_error("LlmProviderConfig not found")),
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
        None => return Html(render_error("LlmProviderConfig not found")),
    };

    Html(connection_edit_form_html(&conn))
}

pub async fn edit_connection_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Form(form): Form<ConnectionForm>,
) -> Html<String> {
    let provider = match form.conn_provider.as_str().parse::<LlmBackendType>() {
        Ok(p) => p,
        Err(e) => return Html(render_error(&e.to_string())),
    };

    let outcome = app_state.settings_service.update_settings(|settings| {
        let is_narrator = settings.narration_connection_id == id;
        let is_quantifier = settings.quantifier_connection_id == id;

        let conn = match settings.find_connection_mut(&id) {
            Some(c) => c,
            None => {
                return Err(EngineError::Config(
                    "LlmProviderConfig not found".to_string(),
                ));
            }
        };

        conn.name = form.conn_name.clone();
        conn.provider = provider;
        conn.model = form.conn_model.clone();
        conn.api_key = opt_string(&form.conn_api_key);
        conn.base_url = opt_string(&form.conn_base_url);
        conn.single_user_message = form.single_user_message;

        let updated = conn.clone();
        Ok((updated, is_narrator, is_quantifier))
    });

    match outcome {
        Ok((conn, is_narrator, is_quantifier)) => {
            Html(connection_card_html(&conn, is_narrator, is_quantifier))
        }
        Err(e) => Html(render_error(&e.to_string())),
    }
}

pub async fn delete_connection_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let outcome = app_state.settings_service.update_settings(|settings| {
        let Some(idx) = settings.connections.iter().position(|c| c.id == id) else {
            return Err(EngineError::Config(
                "LlmProviderConfig not found".to_string(),
            ));
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
            return Err(EngineError::Config(
                "LlmProviderConfig not found".to_string(),
            ));
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
            return Err(EngineError::Config(
                "LlmProviderConfig not found".to_string(),
            ));
        }
        settings.quantifier_connection_id = id.clone();
        Ok(settings.clone())
    });

    match outcome {
        Ok(updated) => render_template(SettingsTemplate::from_settings(&updated)),
        Err(e) => Html(render_error(&e.to_string())),
    }
}
