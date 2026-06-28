//! [DOC: docs/system/dashboard.md]
//! Swipe fragment handler

use axum::{body::Body, extract::State};

use crate::application::application_service::ApplicationError;
use crate::application::message_editing;
use crate::server::AppState;

use crate::server::fragments::renderers::{
    bad_request, internal_error, ok, service_unavailable, service_unavailable_generating,
};

pub async fn switch_swipe_handler(
    State(state): State<AppState>,
    axum::extract::Path((message_id, swipe_index)): axum::extract::Path<(u64, usize)>,
) -> axum::response::Response<Body> {
    let ctx = match state.as_game_service_context() {
        Ok(ctx) => ctx,
        Err(e) => return internal_error(format!("Failed to load context: {e}")),
    };
    match message_editing::switch_swipe(ctx, message_id, swipe_index) {
        Ok(()) => match crate::server::fragments::renderers::render_story_log(&state) {
            Ok(html) => ok(html),
            Err(e) => internal_error(format!("Failed to render story log: {e}")),
        },
        Err(ApplicationError::Validation(msg)) => {
            bad_request(format!("<span class=\"status error\">{msg}</span>"))
        }
        Err(ApplicationError::ConcurrentGeneration) => service_unavailable_generating(),
        Err(ApplicationError::ShuttingDown) => {
            service_unavailable("<span class=\"status error\">Server is shutting down</span>")
        }
        Err(e) => internal_error(format!("Failed to switch swipe: {e}")),
    }
}
