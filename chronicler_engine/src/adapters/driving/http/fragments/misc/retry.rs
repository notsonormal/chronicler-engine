//! [DOC: docs/system/dashboard.md]
//! Retry fragment handler

use axum::{body::Body, extract::State, response::Response};

use crate::application::application_service::ApplicationError;
use crate::application::message_editing;
use crate::adapters::driving::http::AppState;
use crate::error::EngineError;

use crate::adapters::driving::http::fragments::renderers::{ctx_or_error, ok};

pub async fn retry_handler(
    State(state): State<AppState>,
) -> Result<Response<Body>, ApplicationError> {
    let ctx = ctx_or_error(&state).map_err(|e| ApplicationError::Engine(EngineError::Render(e)))?;
    message_editing::retry(state.application_service.game_service(), ctx)?;
    Ok(ok("<span class=\"status ready\">Retrying...</span>"))
}
