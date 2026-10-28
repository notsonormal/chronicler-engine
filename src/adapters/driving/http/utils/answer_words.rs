//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! The words of an action answer: one table of routes, and the layer that writes the first line of every failed answer.

use axum::body::{Body, to_bytes};
use axum::extract::{MatchedPath, Request};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware::Next;
use axum::response::Response;

use super::error::error_fragment;
use super::response::REFUSAL_HEADER;
use crate::application::generation::gate::Refusal;

const UNBUILT_TEXT_LIMIT_BYTES: usize = 64 * 1024;

/// Why the engine refused an action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefusalKind {
    /// A Job or an Autoplay run holds the Game.
    Busy,
    /// The page named a Game that is no longer active.
    GameChanged,
}

impl RefusalKind {
    fn reason(self) -> &'static str {
        match self {
            Self::Busy => "The Game is busy",
            Self::GameChanged => "The active Game changed",
        }
    }

    /// The engine's own line of the refusal: the second line of the body.
    fn engine_line(self) -> String {
        match self {
            Self::Busy => Refusal::StillThinking.line(),
            Self::GameChanged => "The page showed a Game that is no longer active.".to_string(),
        }
    }

    fn header_value(self) -> &'static str {
        match self {
            Self::Busy => "busy",
            Self::GameChanged => "game-changed",
        }
    }
}

/// How an action answered, apart from a done answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnswerOutcome {
    /// `409`: the engine did nothing for the request.
    Refused(RefusalKind),
    /// `400`, or `404` for a dialog opener: the request named a value or a record the engine cannot use.
    Invalid,
    /// `500`: the engine failed.
    Failed,
}

/// What a handler knows of a failed answer. The response carries it as an extension,
/// and `describe_answers` writes the body from it and from the words of the route.
#[derive(Clone, Debug)]
pub struct ErrorAnswer {
    outcome: AnswerOutcome,
    message: Option<String>,
    detail: Option<String>,
    item: Option<String>,
}

impl ErrorAnswer {
    /// The engine's message says what to fix, so it joins the first line.
    pub fn invalid(message: impl std::fmt::Display) -> Self {
        Self {
            outcome: AnswerOutcome::Invalid,
            message: Some(message.to_string()),
            detail: None,
            item: None,
        }
    }

    /// The engine's text stays behind Details, as the second line.
    pub fn failed(detail: impl std::fmt::Display) -> Self {
        Self {
            outcome: AnswerOutcome::Failed,
            message: None,
            detail: Some(detail.to_string()),
            item: None,
        }
    }

    pub fn refused(kind: RefusalKind) -> Self {
        Self {
            outcome: AnswerOutcome::Refused(kind),
            message: None,
            detail: Some(kind.engine_line()),
            item: None,
        }
    }

    /// An answer that no constructor built: the text it carried becomes the second line.
    fn unbuilt(status: StatusCode, text: &str) -> Self {
        let outcome = if status.is_server_error() {
            AnswerOutcome::Failed
        } else {
            AnswerOutcome::Invalid
        };
        let text = text.trim();
        Self {
            outcome,
            message: None,
            detail: (!text.is_empty()).then(|| text.to_string()),
            item: None,
        }
    }

    /// The body that a route with no row keeps.
    fn fallback_body(&self) -> String {
        match (&self.outcome, &self.message, &self.detail) {
            (AnswerOutcome::Refused(_), _, _) => String::new(),
            (_, Some(message), _) => error_fragment(message),
            (_, None, Some(detail)) => error_fragment(detail),
            (_, None, None) => String::new(),
        }
    }
}

/// The response of a failed action, with the status and the answer that the layer reads.
pub fn answer_response(status: StatusCode, answer: ErrorAnswer) -> Response<Body> {
    let mut response = Response::new(Body::from(answer.fallback_body()));
    *response.status_mut() = status;
    response.extensions_mut().insert(answer);
    response
}

/// The item of an answer, from the name that a handler holds. A blank name names nothing.
pub fn item_named(name: &str) -> Option<String> {
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_string())
}

/// Names the item of a failed answer, so its sentence reads correctly away from the
/// control. A handler that could not read the name of its item names none.
pub fn naming_item(mut response: Response<Body>, item: Option<String>) -> Response<Body> {
    if let Some(answer) = response.extensions_mut().get_mut::<ErrorAnswer>() {
        answer.item = item;
    }
    response
}

/// One row of the words table: what was not done on one route.
pub struct RouteWords {
    pub route: &'static str,
    pub noun: &'static str,
    pub predicate: &'static str,
}

impl RouteWords {
    const fn new(route: &'static str, noun: &'static str, predicate: &'static str) -> Self {
        Self {
            route,
            noun,
            predicate,
        }
    }
}

/// The words of every action route: each `post` route of the router, and each `GET`
/// route that a dialog opener names. A new action adds its row in the same commit.
pub const REFUSAL_WORDS: &[RouteWords] = &[
    RouteWords::new("/action", "Action", "was not sent"),
    RouteWords::new("/action/check", "Action", "was not sent"),
    RouteWords::new("/action/confirm", "Action", "was not sent"),
    RouteWords::new("/autoplay", "Autoplay run", "did not start"),
    RouteWords::new("/swipe/new", "Retry", "was not started"),
    RouteWords::new("/retrigger", "retrigger", "was not started"),
    RouteWords::new("/reset", "Game", "was not reset"),
    RouteWords::new("/message/:id/swipe/:index", "Swipe", "was not switched"),
    RouteWords::new("/history/:id", "edit", "was not saved"),
    RouteWords::new("/history/delete", "Message", "was not deleted"),
    RouteWords::new("/status/reset-busy", "Autoplay run", "was not stopped"),
    RouteWords::new("/games", "Game", "was not started"),
    RouteWords::new("/games/:id/switch", "Game", "was not switched"),
    RouteWords::new("/games/:id/rename", "Game", "was not renamed"),
    RouteWords::new("/games/:id/delete", "Game", "was not deleted"),
    RouteWords::new("/games/:id/mode", "change", "was not saved"),
    RouteWords::new("/games/:id/posture", "change", "was not saved"),
    RouteWords::new("/games/:id/presets", "change", "was not saved"),
    RouteWords::new("/worlds", "World", "was not created"),
    RouteWords::new("/worlds/:key", "World", "was not saved"),
    RouteWords::new("/worlds/:key/posture", "posture", "was not saved"),
    RouteWords::new("/worlds/:key/delete", "World", "was not deleted"),
    RouteWords::new("/connections/add", "Connection", "was not saved"),
    RouteWords::new("/connections/:id/edit", "Connection", "was not saved"),
    RouteWords::new("/connections/test", "Connection", "was not tested"),
    RouteWords::new("/connections/:id/test", "Connection", "was not tested"),
    RouteWords::new("/connections/:id/delete", "Connection", "was not deleted"),
    RouteWords::new("/connections/set-narrator", "Narrator", "was not changed"),
    RouteWords::new(
        "/connections/set-quantifier",
        "Quantifier",
        "was not changed",
    ),
    RouteWords::new(
        "/settings/text-check",
        "text-check setting",
        "was not saved",
    ),
    RouteWords::new("/prompt-presets", "preset", "was not saved"),
    RouteWords::new("/prompt-presets/:id", "preset", "was not saved"),
    RouteWords::new("/prompt-presets/:id/delete", "preset", "was not deleted"),
    RouteWords::new(
        "/prompt-presets/:id/duplicate",
        "preset",
        "was not duplicated",
    ),
    RouteWords::new(
        "/prompt-presets/:id/activate",
        "preset",
        "was not activated",
    ),
    RouteWords::new("/fragment/games/new", "form", "was not opened"),
    RouteWords::new("/fragment/games/:id/rename", "form", "was not opened"),
    RouteWords::new("/fragment/worlds/new", "form", "was not opened"),
    RouteWords::new("/worlds/:key/edit", "form", "was not opened"),
    RouteWords::new("/fragment/connections/new", "form", "was not opened"),
    RouteWords::new("/fragment/connections/:id/edit", "form", "was not opened"),
    RouteWords::new("/fragment/prompt-presets/new", "form", "was not opened"),
    RouteWords::new(
        "/fragment/prompt-presets/:id/edit",
        "form",
        "was not opened",
    ),
    RouteWords::new(
        "/fragment/prompt-presets/:id/view",
        "form",
        "was not opened",
    ),
    RouteWords::new("/fragment/llm-messages/:id", "form", "was not opened"),
];

fn words_of_route(route: &str) -> Option<&'static RouteWords> {
    REFUSAL_WORDS.iter().find(|row| row.route == route)
}

fn capitalised(sentence: &str) -> String {
    let mut characters = sentence.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect(),
        None => String::new(),
    }
}

fn first_line(answer: &ErrorAnswer, words: &RouteWords) -> String {
    let named = match &answer.item {
        Some(item) => format!("{} \"{item}\"", words.noun),
        None => words.noun.to_string(),
    };
    let what = format!("the {named} {}", words.predicate);
    match answer.outcome {
        AnswerOutcome::Refused(kind) => format!("{}, so {what}.", kind.reason()),
        AnswerOutcome::Invalid => match &answer.message {
            Some(message) => {
                let message = message.trim();
                let stop = if message.ends_with('.') { "" } else { "." };
                format!("{}: {message}{stop}", capitalised(&what))
            }
            None => format!("{}.", capitalised(&what)),
        },
        AnswerOutcome::Failed => format!("{}.", capitalised(&what)),
    }
}

fn body_of(answer: &ErrorAnswer, words: &RouteWords) -> String {
    let first = first_line(answer, words);
    match &answer.detail {
        Some(detail) => error_fragment(format!("{first}\n{detail}")),
        None => error_fragment(first),
    }
}

/// The layer of the words: after routing, it writes the body of each answer of status 400
/// or higher on a route with a row, and sets `X-Refusal` on a refusal. An answer that no
/// constructor built keeps its text as the second line. A `409` that no constructor built
/// keeps its body, because the layer cannot know its reason.
pub async fn describe_answers(request: Request, next: Next) -> Response {
    let words = request
        .extensions()
        .get::<MatchedPath>()
        .and_then(|matched| words_of_route(matched.as_str()));
    let mut response = next.run(request).await;
    let status = response.status();
    if !(status.is_client_error() || status.is_server_error()) {
        return response;
    }
    let built = response.extensions().get::<ErrorAnswer>().cloned();
    if let Some(ErrorAnswer {
        outcome: AnswerOutcome::Refused(kind),
        ..
    }) = &built
    {
        response.headers_mut().insert(
            REFUSAL_HEADER,
            HeaderValue::from_static(kind.header_value()),
        );
    }
    let Some(words) = words else {
        return response;
    };
    let (mut parts, body) = response.into_parts();
    let answer = match built {
        Some(answer) => answer,
        None if status == StatusCode::CONFLICT => return Response::from_parts(parts, body),
        None => {
            let bytes = to_bytes(body, UNBUILT_TEXT_LIMIT_BYTES)
                .await
                .unwrap_or_default();
            ErrorAnswer::unbuilt(status, &String::from_utf8_lossy(&bytes))
        }
    };
    parts.headers.remove(header::CONTENT_LENGTH);
    Response::from_parts(parts, Body::from(body_of(&answer, words)))
}
