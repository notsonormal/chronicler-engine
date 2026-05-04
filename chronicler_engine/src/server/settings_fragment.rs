//! [DOC: docs/architecture/system.md]

use askama::Template;
use axum::{Form, extract::State, response::Html};

use crate::model::llm_backend::LlmBackendType;
use crate::model::settings::{AppSettings, Connection};
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

fn provider_options_html(selected: &str) -> String {
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
            {{ conn.provider|fmt("{:?}") }} — {{ conn.model }}
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
            <label>
                <input type="checkbox" name="single_user_message" value="true" />
                Single User Message (merge system + user for models that ignore system prompts)
            </label>
        </div>
        <button type="submit" class="primary">Add Connection</button>
    </form>
    <span id="settings-status"></span>
</div>
"##,
    ext = "html"
)]
pub struct SettingsTemplate {
    pub connections: Vec<Connection>,
    pub narration_connection_id: String,
    pub quantifier_connection_id: String,
    pub provider_options: String,
}

impl SettingsTemplate {
    fn from_settings(settings: &AppSettings) -> Self {
        Self {
            connections: settings.connections.clone(),
            narration_connection_id: settings.narration_connection_id.clone(),
            quantifier_connection_id: settings.quantifier_connection_id.clone(),
            provider_options: provider_options_html("openrouter"),
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

    // Return empty string — HTMX will remove the card
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
    <div class="card-details">{:?} — {}</div>
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
            <label>
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::settings::AppSettings;

    mod parse_backend {
        use super::*;

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
        fn test_unknown_returns_openrouter_default() {
            assert_eq!(
                LlmBackendType::from("unknown_backend"),
                LlmBackendType::OpenRouter
            );
            assert_eq!(LlmBackendType::from(""), LlmBackendType::OpenRouter);
        }

        #[test]
        fn test_ollama_returns_ollama() {
            assert_eq!(LlmBackendType::from("ollama"), LlmBackendType::Ollama);
        }
    }

    mod parse_api_key {
        use super::*;

        #[test]
        fn test_empty_returns_none() {
            assert_eq!(parse_api_key(""), None);
        }

        #[test]
        fn test_non_empty_returns_some() {
            assert_eq!(parse_api_key("sk-test123"), Some("sk-test123".to_string()));
            assert_eq!(parse_api_key("   "), Some("   ".to_string()));
        }
    }

    mod settings_template_from_settings {
        use super::*;

        fn make_settings() -> AppSettings {
            AppSettings {
                connections: vec![
                    Connection {
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
                    Connection {
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
            }
        }

        #[test]
        fn test_template_renders_connections() {
            let settings = make_settings();
            let template = SettingsTemplate::from_settings(&settings);

            assert_eq!(template.connections.len(), 2);
            assert!(template.render().unwrap().contains("conn-1"));
            assert!(template.render().unwrap().contains("conn-2"));
        }

        #[test]
        fn test_narrator_badge_renders() {
            let settings = make_settings();
            let template = SettingsTemplate::from_settings(&settings);
            let html = template.render().unwrap();

            assert!(html.contains(r#"<span class="badge">Narrator</span>"#));
        }

        #[test]
        fn test_quantifier_badge_renders() {
            let settings = make_settings();
            let template = SettingsTemplate::from_settings(&settings);
            let html = template.render().unwrap();

            assert!(html.contains(r#"<span class="badge quantifier">Quantifier</span>"#));
        }
    }
}
