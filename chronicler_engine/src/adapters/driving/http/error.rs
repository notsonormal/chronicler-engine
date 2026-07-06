//! [DOC: docs/adr/adr-029-http-error-boundary.md]
//! HTTP driving adapter — maps application `ApplicationError` to axum `Response`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::application::application_service::ApplicationError;
use crate::application::error::{ErrorResponse, HttpError, HttpStatusCode};

fn map_status(status: HttpStatusCode) -> StatusCode {
    match status {
        HttpStatusCode::BadRequest => StatusCode::BAD_REQUEST,
        HttpStatusCode::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        HttpStatusCode::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

impl IntoResponse for ApplicationError {
    fn into_response(self) -> Response {
        let ErrorResponse { status, body } = self.error_body();
        (map_status(status), body).into_response()
    }
}
