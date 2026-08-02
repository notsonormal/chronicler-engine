//! [DOC: chronicler_engine/docs/diataxis/reference/architecture_system.md]
//! HTTP driving adapter — maps application `ApplicationError` to axum `Response`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use super::utils::error::render_error;

use crate::application::errors::ApplicationError;

impl IntoResponse for ApplicationError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApplicationError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApplicationError::ConcurrentGeneration => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Generation in progress, please wait...".to_string(),
            ),
            ApplicationError::ShuttingDown => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Server is shutting down".to_string(),
            ),
            ApplicationError::Engine(_) if self.is_user_displayable() => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            ApplicationError::Engine(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ),
        };

        (status, render_error(&message)).into_response()
    }
}
