//! [DOC: docs/system/dashboard.md]
//! Swipe fragment handler

use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
};

use crate::application::application_service::ApplicationError;
use crate::application::message_editing;
use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::op_context_loader::load_op_context_for_active_game;
use crate::error::EngineError;

use crate::adapters::driving::http::fragments::renderers::{ok, render_story_log};

pub async fn switch_swipe_handler(
    State(state): State<AppState>,
    Path((message_id, swipe_index)): Path<(u64, usize)>,
) -> Result<Response<Body>, ApplicationError> {
    let ctx = load_op_context_for_active_game(&state)
        .map_err(|e| ApplicationError::Engine(EngineError::Render(e.to_string())))?;
    message_editing::switch_swipe(ctx, message_id, swipe_index)?;
    let html = render_story_log(&state)
        .map_err(|e| EngineError::Render(format!("Failed to render story log: {e}")))?;
    Ok(ok(html))
}
