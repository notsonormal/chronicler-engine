//! [DOC: docs/system/dashboard.md]
//! Swipe and retry fragment handlers

use axum::{body::Body, extract::State, response::Response};

use crate::application::application_service::ApplicationError;
use crate::server::AppState;

use crate::server::fragments::renderers::{
    app_err_to_response, bad_request, ctx_or_error, internal_error, ok, service_unavailable,
    service_unavailable_generating,
};

pub async fn retry_handler(State(state): State<AppState>) -> Response<Body> {
    let Ok(ctx) = ctx_or_error(&state) else {
        return match ctx_or_error(&state) {
            Ok(_) => unreachable!(),
            Err(e) => *e,
        };
    };
    match state.application_service.retry(ctx) {
        Ok(()) => ok("<span class=\"status ready\">Retrying...</span>"),
        Err(e) => app_err_to_response(e),
    }
}

pub async fn retrigger_handler(State(state): State<AppState>) -> Response<Body> {
    let Ok(ctx) = ctx_or_error(&state) else {
        return match ctx_or_error(&state) {
            Ok(_) => unreachable!(),
            Err(e) => *e,
        };
    };
    match state.application_service.retrigger(ctx) {
        Ok(()) => ok("<span class=\"status ready\">Retriggering...</span>"),
        Err(e) => app_err_to_response(e),
    }
}

pub async fn switch_swipe_handler(
    State(state): State<AppState>,
    axum::extract::Path((message_id, swipe_index)): axum::extract::Path<(u64, usize)>,
) -> axum::response::Response<Body> {
    let ctx = match state.as_game_service_context() {
        Ok(ctx) => ctx,
        Err(e) => return internal_error(format!("Failed to load context: {e}")),
    };
    match state
        .application_service
        .switch_swipe(ctx, message_id, swipe_index)
    {
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
