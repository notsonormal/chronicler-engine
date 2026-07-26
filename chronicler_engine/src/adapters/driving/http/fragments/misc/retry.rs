//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Retry fragment handler

use axum::{body::Body, extract::State, response::Response};

use crate::application::application_service::ApplicationError;
use crate::application::retry;
use crate::adapters::driving::http::AppState;

use crate::adapters::driving::http::utils::response::ok;

pub async fn retry_handler(
    State(state): State<AppState>,
) -> Result<Response<Body>, ApplicationError> {
    retry(state.application_service.clone())?;
    Ok(ok("<span class=\"status ready\">Retrying...</span>"))
}
