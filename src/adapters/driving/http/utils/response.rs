//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! HTTP response helpers

use axum::{body::Body, http::StatusCode, response::Response};

/// The answer of a write that the busy Game refuses. `dashboard-errors.js` keys the
/// refusal on the `status wait` class in this markup and reports it in the slot of
/// the refused action; the markup itself is never shown. The stub server answers a
/// refused Action with it.
pub const BUSY_STATUS: &str = "<span class=\"status wait\">Still thinking...</span>";

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
