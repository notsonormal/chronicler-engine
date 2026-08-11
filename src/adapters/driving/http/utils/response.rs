//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! HTTP response helpers

use axum::{body::Body, http::StatusCode, response::Response};

use crate::adapters::driving::http::utils::error::render_error;

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

#[allow(clippy::expect_used)]
pub fn ok_refresh() -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header("HX-Refresh", "true")
        .body(Body::empty())
        .expect("static response body is valid")
}

pub fn bad_request(body: impl Into<String>) -> Response<Body> {
    status_response(StatusCode::BAD_REQUEST, body)
}

pub fn internal_error(body: impl Into<String>) -> Response<Body> {
    status_response(StatusCode::INTERNAL_SERVER_ERROR, body)
}

pub fn service_unavailable(body: impl Into<String>) -> Response<Body> {
    status_response(StatusCode::SERVICE_UNAVAILABLE, body)
}

pub fn service_unavailable_generating() -> Response<Body> {
    service_unavailable(render_error("Generation in progress, please wait..."))
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
