//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Fragment-rendering glue: the template render helpers.

use askama::Template;
use axum::body::Body;
use axum::response::Response;

use crate::adapters::driving::http::utils::error::{failed_answer, error_fragment};
use crate::adapters::driving::http::utils::response::ok;

pub fn render_or_error_fragment(template: &impl Template) -> String {
    template.render().unwrap_or_else(|e| {
        tracing::error!("template render failed: {e}");
        error_fragment(format!("Template error: {e}"))
    })
}

pub fn render_failure_response(error: &askama::Error) -> Response<Body> {
    tracing::error!("template render failed: {error}");
    failed_answer(format!("Template error: {error}"))
}

pub fn render_response(template: &impl Template) -> Response<Body> {
    match template.render() {
        Ok(html) => ok(html),
        Err(e) => render_failure_response(&e),
    }
}
