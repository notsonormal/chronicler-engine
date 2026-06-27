//! [DOC: docs/system/dashboard.md]
//! HTTP response helpers

use axum::{body::Body, http::StatusCode, response::Response};

use crate::application::application_service::ApplicationError;
use crate::server::AppState;
use super::fragment_renderers::render_error;

#[allow(clippy::expect_used)]
pub fn ok(body: impl Into<String>) -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(body.into()))
        .expect("static response body is valid")
}

#[allow(clippy::expect_used)]
pub fn ok_refresh() -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header("HX-Refresh", "true")
        .body(Body::empty())
        .expect("static response body is valid")
}

#[allow(clippy::expect_used)]
pub fn bad_request(body: impl Into<String>) -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(body.into()))
        .expect("static response body is valid")
}

#[allow(clippy::expect_used)]
pub fn internal_error(body: impl Into<String>) -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(body.into()))
        .expect("static response body is valid")
}

pub fn ctx_or_error(
    state: &AppState,
) -> std::result::Result<crate::application::GameServiceContext, Box<Response<Body>>> {
    match state.as_game_service_context() {
        Ok(ctx) => Ok(ctx),
        Err(e) => Err(Box::new(internal_error(e.to_string()))),
    }
}

#[allow(clippy::expect_used)]
pub fn service_unavailable(body: impl Into<String>) -> Response<Body> {
    Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .body(Body::from(body.into()))
        .expect("static response body is valid")
}

pub fn service_unavailable_generating() -> Response<Body> {
    service_unavailable("<span class=\"status wait\">Generation in progress, please wait...</span>")
}

pub fn app_err_to_response(err: ApplicationError) -> Response<Body> {
    match err {
        ApplicationError::Validation(msg) => bad_request(render_error(&msg)),
        ApplicationError::ConcurrentGeneration => service_unavailable_generating(),
        ApplicationError::ShuttingDown => {
            service_unavailable(render_error("Server is shutting down"))
        }
        ApplicationError::Engine(e) => internal_error(render_error(&e.to_string())),
    }
}

pub fn app_err_to_tuple(err: ApplicationError) -> (StatusCode, String) {
    match err {
        ApplicationError::Validation(msg) => (StatusCode::BAD_REQUEST, render_error(&msg)),
        ApplicationError::ConcurrentGeneration => (
            StatusCode::SERVICE_UNAVAILABLE,
            "<span class=\"status wait\">Generation in progress, please wait...</span>".to_string(),
        ),
        ApplicationError::ShuttingDown => (
            StatusCode::SERVICE_UNAVAILABLE,
            render_error("Server is shutting down"),
        ),
        ApplicationError::Engine(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            render_error(&e.to_string()),
        ),
    }
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
