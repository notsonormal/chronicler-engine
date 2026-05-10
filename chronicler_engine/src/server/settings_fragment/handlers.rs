use axum::{Form, extract::State, response::Html};

use crate::model::llm_backend::LlmBackendType;
use crate::model::settings::{Connection, TextCheckMode};
use crate::server::AppState;

use super::fragments::{connection_card_html, connection_edit_form_html};
use super::template::{SettingsTemplate, parse_api_key};

macro_rules! try_lock {
    ($lock:expr) => {
        match $lock {
            Ok(g) => g,
            Err(_) => {
                return Html(
                    "<span class='error'>Internal error: settings lock poisoned</span>".to_string(),
                )
            }
        }
    };
}

fn render_template<T: askama::Template>(template: T) -> Html<String> {
    match template.render() {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<span class='error'>Template error: {e}</span>")),
    }
}

fn opt_string(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

/// [DOC: docs/architecture/system.md]
pub async fn settings_panel(State(app_state): State<AppState>) -> Html<String> {
    let settings = try_lock!(app_state.settings.read());
    render_template(SettingsTemplate::from_settings(&settings))
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

/// [DOC: docs/architecture/system.md]
pub async fn save_settings_handler(
    State(app_state): State<AppState>,
    Form(form): Form<SettingsForm>,
) -> Html<String> {
    let mut settings = try_lock!(app_state.settings.write());

    settings.narration_connection_id = form.narration_connection_id;
    settings.quantifier_connection_id = form.quantifier_connection_id;

    if let Err(e) = settings.save() {
        return Html(format!("<span class='error'>Save failed: {e}</span>"));
    }

    Html("Settings saved!".to_string())
}

/// [DOC: docs/system/text_check.md]
pub async fn save_text_check_handler(
    State(app_state): State<AppState>,
    Form(form): Form<TextCheckForm>,
) -> Html<String> {
    let mut settings = try_lock!(app_state.settings.write());

    settings.text_check.mode = match form.check_mode.as_str() {
        "spell" => TextCheckMode::Spell,
        "grammar" => TextCheckMode::Grammar,
        "spell_grammar" => TextCheckMode::SpellGrammar,
        _ => TextCheckMode::Disabled,
    };
    settings.text_check.enable_auto_check = form.enable_auto_check;

    if let Err(e) = settings.save() {
        return Html(format!("<span class='error'>Save failed: {e}</span>"));
    }

    Html("Text check settings saved!".to_string())
}

/// [DOC: docs/architecture/system.md]
pub async fn add_connection_handler(
    State(app_state): State<AppState>,
    Form(form): Form<ConnectionForm>,
) -> Html<String> {
    let mut settings = try_lock!(app_state.settings.write());

    let id = format!(
        "conn-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    let api_key = parse_api_key(&form.conn_api_key);
    let base_url = opt_string(&form.conn_base_url);

    let connection = Connection {
        id,
        name: form.conn_name,
        provider: LlmBackendType::from(form.conn_provider.as_str()),
        model: form.conn_model,
        api_key,
        base_url,
        single_user_message: form.single_user_message,
        max_tokens: None,
        max_context_tokens: None,
    };

    settings.connections.push(connection);

    if let Err(e) = settings.save() {
        return Html(format!("<span class='error'>Save failed: {e}</span>"));
    }

    // Return full settings panel so the new connection appears
    render_template(SettingsTemplate::from_settings(&settings))
}

pub async fn connection_card_fragment(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let settings = try_lock!(app_state.settings.read());

    let conn = match settings.find_connection(&id) {
        Some(c) => c.clone(),
        None => return Html("<span class='error'>Connection not found</span>".to_string()),
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
    let settings = try_lock!(app_state.settings.read());

    let conn = match settings.find_connection(&id) {
        Some(c) => c.clone(),
        None => return Html("<span class='error'>Connection not found</span>".to_string()),
    };

    Html(connection_edit_form_html(&conn))
}

/// [DOC: docs/architecture/system.md]
pub async fn edit_connection_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Form(form): Form<ConnectionForm>,
) -> Html<String> {
    let mut settings = try_lock!(app_state.settings.write());

    let conn = match settings.find_connection_mut(&id) {
        Some(c) => c,
        None => return Html("<span class='error'>Connection not found</span>".to_string()),
    };

    conn.name = form.conn_name;
    conn.provider = LlmBackendType::from(form.conn_provider.as_str());
    conn.model = form.conn_model;
    conn.api_key = parse_api_key(&form.conn_api_key);
    conn.base_url = opt_string(&form.conn_base_url);
    conn.single_user_message = form.single_user_message;

    let is_narrator = settings.narration_connection_id == id;
    let is_quantifier = settings.quantifier_connection_id == id;

    if let Err(e) = settings.save() {
        return Html(format!("<span class='error'>Save failed: {e}</span>"));
    }

    // Return updated card
    match settings.find_connection(&id) {
        Some(updated_conn) => Html(connection_card_html(
            updated_conn,
            is_narrator,
            is_quantifier,
        )),
        None => Html("<span class='error'>Connection not found after update</span>".to_string()),
    }
}

/// [DOC: docs/architecture/system.md]
pub async fn delete_connection_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let mut settings = try_lock!(app_state.settings.write());

    let Some(idx) = settings.connections.iter().position(|c| c.id == id) else {
        return Html("<span class='error'>Connection not found</span>".to_string());
    };

    if settings.connections.len() <= 1 {
        return Html("<span class='error'>Cannot delete the last connection</span>".to_string());
    }

    settings.connections.remove(idx);

    // Reassign active connections if the deleted one was active
    if settings.narration_connection_id == id {
        settings.narration_connection_id = settings.connections[0].id.clone();
    }
    if settings.quantifier_connection_id == id {
        settings.quantifier_connection_id = settings.connections[0].id.clone();
    }

    if let Err(e) = settings.save() {
        return Html(format!("<span class='error'>Save failed: {e}</span>"));
    }

    // Return empty string - HTMX will remove the card
    Html(String::new())
}

/// [DOC: docs/architecture/system.md]
pub async fn set_narrator_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let mut settings = try_lock!(app_state.settings.write());

    if settings.find_connection(&id).is_none() {
        return Html("<span class='error'>Connection not found</span>".to_string());
    }

    settings.narration_connection_id = id;

    if let Err(e) = settings.save() {
        return Html(format!("<span class='error'>Save failed: {e}</span>"));
    }

    render_template(SettingsTemplate::from_settings(&settings))
}

/// [DOC: docs/architecture/system.md]
pub async fn set_quantifier_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let mut settings = try_lock!(app_state.settings.write());

    if settings.find_connection(&id).is_none() {
        return Html("<span class='error'>Connection not found</span>".to_string());
    }

    settings.quantifier_connection_id = id;

    if let Err(e) = settings.save() {
        return Html(format!("<span class='error'>Save failed: {e}</span>"));
    }

    render_template(SettingsTemplate::from_settings(&settings))
}
