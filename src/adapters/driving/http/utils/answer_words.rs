//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! The words of a failed answer: one table of routes, and the layer that writes the first line of every failed answer.

use axum::body::{Body, to_bytes};
use axum::extract::{MatchedPath, Request};
use axum::http::{StatusCode, header};
use axum::middleware::Next;
use axum::response::Response;

use super::error::error_fragment;
use crate::application::generation::gate::NotAccepted;

const UNBUILT_TEXT_LIMIT_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefusalKind {
    /// A Job or an Autoplay run holds the Game.
    Busy,
    /// The page named a Game that is no longer active.
    GameChanged,
    /// The page named a Message that is no longer the last one.
    LastMessageChanged,
}

impl RefusalKind {
    fn reason(self) -> &'static str {
        match self {
            Self::Busy => "The Game is busy",
            Self::GameChanged => "The active Game changed",
            Self::LastMessageChanged => "The last Message changed",
        }
    }

    /// Written as the second line of the body.
    fn engine_line(self) -> String {
        match self {
            Self::Busy => NotAccepted::StillThinking.line(),
            Self::GameChanged => "The page showed a Game that is no longer active.".to_string(),
            Self::LastMessageChanged => NotAccepted::LastMessageChanged.line(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnswerOutcome {
    /// `409`: the engine did nothing for the request.
    Refused(RefusalKind),
    /// `400`, or `404` for a dialog opener: the request named a value or a record the engine cannot use.
    Invalid,
    /// `500`: the engine failed.
    Failed,
}

/// Carried as a response extension; `describe_answers` writes the body from it.
#[derive(Clone, Debug)]
pub struct ErrorAnswer {
    outcome: AnswerOutcome,
    message: Option<String>,
    detail: Option<String>,
    item: Option<String>,
}

impl ErrorAnswer {
    /// The message names what to fix, so it joins the first line.
    pub fn invalid(message: impl std::fmt::Display) -> Self {
        Self {
            outcome: AnswerOutcome::Invalid,
            message: Some(message.to_string()),
            detail: None,
            item: None,
        }
    }

    /// Hidden behind Details, as the second line of the body.
    pub fn failed(detail: impl std::fmt::Display) -> Self {
        Self {
            outcome: AnswerOutcome::Failed,
            message: None,
            detail: Some(detail.to_string()),
            item: None,
        }
    }

    /// The sentence then reads correctly away from the control that failed.
    pub fn with_item(mut self, item: Option<String>) -> Self {
        self.item = item;
        self
    }

    pub fn refused(kind: RefusalKind) -> Self {
        Self {
            outcome: AnswerOutcome::Refused(kind),
            message: None,
            detail: Some(kind.engine_line()),
            item: None,
        }
    }

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

    fn fallback_body(&self) -> String {
        match (&self.outcome, &self.message, &self.detail) {
            (AnswerOutcome::Refused(_), _, _) => String::new(),
            (_, Some(message), _) => error_fragment(message),
            (_, None, Some(detail)) => error_fragment(detail),
            (_, None, None) => String::new(),
        }
    }
}

pub fn answer_response(status: StatusCode, answer: ErrorAnswer) -> Response<Body> {
    let mut response = Response::new(Body::from(answer.fallback_body()));
    *response.status_mut() = status;
    response.extensions_mut().insert(answer);
    response
}

pub fn item_named(name: &str) -> Option<String> {
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_string())
}

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

/// Each `post` route of the router, and each `GET` route that a dialog opener names.
pub const ANSWER_WORDS: &[RouteWords] = &[
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
    ANSWER_WORDS.iter().find(|row| row.route == route)
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

pub async fn describe_answers(request: Request, next: Next) -> Response {
    let words = request
        .extensions()
        .get::<MatchedPath>()
        .and_then(|matched| words_of_route(matched.as_str()));
    let response = next.run(request).await;
    let status = response.status();
    if !(status.is_client_error() || status.is_server_error()) {
        return response;
    }
    let built = response.extensions().get::<ErrorAnswer>().cloned();
    let Some(words) = words else {
        return response;
    };
    let (mut parts, body) = response.into_parts();
    let answer = match built {
        Some(answer) => answer,
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
