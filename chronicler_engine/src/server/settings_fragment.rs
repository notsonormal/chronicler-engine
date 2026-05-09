//! [DOC: docs/architecture/system.md]

use askama::Template;
use axum::{Form, extract::State, response::Html};

use crate::model::llm_backend::LlmBackendType;
use crate::model::settings::{AppSettings, Connection, TextCheckMode};
use crate::server::AppState;

/// [DOC: docs/architecture/system.md]
pub fn parse_api_key(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

fn provider_option_html(value: &str, label: &str, selected: bool) -> String {
    let sel = if selected { " selected" } else { "" };
    format!(r#"<option value="{value}"{sel}>{label}</option>"#)
}

pub(crate) fn provider_options_html(selected: &str) -> String {
    [
        ("openrouter", "OpenRouter"),
        ("deepseek", "DeepSeek"),
        ("ollama", "Ollama"),
    ]
    .iter()
    .map(|(v, l)| provider_option_html(v, l, *v == selected))
    .collect::<Vec<_>>()
    .join("\n")
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[derive(Template)]
#[template(
    source = r##"
<div class="settings-panel">
    <h2>Connections</h2>
    {% for conn in connections %}
    <div class="connection-card">
        <div class="card-header">
            <span class="card-title">{{ conn.name }}</span>
            <div class="card-badges">
                {% if conn.id == narration_connection_id %}<span class="badge">Narrator</span>{% endif %}
                {% if conn.id == quantifier_connection_id %}<span class="badge quantifier">Quantifier</span>{% endif %}
            </div>
        </div>
        <div class="card-details">
            {{ conn.provider|fmt("{:?}") }} - {{ conn.model }}
        </div>
        <div class="card-actions">
            <button hx-get="/fragment/connections/{{ conn.id }}/edit" hx-target="closest .connection-card" hx-swap="outerHTML">Edit</button>
            <button hx-post="/connections/{{ conn.id }}/delete" hx-confirm="Delete this connection?" hx-target="closest .connection-card" hx-swap="outerHTML swap:0.3s" class="danger">Delete</button>
            {% if conn.id != narration_connection_id %}
            <button hx-post="/connections/{{ conn.id }}/set-narrator" hx-target=".settings-panel" hx-swap="innerHTML" class="primary">Set as Narrator</button>
            {% endif %}
            {% if conn.id != quantifier_connection_id %}
            <button hx-post="/connections/{{ conn.id }}/set-quantifier" hx-target=".settings-panel" hx-swap="innerHTML" class="primary">Set as Quantifier</button>
            {% endif %}
        </div>
    </div>
    {% endfor %}

    <h3>Add Connection</h3>
    <form hx-post="/connections/add" hx-target=".settings-panel" hx-swap="innerHTML">
        <div class="form-group">
            <label for="conn_name">Name</label>
            <input type="text" id="conn_name" name="conn_name" placeholder="My OpenRouter" />
        </div>
        <div class="form-group">
            <label for="conn_provider">Provider</label>
            <select name="conn_provider" id="conn_provider">
                {{ provider_options|safe }}
            </select>
        </div>
        <div class="form-group">
            <label for="conn_model">Model</label>
            <input type="text" id="conn_model" name="conn_model" placeholder="openai/gpt-4o-mini" />
        </div>
        <div class="form-group">
            <label for="conn_api_key">API Key</label>
            <input type="password" id="conn_api_key" name="conn_api_key" placeholder="(optional)" />
        </div>
        <div class="form-group">
            <label for="conn_base_url">Base URL</label>
            <input type="text" id="conn_base_url" name="conn_base_url" placeholder="(optional)" />
        </div>
        <div class="form-group">
            <label class="checkbox-label">
                <input type="checkbox" name="single_user_message" value="true" />
                Single User Message (merge system + user for models that ignore system prompts)
            </label>
        </div>
        <button type="submit" class="primary">Add Connection</button>
    </form>
    <span id="settings-status"></span>

    <h2>Text Check</h2>
    <div class="connection-card">
        <div class="card-header">
            <span class="card-title">Spell &amp; Grammar Check</span>
        </div>
        <div class="card-details">
            Check player input for spelling and grammar issues before sending to the LLM.
        </div>
        <form hx-post="/settings/text-check" hx-target="#settings-status" hx-swap="innerHTML">
            <div class="form-group">
                <label for="check_mode">Check Mode</label>
                <select name="check_mode" id="check_mode">
                    <option value="disabled" {% if text_check_mode == "disabled" %}selected{% endif %}>Disabled</option>
                    <option value="spell" {% if text_check_mode == "spell" %}selected{% endif %}>Spell Check Only</option>
                    <option value="grammar" {% if text_check_mode == "grammar" %}selected{% endif %}>Grammar Check Only</option>
                    <option value="spell_grammar" {% if text_check_mode == "spell_grammar" %}selected{% endif %}>Spell + Grammar</option>
                </select>
            </div>
            <div class="form-group">
                <label class="checkbox-label">
                    <input type="checkbox" name="enable_auto_check" value="true" {% if enable_auto_check %}checked{% endif %} />
                    Check before sending to LLM
                </label>
            </div>
            <div class="form-actions">
                <button type="submit" class="primary">Save</button>
            </div>
        </form>
    </div>
</div>
"##,
    ext = "html"
)]
pub struct SettingsTemplate {
    pub connections: Vec<Connection>,
    pub narration_connection_id: String,
    pub quantifier_connection_id: String,
    pub provider_options: String,
    pub text_check_mode: String,
    pub enable_auto_check: bool,
}

impl SettingsTemplate {
    pub(crate) fn from_settings(settings: &AppSettings) -> Self {
        Self {
            connections: settings.connections.clone(),
            narration_connection_id: settings.narration_connection_id.clone(),
            quantifier_connection_id: settings.quantifier_connection_id.clone(),
            provider_options: provider_options_html("openrouter"),
            text_check_mode: match settings.text_check.mode {
                TextCheckMode::Disabled => "disabled",
                TextCheckMode::Spell => "spell",
                TextCheckMode::Grammar => "grammar",
                TextCheckMode::SpellGrammar => "spell_grammar",
            }
            .to_string(),
            enable_auto_check: settings.text_check.enable_auto_check,
        }
    }
}

/// [DOC: docs/architecture/system.md]
pub async fn settings_panel(State(app_state): State<AppState>) -> Html<String> {
    let settings = match app_state.settings.read() {
        Ok(g) => g,
        Err(_) => {
            return Html(
                "<span class='error'>Internal error: settings lock poisoned</span>".to_string(),
            );
        }
    };
    let template = SettingsTemplate::from_settings(&settings);
    match template.render() {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<span class='error'>Template error: {e}</span>")),
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

/// [DOC: docs/architecture/system.md]
pub async fn save_settings_handler(
    State(app_state): State<AppState>,
    Form(form): Form<SettingsForm>,
) -> Html<String> {
    let mut settings = match app_state.settings.write() {
        Ok(g) => g,
        Err(_) => {
            return Html(
                "<span class='error'>Internal error: settings lock poisoned</span>".to_string(),
            );
        }
    };

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
    let mut settings = match app_state.settings.write() {
        Ok(g) => g,
        Err(_) => {
            return Html(
                "<span class='error'>Internal error: settings lock poisoned</span>".to_string(),
            );
        }
    };

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
    let mut settings = match app_state.settings.write() {
        Ok(g) => g,
        Err(_) => {
            return Html(
                "<span class='error'>Internal error: settings lock poisoned</span>".to_string(),
            );
        }
    };

    let id = format!(
        "conn-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    let api_key = parse_api_key(&form.conn_api_key);
    let base_url = if form.conn_base_url.is_empty() {
        None
    } else {
        Some(form.conn_base_url)
    };

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
    let template = SettingsTemplate::from_settings(&settings);
    match template.render() {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<span class='error'>Template error: {e}</span>")),
    }
}

pub async fn connection_card_fragment(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let settings = match app_state.settings.read() {
        Ok(g) => g,
        Err(_) => return Html("<span class='error'>Lock poisoned</span>".to_string()),
    };

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
    let settings = match app_state.settings.read() {
        Ok(g) => g,
        Err(_) => return Html("<span class='error'>Lock poisoned</span>".to_string()),
    };

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
    let mut settings = match app_state.settings.write() {
        Ok(g) => g,
        Err(_) => {
            return Html(
                "<span class='error'>Internal error: settings lock poisoned</span>".to_string(),
            );
        }
    };

    let conn = match settings.find_connection_mut(&id) {
        Some(c) => c,
        None => return Html("<span class='error'>Connection not found</span>".to_string()),
    };

    conn.name = form.conn_name;
    conn.provider = LlmBackendType::from(form.conn_provider.as_str());
    conn.model = form.conn_model;
    conn.api_key = parse_api_key(&form.conn_api_key);
    conn.base_url = if form.conn_base_url.is_empty() {
        None
    } else {
        Some(form.conn_base_url)
    };
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
    let mut settings = match app_state.settings.write() {
        Ok(g) => g,
        Err(_) => {
            return Html(
                "<span class='error'>Internal error: settings lock poisoned</span>".to_string(),
            );
        }
    };

    let idx = settings.connections.iter().position(|c| c.id == id);
    let idx = match idx {
        Some(i) => i,
        None => return Html("<span class='error'>Connection not found</span>".to_string()),
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
    let mut settings = match app_state.settings.write() {
        Ok(g) => g,
        Err(_) => {
            return Html(
                "<span class='error'>Internal error: settings lock poisoned</span>".to_string(),
            );
        }
    };

    if settings.find_connection(&id).is_none() {
        return Html("<span class='error'>Connection not found</span>".to_string());
    }

    settings.narration_connection_id = id;

    if let Err(e) = settings.save() {
        return Html(format!("<span class='error'>Save failed: {e}</span>"));
    }

    let template = SettingsTemplate::from_settings(&settings);
    match template.render() {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<span class='error'>Template error: {e}</span>")),
    }
}

/// [DOC: docs/architecture/system.md]
pub async fn set_quantifier_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let mut settings = match app_state.settings.write() {
        Ok(g) => g,
        Err(_) => {
            return Html(
                "<span class='error'>Internal error: settings lock poisoned</span>".to_string(),
            );
        }
    };

    if settings.find_connection(&id).is_none() {
        return Html("<span class='error'>Connection not found</span>".to_string());
    }

    settings.quantifier_connection_id = id;

    if let Err(e) = settings.save() {
        return Html(format!("<span class='error'>Save failed: {e}</span>"));
    }

    let template = SettingsTemplate::from_settings(&settings);
    match template.render() {
        Ok(html) => Html(html),
        Err(e) => Html(format!("<span class='error'>Template error: {e}</span>")),
    }
}

// === HTML fragment helpers ===

fn connection_card_html(conn: &Connection, is_narrator: bool, is_quantifier: bool) -> String {
    let badges = if is_narrator {
        r#"<span class="badge">Narrator</span>"#.to_string()
    } else {
        String::new()
    } + if is_quantifier {
        r#" <span class="badge quantifier">Quantifier</span>"#
    } else {
        ""
    };

    let mut actions = String::new();
    actions.push_str(&format!(
        r#"<button hx-get="/fragment/connections/{}/edit" hx-target="closest .connection-card" hx-swap="outerHTML">Edit</button>"#,
        html_escape(&conn.id)
    ));
    actions.push_str(&format!(
        r#"<button hx-post="/connections/{}/delete" hx-confirm="Delete this connection?" hx-target="closest .connection-card" hx-swap="outerHTML swap:0.3s" class="danger">Delete</button>"#,
        html_escape(&conn.id)
    ));
    if !is_narrator {
        actions.push_str(&format!(
            r#"<button hx-post="/connections/{}/set-narrator" hx-target=".settings-panel" hx-swap="innerHTML" class="primary">Set as Narrator</button>"#,
            html_escape(&conn.id)
        ));
    }
    if !is_quantifier {
        actions.push_str(&format!(
            r#"<button hx-post="/connections/{}/set-quantifier" hx-target=".settings-panel" hx-swap="innerHTML" class="primary">Set as Quantifier</button>"#,
            html_escape(&conn.id)
        ));
    }

    format!(
        r#"<div class="connection-card">
    <div class="card-header">
        <span class="card-title">{}</span>
        <div class="card-badges">{}</div>
    </div>
    <div class="card-details">{:?} - {}</div>
    <div class="card-actions">{}</div>
</div>"#,
        html_escape(&conn.name),
        badges,
        conn.provider,
        html_escape(&conn.model),
        actions
    )
}

fn connection_edit_form_html(conn: &Connection) -> String {
    let provider = match conn.provider {
        LlmBackendType::OpenRouter => "openrouter",
        LlmBackendType::DeepSeek => "deepseek",
        LlmBackendType::Ollama => "ollama",
        LlmBackendType::Mock => "mock",
    };
    let api_key_value = conn.api_key.as_deref().unwrap_or("");
    let base_url_value = conn.base_url.as_deref().unwrap_or("");

    format!(
        r#"<div class="connection-edit-form">
    <div class="card-header">
        <span class="card-title">Edit {}</span>
    </div>
    <form hx-post="/connections/{}/edit" hx-target="closest .connection-edit-form" hx-swap="outerHTML">
        <div class="form-group">
            <label for="edit-name-{}">Name</label>
            <input type="text" id="edit-name-{}" name="conn_name" value="{}" />
        </div>
        <div class="form-group">
            <label for="edit-provider-{}">Provider</label>
            <select name="conn_provider" id="edit-provider-{}">
                {}
            </select>
        </div>
        <div class="form-group">
            <label for="edit-model-{}">Model</label>
            <input type="text" id="edit-model-{}" name="conn_model" value="{}" />
        </div>
        <div class="form-group">
            <label for="edit-api-key-{}">API Key</label>
            <input type="password" id="edit-api-key-{}" name="conn_api_key" value="{}" placeholder="(optional)" />
        </div>
        <div class="form-group">
            <label for="edit-base-url-{}">Base URL</label>
            <input type="text" id="edit-base-url-{}" name="conn_base_url" value="{}" placeholder="(optional)" />
        </div>
        <div class="form-group">
            <label class="checkbox-label">
                <input type="checkbox" name="single_user_message" value="true" {} />
                Single User Message (merge system + user for models that ignore system prompts)
            </label>
        </div>
        <div class="form-actions">
            <button type="submit" class="primary">Save</button>
            <button type="button" hx-get="/fragment/connections/{}" hx-target="closest .connection-edit-form" hx-swap="outerHTML">Cancel</button>
        </div>
    </form>
</div>"#,
        html_escape(&conn.name),
        html_escape(&conn.id),
        conn.id,
        conn.id,
        html_escape(&conn.name),
        conn.id,
        conn.id,
        provider_options_html(provider),
        conn.id,
        conn.id,
        html_escape(&conn.model),
        conn.id,
        conn.id,
        html_escape(api_key_value),
        conn.id,
        conn.id,
        html_escape(base_url_value),
        if conn.single_user_message {
            "checked"
        } else {
            ""
        },
        conn.id,
    )
}
