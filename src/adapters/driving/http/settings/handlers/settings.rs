//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Settings handlers

use axum::body::Body;
use axum::{Form, extract::State, response::Html, response::IntoResponse, response::Response};

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::builders::headers::role_label;
use crate::adapters::driving::http::settings::templates::settings::{
    ConnectionFormTemplate, ConnectionTestResultTemplate, SettingsTemplate, TextCheckCardTemplate,
};
use crate::adapters::driving::http::utils::error::{
    action_error_response, action_failure_response, action_refusal_response, error_disclosure,
    raw_error_detail, render_error,
};
use crate::adapters::driving::http::utils::fragment::render_or_error_fragment;
use crate::adapters::driving::http::utils::handler_helpers::{generate_storage_id, opt_string};
use crate::application::connection_test_service::ConnectionTestResult;
use crate::domain::model::agent::Role;
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::{AppSettings, LlmConnection, ReasoningEffort, TextCheckMode};
use crate::error::EngineError;

pub async fn settings_panel(State(app_state): State<AppState>) -> Html<String> {
    render_settings_panel(&app_state, None)
}

/// A refusal keeps the Connections sub-tab in place, so its message rides inside the panel rather than replacing it.
fn render_settings_panel(app_state: &AppState, error: Option<&EngineError>) -> Html<String> {
    match app_state.settings() {
        Ok(settings) => match app_state.game_view_query.role_health() {
            Ok(roles) => Html(render_or_error_fragment(&SettingsTemplate::from_settings(
                &settings, &roles, error,
            ))),
            Err(e) => Html(render_error(&e.to_string())),
        },
        Err(e) => Html(render_error(&e.to_string())),
    }
}

fn roles_using(settings: &AppSettings, connection_id: &str) -> Vec<&'static str> {
    let mut roles = Vec::new();
    if settings.narration_connection_id == connection_id {
        roles.push(role_label(Role::Narrator));
    }
    if settings.quantifier_connection_id == connection_id {
        roles.push(role_label(Role::Quantifier));
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
    pub conn_max_context_tokens: String,
    #[serde(default)]
    pub conn_reasoning_effort: String,
    #[serde(default)]
    pub single_user_message: bool,
}

impl ConnectionForm {
    /// A blank field follows the provider default: `resolve_max_context_tokens`
    /// decides it. Anything but a positive whole number is refused.
    fn resolved_max_context_tokens(&self) -> Result<Option<u32>, String> {
        let typed = self.conn_max_context_tokens.trim();
        if typed.is_empty() {
            return Ok(None);
        }
        match typed.parse::<u32>() {
            Ok(tokens) if tokens > 0 => Ok(Some(tokens)),
            _ => Err(
                "Context window must be a positive whole number, or empty for the provider \
                 default"
                    .to_string(),
            ),
        }
    }

    /// A blank field sends no reasoning field, so the model default applies.
    /// Anything but a listed effort is refused.
    fn resolved_reasoning_effort(&self) -> Result<Option<ReasoningEffort>, String> {
        let typed = self.conn_reasoning_effort.trim();
        if typed.is_empty() {
            return Ok(None);
        }
        typed.parse::<ReasoningEffort>().map(Some).map_err(|_| {
            "Reasoning effort must be one of the listed values, or empty for the provider \
             default"
                .to_string()
        })
    }

    fn resolved_provider(&self) -> Result<LlmBackendType, String> {
        self.conn_provider
            .as_str()
            .parse::<LlmBackendType>()
            .map_err(|e| e.to_string())
    }

    /// The one mapping from the form to a connection, shared by add, edit and the
    /// form test. `id` is the stored connection id, or empty for testing unsaved
    /// form values. The form does not own `max_tokens`, so it comes back `None`.
    fn into_config(self, id: String) -> Result<LlmConnection, String> {
        let provider = self.resolved_provider()?;
        let max_context_tokens = self.resolved_max_context_tokens()?;
        let reasoning_effort = self.resolved_reasoning_effort()?;
        Ok(LlmConnection {
            id,
            name: self.conn_name,
            provider,
            model: self.conn_model,
            api_key: opt_string(&self.conn_api_key),
            base_url: opt_string(&self.conn_base_url),
            single_user_message: self.single_user_message,
            max_tokens: None,
            max_context_tokens,
            reasoning_effort,
        })
    }
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
) -> Response<Body> {
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
        Ok(updated) => Html(render_or_error_fragment(
            &TextCheckCardTemplate::from_settings(&updated, "Saved"),
        ))
        .into_response(),
        Err(e) => action_failure_response(format!("Save failed: {e}")),
    }
}

pub async fn new_connection_form(State(_app_state): State<AppState>) -> Html<String> {
    Html(render_or_error_fragment(&ConnectionFormTemplate::new(None)))
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
        Some(conn) => Html(render_or_error_fragment(&ConnectionFormTemplate::new(
            Some(conn),
        ))),
        None => Html(render_error("Connection not found")),
    }
}

pub async fn add_connection_handler(
    State(app_state): State<AppState>,
    Form(form): Form<ConnectionForm>,
) -> Response<Body> {
    let connection = match form.into_config(generate_storage_id("conn")) {
        Ok(connection) => connection,
        Err(message) => return action_refusal_response(message),
    };

    match app_state.settings_service.add_connection(connection) {
        Ok(_) => render_settings_panel(&app_state, None).into_response(),
        Err(e) => action_error_response(e, "Error"),
    }
}

pub async fn edit_connection_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Form(form): Form<ConnectionForm>,
) -> Response<Body> {
    let replacement = match form.into_config(id.clone()) {
        Ok(connection) => connection,
        Err(message) => return action_refusal_response(message),
    };
    let outcome = app_state
        .settings_service
        .update_connection(&id, move |connection| {
            let max_tokens = connection.max_tokens;
            *connection = LlmConnection {
                max_tokens,
                ..replacement
            };
        });

    match outcome {
        Ok(_) => render_settings_panel(&app_state, None).into_response(),
        Err(e) => action_error_response(e, "Error"),
    }
}

#[derive(Clone, Copy)]
enum ConnectionTestSurface<'a> {
    /// A saved connection row, identified by its connection id.
    SavedConnection(&'a str),
    /// The shared Add/Edit form, testing values typed before Save.
    Form,
}

impl ConnectionTestSurface<'_> {
    fn popover_id(&self) -> String {
        match self {
            Self::SavedConnection(id) => format!("connection-test-{id}-popover"),
            Self::Form => "connection-test-form-popover".to_string(),
        }
    }
}

/// A 200 keeps htmx's swap inside the result slot, so the connection row or form is never replaced.
fn connection_test_error(message: &str, surface: ConnectionTestSurface<'_>) -> Html<String> {
    Html(error_disclosure(
        &surface.popover_id(),
        "The connection test failed.",
        &raw_error_detail(message),
    ))
}

fn connection_test_success(result: &ConnectionTestResult) -> Html<String> {
    Html(render_or_error_fragment(&ConnectionTestResultTemplate {
        backend_name: result.backend_name.clone(),
        model_name: result.model_name.clone(),
        elapsed_ms: result.elapsed_ms,
    }))
}

/// The provider's `complete` is sync (`reqwest::blocking`), so the test runs on
/// the blocking pool to keep the async runtime free.
async fn run_connection_test<T>(
    test: impl FnOnce() -> Result<T, crate::application::errors::ApplicationError> + Send + 'static,
) -> Result<T, String>
where
    T: Send + 'static,
{
    match tokio::task::spawn_blocking(test).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(e)) => Err(e.to_string()),
        Err(join_error) => Err(format!("The test could not run: {join_error}")),
    }
}

pub async fn test_saved_connection_handler(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Html<String> {
    let service = app_state.connection_test_service.clone();
    let surface_id = id.clone();
    let surface = ConnectionTestSurface::SavedConnection(&surface_id);
    match run_connection_test(move || service.test_saved_connection(&id)).await {
        Ok(result) => connection_test_success(&result),
        Err(message) => connection_test_error(&message, surface),
    }
}

pub async fn test_form_connection_handler(
    State(app_state): State<AppState>,
    Form(form): Form<ConnectionForm>,
) -> Html<String> {
    let connection = match form.into_config(String::new()) {
        Ok(connection) => connection,
        Err(message) => return connection_test_error(&message, ConnectionTestSurface::Form),
    };
    let service = app_state.connection_test_service.clone();
    let outcome = run_connection_test(move || service.test_connection(&connection)).await;
    match outcome {
        Ok(result) => connection_test_success(&result),
        Err(message) => connection_test_error(&message, ConnectionTestSurface::Form),
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
        Err(e) => render_settings_panel(&app_state, Some(&e)),
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
        Err(e) => render_settings_panel(app_state, Some(&e)),
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
