//! [DOC: docs/system/dashboard.md]
//! Retriggers fragment handler

use axum::{body::Body, extract::State, response::Response};

use crate::application::application_service::ApplicationError;
use crate::application::message_editing;
use crate::adapters::driving::http::AppState;

use crate::adapters::driving::http::fragments::renderers::ok;

pub async fn retrigger_handler(
    State(state): State<AppState>,
) -> Result<Response<Body>, ApplicationError> {
    message_editing::retrigger(state.application_service.clone())?;
    Ok(ok("<span class=\"status ready\">Retriggering...</span>"))
}
