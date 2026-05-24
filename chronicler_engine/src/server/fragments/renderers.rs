use askama::Template;
use axum::{body::Body, http::StatusCode, response::Response};

use crate::application::application_service::ApplicationError;
use crate::error::{EngineError, Result};
use crate::model::state::GameState;
use crate::server::AppState;
use crate::server::templates::{
    ActionAreaTemplate, CharacterHeadshotsTemplate, HeaderTemplate, LlmMessagesTemplate,
    StoryLogTemplate, VisualSidebarTemplate,
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
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

pub fn render_header(state: &AppState) -> Result<String> {
    let game_name = state
        .application_service
        .get_current_game_name(state.as_game_service_context())
        .unwrap_or_else(|_| "Unknown".to_string());
    render_header_unlocked(game_name)
}

pub fn render_story_log(state: &AppState) -> Result<String> {
    let state_guard = state
        .application_service
        .load_state(state.as_game_service_context())?;

    let entries: Vec<_> = state_guard
        .narrative
        .history()
        .iter()
        .take(MAX_LOG_DISPLAY)
        .cloned()
        .collect();
    let has_last_trigger = state_guard.narrative.last_trigger.is_some();
    let template = StoryLogTemplate::new(&entries, has_last_trigger);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

fn render_visual_sidebar_unlocked(state: &GameState) -> Result<String> {
    let room = state
        .current_room()
        .ok_or_else(|| EngineError::RoomNotFound("current room not found".to_string()))?;

    let image_path = room
        .image_path
        .clone()
        .or_else(|| state.world.default_room_image.clone());

    let resolve_headshot = |npc_id: &str| {
        let npc = state.npcs.get(npc_id)?;
        let image_path = npc.sheet.preferred_image()?.to_string();
        let name = npc.sheet.name.clone();
        Some(NpcPortraitView { image_path, name })
    };

    let npc_data: Vec<NpcPortraitView> = state
        .scene
        .npcs_in_area
        .iter()
        .filter_map(|npc| resolve_headshot(&npc.id))
        .collect();

    let vm = VisualSidebarViewModel::new(image_path, room.name.clone(), npc_data);
    let template = VisualSidebarTemplate::new(vm);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

pub fn render_visual_sidebar(state: &AppState) -> Result<String> {
    let state_guard = state
        .application_service
        .load_state(state.as_game_service_context())?;
    render_visual_sidebar_unlocked(&state_guard)
}

/// [DOC: docs/system/game_flow.md]
pub fn render_action_area(state: &AppState) -> Result<String> {
    let state_guard = state
        .application_service
        .load_state(state.as_game_service_context())?;

    let status = state_guard.narrative.input_buffer.status.clone();
    let phase = state_guard.narrative.input_buffer.phase.clone();
    drop(state_guard);

    let vm = ActionAreaViewModel::new(&status, &phase, &[]);
    let template = ActionAreaTemplate::new(vm);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

/// [DOC: docs/system/game_flow.md]
pub fn render_character_headshots(state: &AppState) -> Result<String> {
    let state_guard = state
        .application_service
        .load_state(state.as_game_service_context())?;

    let npc_data: Vec<NpcPortraitView> = state_guard
        .npcs
        .iter()
        .filter_map(|(_npc_id, npc)| {
            let image = npc.sheet.preferred_image()?;
            let name = npc.sheet.name.clone();
            Some(NpcPortraitView {
                image_path: image.to_string(),
                name,
            })
        })
        .collect();

    let template = CharacterHeadshotsTemplate::new(npc_data);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

pub fn render_action_hints(_state: &AppState) -> Result<String> {
    Ok(String::new())
}

pub fn render_llm_messages(state: &AppState) -> Result<String> {
    let messages = state
        .application_service
        .list_latest_llm_messages(state.as_game_service_context(), 50)
        .map_err(|e| {
            crate::error::EngineError::Template(format!("Failed to load LLM messages: {e}"))
        })?;
    let template = LlmMessagesTemplate::new(&messages);
    template
        .render()
        .map_err(|e| crate::error::EngineError::Template(e.to_string()))
}

// ---------------------------------------------------------------------------
// Response builders
// ---------------------------------------------------------------------------

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

/// Maps the common `ApplicationError` variants to HTTP responses.
/// Handlers that need non-standard mappings should match manually.
/// [DOC: docs/architecture/system.md]
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

/// Maps the common `ApplicationError` variants to `(StatusCode, String)` tuples.
/// [DOC: docs/architecture/system.md]
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
