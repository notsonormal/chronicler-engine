//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Chat window HTTP request handlers.

use askama::Template;
use axum::{
    body::Body,
    extract::{Form, Path, State},
    response::{Html, Response},
};
use serde::Deserialize;

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::templates::TextCheckPreviewTemplate;
use crate::adapters::driving::http::utils::response::{bad_request, internal_error, ok, ok_refresh};
use crate::application::errors::ApplicationError;
use crate::domain::model::settings::TextCheckMode;
use crate::error::EngineError;

pub async fn index_handler() -> Html<String> {
    Html(include_str!("../../../../../../assets/index.html").to_string())
}

pub async fn reset_handler(
    State(state): State<AppState>,
) -> Result<axum::response::Response<Body>, ApplicationError> {
    match state.game_catalogue.reset() {
        Ok(()) => Ok(ok_refresh()),
        Err(e) => Ok(internal_error(e.to_string())),
    }
}

pub async fn retrigger_handler(
    State(state): State<AppState>,
) -> Result<Response<Body>, ApplicationError> {
    state.pipeline.retrigger()?;
    Ok(ok("<span class=\"status ready\">Retriggering...</span>"))
}

pub async fn retry_handler(
    State(state): State<AppState>,
) -> Result<Response<Body>, ApplicationError> {
    state.pipeline.retry()?;
    Ok(ok("<span class=\"status ready\">Retrying...</span>"))
}

pub async fn switch_swipe_handler(
    State(state): State<AppState>,
    Path((message_id, swipe_index)): Path<(u64, usize)>,
) -> Result<Response<Body>, ApplicationError> {
    let current_game_id = state.game_catalogue.current_game_id();
    let is_generating = state.generation_gate.is_busy(current_game_id);
    state
        .message_service
        .switch_swipe(is_generating, message_id, swipe_index)?;
    let html = state
        .render_story_log()
        .map_err(|e| EngineError::Render(format!("Failed to render story log: {e}")))?;
    Ok(ok(html))
}

#[derive(Deserialize)]
pub struct CheckTextForm {
    pub command: String,
}

#[allow(clippy::expect_used)]
pub async fn check_text_handler(
    State(state): State<AppState>,
    Form(form): Form<CheckTextForm>,
) -> axum::response::Response<Body> {
    let text = form.command.trim().to_string();
    if text.is_empty() {
        return bad_request("<span class=\"status error\">Enter text to check</span>");
    }

    let settings = state.settings();

    if settings.text_check.mode == TextCheckMode::Disabled {
        return ok("<span class=\"status ready\">Text check is disabled</span>");
    }

    match state.text_check_service().check_player_input(
        &text,
        settings.text_check.mode,
        &settings.text_check.ignored_words,
    ) {
        Ok(Some(result)) => {
            let template = TextCheckPreviewTemplate::from_check_result(&result);
            match template.render() {
                Ok(html) => ok(html),
                Err(e) => internal_error(format!("Template error: {e}")),
            }
        }
        Ok(None) => ok("<span class=\"status ready\">No issues found</span>"),
        Err(e) => internal_error(format!("Check failed: {e}")),
    }
}
