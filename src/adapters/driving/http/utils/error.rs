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
