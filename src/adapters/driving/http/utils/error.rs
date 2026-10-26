//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Error rendering helpers for HTTP fragments.

use axum::body::Body;
use axum::response::Response;

use crate::adapters::driving::http::utils::response::{
    bad_request, html_escape, internal_error, not_found,
};
use crate::application::errors::ApplicationError;
use crate::error::EngineError;

pub(crate) fn error_fragment(message: impl std::fmt::Display) -> String {
    format!(
        "<div class=\"error-message\">{}</div>",
        html_escape(&message.to_string())
    )
}

pub fn render_error(message: &str) -> String {
    error_fragment(format!("Error: {message}"))
}

/// The popover's open state is view state: `hidden` and the position `style` stay
/// on the node through a load (`data-keep`), and so does `aria-expanded` on the toggle.
///
/// The short line needs its own block box for the status column's two-line clamp;
/// the message itself is the disclosure's flex item.
pub fn error_disclosure(popover_id: &str, message: &str, detail_html: &str) -> String {
    format!(
        "<div class=\"error-disclosure\"><span class=\"error-disclosure-message\"><span class=\"error-disclosure-text\">{message}</span></span><button type=\"button\" class=\"error-details-toggle\" aria-expanded=\"false\" aria-controls=\"{popover_id}\" data-keep=\"aria-expanded\">Details</button><div class=\"error-detail-popover\" id=\"{popover_id}\" hidden data-keep=\"hidden style\">{detail_html}</div></div>",
        message = html_escape(message),
        popover_id = html_escape(popover_id),
        detail_html = detail_html,
    )
}

pub fn raw_error_detail(raw: &str) -> String {
    format!("<pre class=\"error-detail-raw\">{}</pre>", html_escape(raw))
}

fn classify_error(error: ApplicationError, prefix: &str) -> Result<String, String> {
    match error {
        ApplicationError::Validation(message) => Ok(message),
        other => Err(format!("{prefix}: {other}")),
    }
}

/// A non-2xx so htmx never swaps the element the failure describes.
pub(crate) fn action_refusal_response(message: impl std::fmt::Display) -> Response<Body> {
    bad_request(error_fragment(message))
}

/// A 404 for a dialog that names a record the engine does not hold; the opener
/// reports it in its own slot.
pub(crate) fn action_not_found_response(message: impl std::fmt::Display) -> Response<Body> {
    not_found(error_fragment(message))
}

pub(crate) fn refusal_message_without_layer_prefix_for_validation_and_config(
    error: &EngineError,
) -> String {
    match error {
        EngineError::Validation(message) | EngineError::Config(message) => message.clone(),
        EngineError::Database { context, source } => format!("{context}: {source}"),
        other => other.to_string(),
    }
}

/// A 500 for an error the request cannot correct; htmx still never swaps the
/// element the failure describes.
pub(crate) fn action_failure_response(message: impl std::fmt::Display) -> Response<Body> {
    internal_error(error_fragment(message))
}

pub(crate) fn action_error_response(error: ApplicationError, prefix: &str) -> Response<Body> {
    match classify_error(error, prefix) {
        Ok(message) => action_refusal_response(message),
        Err(message) => action_failure_response(message),
    }
}
