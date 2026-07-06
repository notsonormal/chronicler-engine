//! [DOC: docs/system/dashboard.md]
//! HTTP response helpers

use axum::{body::Body, http::StatusCode, response::Response};

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::op_context_loader::load_op_context_for_active_game;
use super::fragment_renderers::render_error;

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

pub fn ctx_or_error(
    state: &AppState,
) -> std::result::Result<crate::application::OpContext, String> {
    match load_op_context_for_active_game(state) {
        Ok(ctx) => Ok(ctx),
        Err(e) => Err(e.to_string()),
    }
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
