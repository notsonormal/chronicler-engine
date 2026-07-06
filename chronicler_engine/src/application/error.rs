//! [DOC: docs/adr/adr-029-http-error-boundary.md]
//! Application-layer port for HTTP error mapping (adapter impl lives in `adapters/driving/http`).

use crate::application::application_service::ApplicationError;
use crate::error::EngineError;

pub enum HttpStatusCode {
    BadRequest,
    ServiceUnavailable,
    InternalServerError,
}

pub struct ErrorResponse {
    pub status: HttpStatusCode,
    pub body: String,
}

pub trait HttpError {
    fn status_code(&self) -> HttpStatusCode;
    fn error_body(&self) -> ErrorResponse;
}

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

impl HttpError for ApplicationError {
    fn status_code(&self) -> HttpStatusCode {
        match self {
            Self::Validation(_) => HttpStatusCode::BadRequest,
            Self::ConcurrentGeneration | Self::ShuttingDown => HttpStatusCode::ServiceUnavailable,
            Self::Engine(_) => HttpStatusCode::InternalServerError,
        }
    }

    fn error_body(&self) -> ErrorResponse {
        let body = match self {
            Self::Validation(msg) => error_div(msg),
            Self::ConcurrentGeneration => error_div("Generation in progress, please wait..."),
            Self::ShuttingDown => error_div("Server is shutting down"),
            Self::Engine(e) => error_div(&e.to_string()),
        };
        let status = match self {
            Self::Validation(_) => HttpStatusCode::BadRequest,
            Self::ConcurrentGeneration | Self::ShuttingDown => HttpStatusCode::ServiceUnavailable,
            Self::Engine(_) => HttpStatusCode::InternalServerError,
        };
        ErrorResponse { status, body }
    }
}

impl HttpError for EngineError {
    fn status_code(&self) -> HttpStatusCode {
        HttpStatusCode::InternalServerError
    }

    fn error_body(&self) -> ErrorResponse {
        ErrorResponse {
            status: HttpStatusCode::InternalServerError,
            body: error_div(&self.to_string()),
        }
    }
}
