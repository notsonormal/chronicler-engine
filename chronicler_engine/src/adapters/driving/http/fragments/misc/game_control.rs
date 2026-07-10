//! [DOC: docs/system/dashboard.md]
//! Game control fragment handlers

use axum::{body::Body, extract::State};

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::fragments::renderers::{internal_error, ok_refresh};
use crate::application::application_service::ApplicationError;

pub async fn reset_handler(
    State(state): State<AppState>,
) -> Result<axum::response::Response<Body>, ApplicationError> {
    match state.application_service.reset() {
        Ok(()) => Ok(ok_refresh()),
        Err(e) => Ok(internal_error(e.to_string())),
    }
}
