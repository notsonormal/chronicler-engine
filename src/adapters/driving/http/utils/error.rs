//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Error rendering helpers for HTTP fragments.

use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;

use crate::adapters::driving::http::utils::answer_words::{ErrorAnswer, RefusalKind, answer_response};
use crate::adapters::driving::http::utils::response::{html_escape, refused};
use crate::application::errors::ApplicationError;
use crate::application::generation::gate::Refusal;
use crate::error::EngineError;

pub fn error_fragment(message: impl std::fmt::Display) -> String {
    format!(
        "<div class=\"error-message\">{}</div>",
        html_escape(&message.to_string())
    )
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

/// The status is non-2xx, so htmx never swaps the element the failure describes.
pub fn action_invalid_response(message: impl std::fmt::Display) -> Response<Body> {
    answer_response(StatusCode::BAD_REQUEST, ErrorAnswer::invalid(message))
}

/// Counts as an invalid answer, as a `400` does.
pub fn action_not_found_response(message: impl std::fmt::Display) -> Response<Body> {
    answer_response(StatusCode::NOT_FOUND, ErrorAnswer::invalid(message))
}

pub fn action_failure_response(detail: impl std::fmt::Display) -> Response<Body> {
    answer_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorAnswer::failed(detail),
    )
}

#[derive(Clone, Copy)]
pub(crate) enum AnswerPolicy {
    /// The full text of the engine error, such as "Database error: ...".
    EngineText,
    /// A settings save: `EngineError::Validation` and `EngineError::Config` are
    /// invalid values (`400`), and the text of every error is the short reason.
    SettingsSave,
    /// The generic text of an error that no handler classified.
    GenericText,
}

const GENERIC_FAILURE_TEXT: &str = "Internal Server Error";

fn short_reason(error: &EngineError) -> String {
    match error {
        EngineError::Validation(message) | EngineError::Config(message) => message.clone(),
        EngineError::Database { context, source } => format!("{context}: {source}"),
        other => other.to_string(),
    }
}

pub(crate) fn answer_for_error(error: ApplicationError, policy: AnswerPolicy) -> Response<Body> {
    match error {
        ApplicationError::Validation(message) => action_invalid_response(message),
        ApplicationError::Refused(Refusal::StillThinking) => refused(RefusalKind::Busy),
        ApplicationError::Refused(Refusal::LastMessageChanged) => {
            refused(RefusalKind::LastMessageChanged)
        }
        ApplicationError::Refused(Refusal::BadRequest(message)) => action_invalid_response(message),
        ApplicationError::Refused(Refusal::Failed(failure)) => {
            action_failure_response(failure.line())
        }
        ApplicationError::Engine(engine) => {
            let invalid = match &engine {
                EngineError::WorldHasGames { .. } => true,
                EngineError::Validation(_) | EngineError::Config(_) => {
                    matches!(policy, AnswerPolicy::SettingsSave)
                }
                _ => false,
            };
            let text = match policy {
                AnswerPolicy::EngineText => engine.to_string(),
                AnswerPolicy::SettingsSave => short_reason(&engine),
                AnswerPolicy::GenericText if invalid => engine.to_string(),
                AnswerPolicy::GenericText => GENERIC_FAILURE_TEXT.to_string(),
            };
            if invalid {
                action_invalid_response(text)
            } else {
                action_failure_response(text)
            }
        }
    }
}
