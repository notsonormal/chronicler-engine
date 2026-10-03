//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Error rendering helpers for HTTP fragments.

use axum::body::Body;
use axum::response::{Html, IntoResponse, Response};

use crate::adapters::driving::http::utils::response::{bad_request, html_escape};
use crate::application::errors::ApplicationError;

/// The one error fragment. Its class carries the styling, and the shell leaves
/// it in place; every handler failure and refusal renders through it.
pub(crate) fn error_fragment(message: impl std::fmt::Display) -> String {
    format!(
        "<div class=\"error-message\">{}</div>",
        html_escape(&message.to_string())
    )
}

/// The same fragment with the `Error: ` prefix the failure paths carry.
pub fn render_error(message: &str) -> String {
    error_fragment(format!("Error: {message}"))
}

/// A failure the panel keeps in place: a 200 carrying the error fragment.
pub(crate) fn error_fragment_response(message: impl std::fmt::Display) -> Response<Body> {
    Html(error_fragment(message)).into_response()
}

/// A refusal reaches the user as a 400, which the shell shows as a toast and
/// leaves the panel in place. Other errors keep the in-fragment rendering,
/// labelled with `prefix` so the failure's origin survives.
pub(crate) fn error_response(error: ApplicationError, prefix: &str) -> Response<Body> {
    match error {
        ApplicationError::Validation(message) => bad_request(render_error(&message)),
        other => error_fragment_response(format!("{prefix}: {other}")),
    }
}
