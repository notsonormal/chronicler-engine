//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Fragment-rendering glue: the try-render wrapper for AppState renderers and the template render helpers.

use askama::Template;
use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::utils::error::error_fragment;
use crate::adapters::driving::http::utils::response::ok;

pub fn render_fragment<F>(state: &AppState, render: F, name: &str) -> Response<Body>
where
    F: FnOnce(&AppState) -> crate::error::Result<String>,
{
    match render(state) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("{name} failed: {e}");
            failed_fragment_response(&e.to_string())
        }
    }
}

pub(crate) fn failed_fragment_response(message: &str) -> Response<Body> {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html(error_fragment(message)),
    )
        .into_response()
}

pub fn render_or_error_fragment(template: &impl Template) -> String {
    template.render().unwrap_or_else(|e| {
        tracing::error!("template render failed: {e}");
        error_fragment(format!("Template error: {e}"))
    })
}

pub fn render_failure_response(error: &askama::Error) -> Response<Body> {
    tracing::error!("template render failed: {error}");
    failed_fragment_response(&format!("Template error: {error}"))
}

pub fn render_response(template: &impl Template) -> Response<Body> {
    match template.render() {
        Ok(html) => ok(html),
        Err(e) => render_failure_response(&e),
    }
}
