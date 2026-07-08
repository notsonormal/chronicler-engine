//! [DOC: docs/system/dashboard.md]
//! Game control fragment handlers

use axum::{body::Body, extract::State};

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::fragments::renderers::{
    internal_error, ok_refresh, service_unavailable_generating,
};
use crate::application::application_service::ApplicationError;

pub async fn reset_handler(
    State(state): State<AppState>,
) -> Result<axum::response::Response<Body>, ApplicationError> {
    if state
        .is_generating
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return Ok(service_unavailable_generating());
    }

    state.current_cancel_token().cancel();

    match state.application_service.reset() {
        Ok(()) => {
            state
                .is_generating
                .store(false, std::sync::atomic::Ordering::SeqCst);
            state.replace_cancel_token();
            Ok(ok_refresh())
        }
        Err(e) => Ok(internal_error(e.to_string())),
    }
}
