//! [DOC: docs/system/dashboard.md]
//! Fragment renderers

use askama::Template;
use axum::{body::Body, http::StatusCode, response::Response};

use crate::application::application_service::ApplicationError;
use crate::error::{EngineError, Result};
use crate::server::AppState;
use crate::server::templates::{
    ActionAreaTemplate, HeaderTemplate, LlmMessagesTemplate, StoryLogTemplate,
    VisualSidebarTemplate,
};
use crate::server::view_models::{ActionAreaViewModel, NpcPortraitView, VisualSidebarViewModel};

const MAX_LOG_DISPLAY: usize = 50;

pub fn render_error(message: &str) -> String {
    format!(
        "<div class=\"error-message\">Error: {}</div>",
        html_escape(message)
    )
}

fn render_header_unlocked(game_name: String) -> Result<String> {
    let template = HeaderTemplate { game_name };
    template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
}

pub fn render_header(state: &AppState) -> Result<String> {
    let game_name = state
        .application_service
        .get_current_game_name(state.as_game_service_context())
        .unwrap_or_else(|_| "Unknown".to_string());
    render_header_unlocked(game_name)
}

pub fn render_story_log(state: &AppState) -> Result<String> {
    let (entries, has_last_trigger) = state
        .application_service
        .get_story_log_entries(state.as_game_service_context())
        .map_err(|e| EngineError::Config(e.to_string()))?;

    let entries: Vec<_> = entries.into_iter().take(MAX_LOG_DISPLAY).collect();
    let template = StoryLogTemplate::new(&entries, has_last_trigger);
    template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
}

pub fn render_visual_sidebar(state: &AppState) -> Result<String> {
    let (room_name, image_path) = state
        .application_service
        .get_current_room_view(state.as_game_service_context())
        .map_err(|e| EngineError::Config(e.to_string()))?;

    let npc_data = state
        .application_service
        .get_npc_headshots(state.as_game_service_context(), true)
        .map_err(|e| EngineError::Config(e.to_string()))?;

    let npc_portraits: Vec<NpcPortraitView> = npc_data
        .into_iter()
        .map(|(image_path, name)| NpcPortraitView { image_path, name })
        .collect();

    let vm = VisualSidebarViewModel::new(image_path, room_name, npc_portraits);
    let template = VisualSidebarTemplate::new(vm);
    template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
}

pub fn render_action_area(state: &AppState) -> Result<String> {
    let (status, phase) = state
        .application_service
        .get_input_status(state.as_game_service_context())
        .map_err(|e| EngineError::Config(e.to_string()))?;

    let vm = ActionAreaViewModel::new(&status, &phase, &[]);
    let template = ActionAreaTemplate::new(vm);
    template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
}

pub fn render_character_headshots(state: &AppState) -> Result<String> {
    use crate::server::templates::CharacterHeadshotsTemplate;
    use askama::Template;

    let npc_data = state
        .application_service
        .get_npc_headshots(state.as_game_service_context(), false)
        .map_err(|e| EngineError::Config(e.to_string()))?;

    let npc_portraits: Vec<NpcPortraitView> = npc_data
        .into_iter()
        .map(|(image_path, name)| NpcPortraitView { image_path, name })
        .collect();

    let template = CharacterHeadshotsTemplate::new(npc_portraits);
    template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
}

pub fn render_action_hints(_state: &AppState) -> Result<String> {
    Ok(String::new())
}

pub fn render_llm_messages(state: &AppState) -> Result<String> {
    let messages = state
        .application_service
        .list_latest_llm_messages(state.as_game_service_context(), 50)
        .map_err(|e| EngineError::Config(e.to_string()))?;

    let template = LlmMessagesTemplate::new(&messages);
    template
        .render()
        .map_err(|e| EngineError::Template(e.to_string()))
}

#[allow(clippy::expect_used)]
pub fn ok(body: impl Into<String>) -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(body.into()))
        .expect("static response body is valid")
}

#[allow(clippy::expect_used)]
pub fn ok_refresh() -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header("HX-Refresh", "true")
        .body(Body::empty())
        .expect("static response body is valid")
}

#[allow(clippy::expect_used)]
pub fn bad_request(body: impl Into<String>) -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(body.into()))
        .expect("static response body is valid")
}

#[allow(clippy::expect_used)]
pub fn internal_error(body: impl Into<String>) -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(body.into()))
        .expect("static response body is valid")
}

#[allow(clippy::expect_used)]
pub fn service_unavailable(body: impl Into<String>) -> Response<Body> {
    Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .body(Body::from(body.into()))
        .expect("static response body is valid")
}

pub fn service_unavailable_generating() -> Response<Body> {
    service_unavailable("<span class=\"status wait\">Generation in progress, please wait...</span>")
}

pub fn app_err_to_response(err: ApplicationError) -> Response<Body> {
    match err {
        ApplicationError::Validation(msg) => bad_request(render_error(&msg)),
        ApplicationError::ConcurrentGeneration => service_unavailable_generating(),
        ApplicationError::ShuttingDown => {
            service_unavailable(render_error("Server is shutting down"))
        }
        ApplicationError::Engine(e) => internal_error(render_error(&e.to_string())),
    }
}

pub fn app_err_to_tuple(err: ApplicationError) -> (StatusCode, String) {
    match err {
        ApplicationError::Validation(msg) => (StatusCode::BAD_REQUEST, render_error(&msg)),
        ApplicationError::ConcurrentGeneration => (
            StatusCode::SERVICE_UNAVAILABLE,
            "<span class=\"status wait\">Generation in progress, please wait...</span>".to_string(),
        ),
        ApplicationError::ShuttingDown => (
            StatusCode::SERVICE_UNAVAILABLE,
            render_error("Server is shutting down"),
        ),
        ApplicationError::Engine(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            render_error(&e.to_string()),
        ),
    }
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
