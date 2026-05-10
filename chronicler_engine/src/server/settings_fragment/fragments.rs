use crate::model::llm_backend::LlmBackendType;
use crate::model::settings::Connection;

use super::template::{html_escape, provider_options_html};

pub(crate) fn connection_card_html(
    conn: &Connection,
    is_narrator: bool,
    is_quantifier: bool,
) -> String {
    let mut badges = String::new();
    if is_narrator {
        badges.push_str(r#"<span class="badge">Narrator</span>"#);
    }
    if is_quantifier {
        badges.push_str(r#" <span class="badge quantifier">Quantifier</span>"#);
    }

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

pub(crate) fn connection_edit_form_html(conn: &Connection) -> String {
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
