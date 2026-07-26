//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Retriggers fragment handler

use axum::{body::Body, extract::State, response::Response};

use crate::application::application_service::ApplicationError;
use crate::application::retrigger;
use crate::adapters::driving::http::AppState;

use crate::adapters::driving::http::utils::response::ok;

pub async fn retrigger_handler(
    State(state): State<AppState>,
) -> Result<Response<Body>, ApplicationError> {
    retrigger(state.application_service.clone())?;
    Ok(ok("<span class=\"status ready\">Retriggering...</span>"))
}
