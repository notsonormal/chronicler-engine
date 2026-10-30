//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! The two route layers of the request answers: the revision stamp, and the Game header check.

use axum::extract::{Request, State};
use axum::http::{HeaderValue, Method};
use axum::middleware::Next;
use axum::response::Response;

use super::answer_words::RefusalKind;
use super::response::refused;
use crate::adapters::driving::http::AppState;

pub const REVISION_HEADER: &str = "x-revision";
const REQUEST_GAME_HEADER: &str = "x-chronicler-game";

/// Puts `X-Revision: <server>:<revision>` on the answer of every `POST` route, a
/// success and a refusal alike. The stamp is read after the handler has ended, so it
/// holds at least the revision of the handler's own commit.
pub async fn stamp_request_answers(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let is_post = request.method() == Method::POST;
    let mut response = next.run(request).await;
    if is_post {
        let stamp = state.game_view_query.change_versions(None).stamp();
        if let Ok(value) = HeaderValue::from_str(&stamp) {
            response.headers_mut().insert(REVISION_HEADER, value);
        }
    }
    response
}

/// Refuses a request whose `X-Chronicler-Game` header names another Game than the
/// active Game, before the handler runs. A layer, not part of `CurrentGame`: axum
/// turns a rejected `Option<CurrentGame>` into `None`, so such a handler would act on
/// no Game and still answer `2xx`. A request with no header acts on the active Game.
pub async fn refuse_stale_game(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if let Some(named) = request.headers().get(REQUEST_GAME_HEADER) {
        let active = state.game_catalogue.active_game_id();
        let names_active =
            active.is_some_and(|game| named.to_str().is_ok_and(|named| named == game.to_string()));
        if !names_active {
            return refused(RefusalKind::GameChanged);
        }
    }
    next.run(request).await
}
