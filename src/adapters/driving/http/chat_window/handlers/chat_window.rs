//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Chat window HTTP request handlers.

use axum::{
    body::Body,
    extract::{Path, State},
    response::{Html, Response},
};

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::utils::response::{internal_error, ok, ok_refresh};
use crate::application::errors::{ApplicationError, ProcessActionResult};

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
    map_process_action_result(
        state.pipeline.retrigger(&state.generation_gate)?,
        "Retriggering...",
    )
}

pub async fn retry_handler(
    State(state): State<AppState>,
) -> Result<Response<Body>, ApplicationError> {
    map_process_action_result(state.pipeline.retry(&state.generation_gate)?, "Retrying...")
}

fn map_process_action_result(
    result: ProcessActionResult,
    started_label: &str,
) -> Result<Response<Body>, ApplicationError> {
    match result {
        ProcessActionResult::Started => Ok(ok(format!(
            "<span class=\"status ready\">{started_label}</span>"
        ))),
        ProcessActionResult::ConcurrentGeneration => {
            Ok(ok("<span class=\"status wait\">Still thinking...</span>"))
        }
        ProcessActionResult::ShuttingDown => Err(ApplicationError::ShuttingDown),
    }
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
    let html = state.render_story_log()?;
    Ok(ok(html))
}
