//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! HTTP response helpers

use axum::{body::Body, http::StatusCode, response::Response};

/// The header that names why a `409` refused a request. The client finds a refusal by
/// its status and reads this header only to choose the words of its report.
pub const REFUSAL_HEADER: &str = "x-refusal";

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

/// A request the engine refused and did nothing for. `kind` is the value of
/// `REFUSAL_HEADER`.
pub fn refused(kind: &'static str, body: impl Into<String>) -> Response<Body> {
    let mut response = status_response(StatusCode::CONFLICT, body);
    response
        .headers_mut()
        .insert(REFUSAL_HEADER, axum::http::HeaderValue::from_static(kind));
    response
}

pub fn not_found(body: impl Into<String>) -> Response<Body> {
    status_response(StatusCode::NOT_FOUND, body)
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
