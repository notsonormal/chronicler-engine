//! [DOC: docs/system/dashboard.md]
//! Retry fragment handler

use axum::{body::Body, extract::State, response::Response};

use crate::application::message_editing;
use crate::server::AppState;

use crate::server::fragments::renderers::{app_err_to_response, ctx_or_error, ok};

pub async fn retry_handler(State(state): State<AppState>) -> Response<Body> {
    let ctx = match ctx_or_error(&state) {
        Ok(ctx) => ctx,
        Err(e) => return *e,
    };
    match message_editing::retry(state.application_service.game_service(), ctx) {
        Ok(()) => ok("<span class=\"status ready\">Retrying...</span>"),
        Err(e) => app_err_to_response(e),
    }
}
