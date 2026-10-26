//! [DOC: docs/diataxis/reference/architecture_system.md]
//! HTTP driving adapter — maps application `ApplicationError` to axum `Response`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use super::utils::error::render_error;
use super::utils::response::refused;

use crate::application::errors::ApplicationError;
use crate::application::generation::gate::Refusal;

impl IntoResponse for ApplicationError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApplicationError::Refused(refusal) => return refusal.clone().into_response(),
            ApplicationError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
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

impl IntoResponse for Refusal {
    fn into_response(self) -> Response {
        match self {
            Refusal::StillThinking => {
                refused("busy", render_error(&Refusal::StillThinking.line())).into_response()
            }
            Refusal::BadRequest(message) => {
                (StatusCode::BAD_REQUEST, render_error(&message)).into_response()
            }
            Refusal::Failed(failure) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                render_error(&failure.line()),
            )
                .into_response(),
        }
    }
}
