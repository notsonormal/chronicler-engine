//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Error rendering helpers for HTTP fragments.

use axum::body::Body;
use axum::response::Response;

use crate::adapters::driving::http::utils::response::{
    bad_request, html_escape, internal_error, not_found, refused,
};
use crate::application::errors::ApplicationError;
use crate::application::generation::gate::Refusal;
use crate::error::EngineError;

/// The body of every error that the server renders: a failed action, a failed form
/// answer and a failed screen load.
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

/// A non-2xx so htmx never swaps the element the failure describes.
pub(crate) fn action_refusal_response(message: impl std::fmt::Display) -> Response<Body> {
    bad_request(error_fragment(message))
}

/// A 404 for a dialog that names a record the engine does not hold; the opener
/// reports it in its own slot.
pub(crate) fn action_not_found_response(message: impl std::fmt::Display) -> Response<Body> {
    not_found(error_fragment(message))
}

/// A 500 for an error the request cannot correct; htmx still never swaps the
/// element the failure describes.
pub(crate) fn action_failure_response(message: impl std::fmt::Display) -> Response<Body> {
    internal_error(error_fragment(message))
}

/// How `answer_for_error` words and classifies an engine error.
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

/// The one classifier of an application error: a `400` for an invalid value or a
/// change the record's state does not allow, a `409` for a busy Game, and a `500`
/// for a failure.
pub(crate) fn answer_for_error(error: ApplicationError, policy: AnswerPolicy) -> Response<Body> {
    match error {
        ApplicationError::Validation(message) => bad_request(error_fragment(message)),
        ApplicationError::Refused(Refusal::StillThinking) => {
            refused("busy", error_fragment(Refusal::StillThinking.line()))
        }
        ApplicationError::Refused(Refusal::BadRequest(message)) => {
            bad_request(error_fragment(message))
        }
        ApplicationError::Refused(Refusal::Failed(failure)) => {
            internal_error(error_fragment(failure.line()))
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
                bad_request(error_fragment(text))
            } else {
                internal_error(error_fragment(text))
            }
        }
    }
}
