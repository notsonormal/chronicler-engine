//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Error rendering helpers for HTTP fragments.

use axum::body::Body;
use axum::response::{Html, IntoResponse, Response};

use crate::adapters::driving::http::utils::response::{bad_request, html_escape, internal_error};
use crate::application::errors::ApplicationError;
use crate::domain::model::state::generation_status::GenerationFailureKind;
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

/// The short line needs its own block box for the status column's two-line clamp;
/// the message itself is the disclosure's flex item.
pub fn error_disclosure(popover_id: &str, message: &str, detail_html: &str) -> String {
    format!(
        "<div class=\"error-disclosure\"><span class=\"error-disclosure-message\"><span class=\"error-disclosure-text\">{message}</span></span><button type=\"button\" class=\"error-details-toggle\" aria-expanded=\"false\" aria-controls=\"{popover_id}\">Details</button><div class=\"error-detail-popover\" id=\"{popover_id}\" hidden>{detail_html}</div></div>",
        message = html_escape(message),
        popover_id = html_escape(popover_id),
        detail_html = detail_html,
    )
}

pub fn raw_error_detail(raw: &str) -> String {
    format!("<pre class=\"error-detail-raw\">{}</pre>", html_escape(raw))
}

/// The 240px status column clamps this sentence to two lines, so any remedy it
/// names has to be one the dashboard can carry out.
pub fn generation_failure_summary(kind: GenerationFailureKind) -> &'static str {
    match kind {
        GenerationFailureKind::Unreachable => "The language model could not be reached.",
        GenerationFailureKind::PromptTooLong => {
            "The last turn was too long for the language model."
        }
        GenerationFailureKind::UnreadableAnswer => "The language model's answer could not be read.",
        // The dashboard offers no Max Tokens control, so the one remedy a player
        // can act on is a different model on the role's connection.
        GenerationFailureKind::TokenBudgetSpent => {
            "The model used its budget on reasoning. Try another model."
        }
        GenerationFailureKind::SaveFailed => "The last turn could not be saved.",
        GenerationFailureKind::SceneMissing => "The current scene could not be found.",
        GenerationFailureKind::PresetMissing => "The active prompt preset is missing.",
        GenerationFailureKind::Other => "The last turn failed to generate.",
    }
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

/// A non-2xx so htmx never swaps the region the failure describes.
pub(crate) fn action_refusal_response(message: impl std::fmt::Display) -> Response<Body> {
    bad_request(error_fragment(message))
}

pub(crate) fn refusal_message_without_layer_prefix_for_validation_and_config(
    error: &EngineError,
) -> String {
    match error {
        EngineError::Validation(message) | EngineError::Config(message) => message.clone(),
        other => other.to_string(),
    }
}

/// A non-2xx so htmx never swaps the region the failure describes.
pub(crate) fn action_failure_response(message: impl std::fmt::Display) -> Response<Body> {
    internal_error(error_fragment(message))
}

pub(crate) fn action_error_response(error: ApplicationError, prefix: &str) -> Response<Body> {
    match classify_error(error, prefix) {
        Ok(message) => action_refusal_response(message),
        Err(message) => action_failure_response(message),
    }
}
