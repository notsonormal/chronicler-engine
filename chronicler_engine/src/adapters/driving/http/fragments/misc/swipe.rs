//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Swipe fragment handler

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
};

use crate::application::application_service::ApplicationError;
use crate::adapters::driving::http::AppState;
use crate::error::EngineError;

use crate::adapters::driving::http::fragments::renderers::ok;

pub async fn switch_swipe_handler(
    State(state): State<AppState>,
    Path((message_id, swipe_index)): Path<(u64, usize)>,
) -> Result<Response<Body>, ApplicationError> {
    state
        .application_service
        .switch_swipe(message_id, swipe_index)?;
    let html = state
        .render_story_log()
        .map_err(|e| EngineError::Render(format!("Failed to render story log: {e}")))?;
    Ok(ok(html))
}
