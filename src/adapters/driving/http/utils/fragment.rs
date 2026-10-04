//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Fragment-rendering glue: uniform try-render / log-error wrapper for AppState renderers.

use axum::body::Body;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};

use crate::adapters::driving::http::AppState;
use crate::adapters::driving::http::utils::error::render_error;

pub fn render_fragment<F>(state: &AppState, render: F, name: &str) -> Response<Body>
where
    F: FnOnce(&AppState) -> crate::error::Result<String>,
{
    match render(state) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("{name} failed: {e}");
            let mut response = (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_error(&e.to_string())),
            )
                .into_response();
            response
                .headers_mut()
                .insert("HX-Reswap", HeaderValue::from_static("none"));
            response
        }
    }
}
