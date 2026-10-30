//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! HTTP response helpers

use axum::{body::Body, http::StatusCode, response::Response};

use super::answer_words::{ErrorAnswer, RefusalKind, answer_response};

#[allow(clippy::expect_used)]
fn status_response(status: StatusCode, body: impl Into<String>) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(body.into()))
        .expect("static response body is valid")
}

pub fn ok(body: impl Into<String>) -> Response<Body> {
    status_response(StatusCode::OK, body)
}

pub fn bad_request(body: impl Into<String>) -> Response<Body> {
    status_response(StatusCode::BAD_REQUEST, body)
}

/// The body is empty: the words layer writes it from the kind and the route.
pub fn refused(kind: RefusalKind) -> Response<Body> {
    answer_response(StatusCode::CONFLICT, ErrorAnswer::refused(kind))
}

pub fn internal_error(body: impl Into<String>) -> Response<Body> {
    status_response(StatusCode::INTERNAL_SERVER_ERROR, body)
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
