//! [DOC: docs/adr/adr-027-hexagonal-architecture-migration.md]
//! HTTP driving adapter — maps application `ApplicationError` to axum `Response`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::application::application_service::ApplicationError;

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn error_div(message: &str) -> String {
    format!(
        "<div class=\"error-message\">Error: {}</div>",
        html_escape(message)
    )
}

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

        (status, error_div(&message)).into_response()
    }
}
