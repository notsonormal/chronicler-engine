//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Settings templates

use askama::Template;

use crate::application::games::view_query::RoleHealth;
use crate::domain::model::agent::Role;
use crate::domain::model::settings::{AppSettings, LlmProviderConfig, TextCheckMode};
use crate::adapters::driving::http::builders::headers::{
    connections_degraded, connections_degraded_marker, role_health_cell, role_health_cell_id,
    role_label,
};
use crate::adapters::driving::http::utils::error::{
    error_disclosure, raw_error_detail, refusal_message,
};
use crate::adapters::driving::http::utils::template_helpers::select_options_html;
use crate::adapters::driving::http::view_models::{SafeHtml, SelectOptionView};
use crate::error::EngineError;

pub struct RoleRowView {
    pub role: String,
    pub label: String,
    pub health_cell_id: String,
    pub set_route: String,
    pub options: Vec<SelectOptionView>,
    pub health_html: SafeHtml,
}

#[derive(Template)]
#[template(
    source = r##"
<div class="settings-panel">
    <div class="settings-subtabs" role="tablist" aria-label="Settings sections">
        <button class="settings-subtab active" role="tab" id="subtab-connections" aria-controls="settings-connections" aria-selected="true">Connections<span class="subtab-marker-slot" id="subtab-connections-marker">{{ degraded_marker }}</span></button>
        <button class="settings-subtab" role="tab" id="subtab-text-check" aria-controls="settings-text-check" aria-selected="false">Text Check</button>
    </div>
    <div class="settings-subtab-panel active" id="settings-connections" role="tabpanel" aria-labelledby="subtab-connections">
        {% match error %}
        {% when Some with (disclosure) %}
        {{ disclosure }}
        {% when None %}
        {% endmatch %}
        <h3 class="section-heading">Roles</h3>
        <div class="role-rows">
            {% for role in roles %}
            <div class="role-row" data-role="{{ role.role }}">
                <label class="role-name" for="role-select-{{ role.role }}">{{ role.label }}</label>
                <select id="role-select-{{ role.role }}" name="connection_id" class="role-connection-select" hx-post="{{ role.set_route }}" hx-trigger="change" hx-target=".settings-panel" hx-swap="innerHTML">
                    {% for option in role.options %}<option value="{{ option.value }}"{% if option.selected %} selected{% endif %}>{{ option.label }}</option>{% endfor %}
                </select>
                <span class="role-health-slot" id="{{ role.health_cell_id }}">{{ role.health_html }}</span>
            </div>
            {% endfor %}
        </div>
        <h3 class="section-heading">Connections</h3>
        <div class="connection-list">
            {% for conn in connections %}
            <div class="connection-row">
                <div class="connection-meta">
                    <span class="connection-name">{{ conn.name }}</span>
                    <span class="connection-provider">{{ conn.provider|fmt("{:?}") }} - {{ conn.model }}</span>
                </div>
                <div class="connection-roles">
                    {% if conn.id == narration_connection_id %}<span class="badge">Narrator</span>{% endif %}
                    {% if conn.id == quantifier_connection_id %}<span class="badge quantifier">Quantifier</span>{% endif %}
                </div>
                <div class="connection-actions">
                    <button type="button" hx-get="/fragment/connections/{{ conn.id }}/edit" hx-target=".settings-panel" hx-swap="innerHTML" class="btn-cyan">Edit</button>
                    <button type="button" hx-post="/connections/{{ conn.id }}/test" hx-target="next .connection-test-slot" hx-swap="innerHTML" class="btn-cyan">Test</button>
                    <button type="button" hx-post="/connections/{{ conn.id }}/delete" hx-confirm="Delete this connection?" hx-target=".settings-panel" hx-swap="innerHTML" class="btn-danger">Delete</button>
                </div>
                <div class="inline-error-slot connection-test-slot"></div>
            </div>
            {% endfor %}
        </div>
        <div class="connection-add">
            <button type="button" hx-get="/fragment/connections/new" hx-target=".settings-panel" hx-swap="innerHTML" class="btn-primary">Add Connection</button>
        </div>
    </div>
    <div class="settings-subtab-panel" id="settings-text-check" role="tabpanel" aria-labelledby="subtab-text-check">
        {{ text_check_card }}
    </div>
</div>
"##,
    ext = "html"
)]
pub struct SettingsTemplate {
    pub roles: Vec<RoleRowView>,
    pub degraded_marker: SafeHtml,
    pub connections: Vec<LlmProviderConfig>,
    pub narration_connection_id: String,
    pub quantifier_connection_id: String,
    pub error: Option<SafeHtml>,
    pub text_check_card: SafeHtml,
}

impl SettingsTemplate {
    pub fn from_settings(
        settings: &AppSettings,
        roles: &[RoleHealth],
        error: Option<&EngineError>,
    ) -> Self {
        Self {
            roles: Self::role_rows(settings, roles),
            degraded_marker: SafeHtml::new(connections_degraded_marker(connections_degraded(
                roles,
            ))),
            connections: settings.connections.clone(),
            narration_connection_id: settings.narration_connection_id.clone(),
            quantifier_connection_id: settings.quantifier_connection_id.clone(),
            error: error.map(|error| {
                SafeHtml::new(error_disclosure(
                    "settings-panel-error",
                    &refusal_message(error),
                    &raw_error_detail(&error.to_string()),
                ))
            }),
            text_check_card: SafeHtml::new(
                TextCheckCardTemplate::from_settings(settings, "")
                    .render()
                    .unwrap_or_default(),
            ),
        }
    }

    fn role_rows(settings: &AppSettings, roles: &[RoleHealth]) -> Vec<RoleRowView> {
        [
            (
                Role::Narrator,
                "set-narrator",
                settings.narration_connection_id.as_str(),
            ),
            (
                Role::Quantifier,
                "set-quantifier",
                settings.quantifier_connection_id.as_str(),
            ),
        ]
        .into_iter()
        .map(|(role, route, selected_id)| {
            let health = roles.iter().find(|health| health.role == role);
            RoleRowView {
                role: role.agent_name().to_string(),
                label: role_label(role).to_string(),
                health_cell_id: role_health_cell_id(role),
                set_route: format!("/connections/{route}"),
                options: settings
                    .connections
                    .iter()
                    .map(|connection| SelectOptionView {
                        value: connection.id.clone(),
                        label: connection.name.clone(),
                        selected: connection.id == selected_id,
                    })
                    .collect(),
                health_html: role_health_cell(role, health),
            }
        })
        .collect()
    }
}

#[derive(Template)]
#[template(
    source = r##"
<div class="settings-panel connection-form-page">
    <button type="button" class="back-link" hx-get="/fragment/settings" hx-target=".settings-panel" hx-swap="innerHTML"><svg class="icon" aria-hidden="true"><use href="#i-chevron-left"/></svg> Connections</button>
    <h2>{% if editing %}Edit {{ name }}{% else %}Add Connection{% endif %}</h2>
    <form hx-post="{{ form_action }}" hx-target=".settings-panel" hx-swap="innerHTML">
        <div class="form-group">
            <label for="conn_name">Name</label>
            <input type="text" id="conn_name" name="conn_name" value="{{ name }}" placeholder="My OpenRouter" />
        </div>
        <div class="form-group">
            <label for="conn_provider">Provider</label>
            <select name="conn_provider" id="conn_provider">
                {{ provider_options }}
            </select>
        </div>
        <div class="form-group">
            <label for="conn_model">Model</label>
            <input type="text" id="conn_model" name="conn_model" value="{{ model }}" placeholder="openai/gpt-4o-mini" />
        </div>
        <div class="form-group">
            <label for="conn_api_key">API Key</label>
            <input type="password" id="conn_api_key" name="conn_api_key" value="{{ api_key }}" placeholder="(optional)" />
        </div>
        <div class="form-group">
            <label for="conn_base_url">Base URL</label>
            <input type="text" id="conn_base_url" name="conn_base_url" value="{{ base_url }}" placeholder="(optional)" />
        </div>
        <div class="form-group">
            <label class="checkbox-label">
                <input type="checkbox" name="single_user_message" value="true" {% if single_user_message %}checked{% endif %} />
                Single User Message (merge system + user for models that ignore system prompts)
            </label>
        </div>
        <div class="inline-error-slot" data-error-slot="connection-form" hidden></div>
        <div class="form-actions">
            <button type="submit" class="btn-primary">Save</button>
            <button type="button" class="btn-cyan" hx-post="/connections/test" hx-include="closest form" hx-target="next .connection-test-slot" hx-swap="innerHTML">Test</button>
            <button type="button" class="btn-cyan" hx-get="/fragment/settings" hx-target=".settings-panel" hx-swap="innerHTML">Cancel</button>
        </div>
        <div class="inline-error-slot connection-test-slot"></div>
    </form>
</div>
"##,
    ext = "html"
)]
pub struct ConnectionFormTemplate {
    pub editing: bool,
    pub name: String,
    pub provider_options: SafeHtml,
    pub model: String,
    pub api_key: String,
    pub base_url: String,
    pub single_user_message: bool,
    pub form_action: String,
}

impl ConnectionFormTemplate {
    pub fn new(connection: Option<&LlmProviderConfig>) -> Self {
        let provider = connection.map(|conn| match conn.provider {
            crate::domain::model::llm_backend::LlmBackendType::OpenRouter => "openrouter",
            crate::domain::model::llm_backend::LlmBackendType::DeepSeek => "deepseek",
            crate::domain::model::llm_backend::LlmBackendType::Ollama => "ollama",
            crate::domain::model::llm_backend::LlmBackendType::Mock => "mock",
        });
        Self {
            editing: connection.is_some(),
            name: connection.map(|conn| conn.name.clone()).unwrap_or_default(),
            provider_options: select_options_html(SelectOptionView::providers(
                provider.unwrap_or("openrouter"),
            )),
            model: connection
                .map(|conn| conn.model.clone())
                .unwrap_or_default(),
            api_key: connection
                .and_then(|conn| conn.api_key.clone())
                .unwrap_or_default(),
            base_url: connection
                .and_then(|conn| conn.base_url.clone())
                .unwrap_or_default(),
            single_user_message: connection.is_some_and(|conn| conn.single_user_message),
            form_action: connection
                .map(|conn| format!("/connections/{}/edit", conn.id))
                .unwrap_or_else(|| "/connections/add".to_string()),
        }
    }
}

#[derive(Template)]
#[template(
    source = r##"<div class="connection-test-result success">OK &middot; {{ backend_name }} {{ model_name }} &middot; {{ elapsed_ms }} ms</div>"##,
    ext = "html"
)]
pub struct ConnectionTestResultTemplate {
    pub backend_name: String,
    pub model_name: String,
    pub elapsed_ms: u128,
}

#[derive(Template)]
#[template(
    source = r##"
<div class="connection-card" id="text-check-card">
    <div class="card-header">
        <span class="card-title">Spell &amp; Grammar Check</span>
        <span class="text-check-status">{{ status }}</span>
    </div>
    <div class="card-details">
        Check player input for spelling and grammar issues before sending to the LLM.
    </div>
    <form class="text-check-form" hx-post="/settings/text-check" hx-trigger="change" hx-target="#text-check-card" hx-swap="outerHTML">
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
            <label class="checkbox-label" for="enable_auto_check">
                <input type="checkbox" id="enable_auto_check" name="enable_auto_check" value="true" {% if enable_auto_check %}checked{% endif %} {% if text_check_disabled %}disabled{% endif %} />
                Check before sending to LLM
            </label>
        </div>
        <div class="inline-error-slot" data-error-slot="text-check-card" hidden></div>
    </form>
</div>
"##,
    ext = "html"
)]
pub struct TextCheckCardTemplate {
    pub text_check_mode: String,
    pub enable_auto_check: bool,
    pub text_check_disabled: bool,
    pub status: String,
}

impl TextCheckCardTemplate {
    pub(crate) fn from_settings(settings: &AppSettings, status: &str) -> Self {
        let text_check_disabled = matches!(settings.text_check.mode, TextCheckMode::Disabled);
        Self {
            text_check_mode: match settings.text_check.mode {
                TextCheckMode::Disabled => "disabled",
                TextCheckMode::Spell => "spell",
                TextCheckMode::Grammar => "grammar",
                TextCheckMode::SpellGrammar => "spell_grammar",
            }
            .to_string(),
            enable_auto_check: settings.text_check.effective_enable_auto_check(),
            text_check_disabled,
            status: status.to_string(),
        }
    }
}
