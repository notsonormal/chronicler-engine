//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Error rendering helpers for HTTP fragments.

use axum::body::Body;
use axum::response::{Html, IntoResponse, Response};

use crate::adapters::driving::http::utils::response::{bad_request, html_escape, internal_error};
use crate::application::errors::ApplicationError;

pub(crate) fn error_fragment(message: impl std::fmt::Display) -> String {
    format!(
        "<div class=\"error-message\">{}</div>",
        html_escape(&message.to_string())
    )
}

pub fn render_error(message: &str) -> String {
    error_fragment(format!("Error: {message}"))
}

pub fn error_disclosure(popover_id: &str, message: &str, detail_html: &str) -> String {
    format!(
        "<div class=\"error-disclosure\"><span class=\"error-disclosure-message\">{message}</span><button type=\"button\" class=\"error-details-toggle\" aria-expanded=\"false\" aria-controls=\"{popover_id}\">Details</button><div class=\"error-detail-popover\" id=\"{popover_id}\" hidden>{detail_html}</div></div>",
        message = html_escape(message),
        popover_id = html_escape(popover_id),
        detail_html = detail_html,
    )
}

pub fn raw_error_detail(raw: &str) -> String {
    format!("<pre class=\"error-detail-raw\">{}</pre>", html_escape(raw))
}

pub fn generation_error_summary(raw: &str) -> String {
    let lower = raw.to_lowercase();
    if lower.contains("llm") || lower.contains("connection") || lower.contains("backend") {
        return "The language model could not be reached.".to_string();
    }
    if lower.contains("save") {
        return "The last turn could not be saved.".to_string();
    }
    if lower.contains("room") || lower.contains("scene") {
        return "The current scene could not be found.".to_string();
    }
    if lower.contains("preset") {
        return "The active prompt preset is missing.".to_string();
    }
    "The last turn failed to generate.".to_string()
}

pub(crate) fn error_fragment_response(message: impl std::fmt::Display) -> Response<Body> {
    Html(error_fragment(message)).into_response()
}

fn classify_error(error: ApplicationError, prefix: &str) -> Result<String, String> {
    match error {
        ApplicationError::Validation(message) => Ok(message),
        other => Err(format!("{prefix}: {other}")),
    }
}

/// A refusal reaches the user as a 400, which the shell renders into the
/// failing surface's inline error slot and leaves the panel in place. Other
/// errors keep the in-fragment rendering, labelled with `prefix` so the
/// failure's origin survives.
pub(crate) fn error_response(error: ApplicationError, prefix: &str) -> Response<Body> {
    match classify_error(error, prefix) {
        Ok(message) => bad_request(render_error(&message)),
        Err(message) => error_fragment_response(message),
    }
}

/// A refused or malformed form action: 400 with the failure for the client's
/// inline slot. A non-2xx so htmx never swaps the region the failure describes.
pub(crate) fn action_refusal_response(message: impl std::fmt::Display) -> Response<Body> {
    bad_request(error_fragment(message))
}

/// A server-side form action failure: 500 with the failure for the client's
/// inline slot. A non-2xx so htmx never swaps the region the failure describes.
pub(crate) fn action_failure_response(message: impl std::fmt::Display) -> Response<Body> {
    internal_error(error_fragment(message))
}

pub(crate) fn action_error_response(error: ApplicationError, prefix: &str) -> Response<Body> {
    match classify_error(error, prefix) {
        Ok(message) => action_refusal_response(message),
        Err(message) => action_failure_response(message),
    }
}
