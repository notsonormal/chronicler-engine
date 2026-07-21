//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Swipe fragment handler

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
};

use crate::application::application_service::ApplicationError;
use crate::application::message_editing;
use crate::adapters::driving::http::AppState;
use crate::error::EngineError;

use crate::adapters::driving::http::fragments::renderers::{ok, render_story_log};

pub async fn switch_swipe_handler(
    State(state): State<AppState>,
    Path((message_id, swipe_index)): Path<(u64, usize)>,
) -> Result<Response<Body>, ApplicationError> {
    message_editing::switch_swipe(&state.application_service, message_id, swipe_index)?;
    let html = render_story_log(&state)
        .map_err(|e| EngineError::Render(format!("Failed to render story log: {e}")))?;
    Ok(ok(html))
}
