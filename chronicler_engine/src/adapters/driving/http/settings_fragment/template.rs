//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Settings templates

use askama::Template;

use crate::domain::model::settings::{AppSettings, LlmProviderConfig, TextCheckMode};

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
            <button hx-get="/fragment/connections/{{ conn.id }}/edit" hx-target="closest .connection-card" hx-swap="outerHTML" class="btn-cyan">Edit</button>
            <button hx-post="/connections/{{ conn.id }}/delete" hx-confirm="Delete this connection?" hx-target="closest .connection-card" hx-swap="outerHTML swap:0.3s" class="btn-danger">Delete</button>
            {% if conn.id != narration_connection_id %}
            <button hx-post="/connections/{{ conn.id }}/set-narrator" hx-target=".settings-panel" hx-swap="innerHTML" class="btn-primary">Set as Narrator</button>
            {% endif %}
            {% if conn.id != quantifier_connection_id %}
            <button hx-post="/connections/{{ conn.id }}/set-quantifier" hx-target=".settings-panel" hx-swap="innerHTML" class="btn-primary">Set as Quantifier</button>
            {% endif %}
        </div>
    </div>
    {% endfor %}

    <h3>Add LlmProviderConfig</h3>
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
        <button type="submit" class="btn-primary">Add LlmProviderConfig</button>
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
                <button type="submit" class="btn-primary">Save</button>
            </div>
        </form>
    </div>
</div>
"##,
    ext = "html"
)]
pub struct SettingsTemplate {
    pub connections: Vec<LlmProviderConfig>,
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
