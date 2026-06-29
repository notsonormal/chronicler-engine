//! [DOC: docs/system/dashboard.md]
//! Retriggers fragment handler

use axum::{body::Body, extract::State, response::Response};

use crate::application::message_editing;
use crate::adapters::driving::http::AppState;

use crate::adapters::driving::http::fragments::renderers::{app_err_to_response, ctx_or_error, ok};

pub async fn retrigger_handler(State(state): State<AppState>) -> Response<Body> {
    let ctx = match ctx_or_error(&state) {
        Ok(ctx) => ctx,
        Err(e) => return *e,
    };
    match message_editing::retrigger(state.application_service.game_service(), ctx) {
        Ok(()) => ok("<span class=\"status ready\">Retriggering...</span>"),
        Err(e) => app_err_to_response(e),
    }
}
