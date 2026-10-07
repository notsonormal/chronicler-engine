//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Settings handlers

use axum::body::Body;
use axum::{Form, extract::State, response::Html, response::IntoResponse, response::Response};

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::settings::templates::settings::{
    ConnectionFormTemplate, ConnectionTestResultTemplate, SettingsTemplate, TextCheckCardTemplate,
};
use crate::adapters::driving::http::utils::error::{
    action_error_response, action_refusal_response, error_disclosure, raw_error_detail,
    render_error,
};
use crate::adapters::driving::http::utils::handler_helpers::{
    generate_storage_id, opt_string, render_template,
};
use crate::application::connection_test_service::ConnectionTestResult;
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

impl ConnectionForm {
    /// `id` is the stored connection id, or empty for testing unsaved form values.
    fn into_config(self, id: String, provider: LlmBackendType) -> LlmProviderConfig {
        LlmProviderConfig {
            id,
            name: self.conn_name,
            provider,
            model: self.conn_model,
            api_key: opt_string(&self.conn_api_key),
            base_url: opt_string(&self.conn_base_url),
            single_user_message: self.single_user_message,
            max_tokens: None,
            max_context_tokens: None,
        }
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
        Err(e) => return action_refusal_response(e.to_string()),
    };

    let connection = form.into_config(generate_storage_id("conn"), provider);

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
    let provider = match form.conn_provider.as_str().parse::<LlmBackendType>() {
        Ok(p) => p,
        Err(e) => return action_refusal_response(e.to_string()),
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
    render_template(ConnectionTestResultTemplate {
        backend_name: result.backend_name.clone(),
        model_name: result.model_name.clone(),
        elapsed_ms: result.elapsed_ms,
    })
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
    let provider = match form.conn_provider.as_str().parse::<LlmBackendType>() {
        Ok(p) => p,
        Err(e) => return connection_test_error(&e.to_string(), ConnectionTestSurface::Form),
    };

    let connection = form.into_config(String::new(), provider);
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
