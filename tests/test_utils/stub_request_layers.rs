//! What the engine's two request layers do, for the stub server: record the Game header of each `POST`, refuse a Game-bound one that names another Game, and stamp the answer with `X-Revision`.

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderValue, Method};
use axum::middleware::Next;
use axum::response::Response;

use chronicler_engine::adapters::driving::http::utils::answer_words::RefusalKind;
use chronicler_engine::adapters::driving::http::utils::response::refused;

/// The routes of the engine whose handler acts on the active Game, as the router
/// writes them. A test compares this list with `game_bound_routes` in `router.rs`.
pub const GAME_BOUND_ROUTES: [&str; 11] = [
    "/action",
    "/action/check",
    "/action/confirm",
    "/autoplay",
    "/swipe/new",
    "/retrigger",
    "/reset",
    "/message/:id/swipe/:index",
    "/history/:id",
    "/history/delete",
    "/status/reset-busy",
];

/// A path matches a route when each segment is equal or the route's segment is a
/// parameter. A literal route wins over a parameter route in the router, so
/// `/history/delete` is checked as a literal first by being in the list too.
fn is_game_bound(path: &str) -> bool {
    let segments: Vec<&str> = path.split('/').collect();
    GAME_BOUND_ROUTES.iter().any(|route| {
        let pattern: Vec<&str> = route.split('/').collect();
        pattern.len() == segments.len()
            && pattern
                .iter()
                .zip(&segments)
                .all(|(wanted, actual)| wanted.starts_with(':') || wanted == actual)
    })
}

#[derive(Default)]
pub struct StubRequestLayers {
    /// The `X-Chronicler-Game` header of each `POST`, by path, in arrival order.
    headers: std::sync::Mutex<Vec<(String, Option<String>)>>,
    /// The Game that the Game-bound routes treat as active. `None` means the Game of
    /// `GET /changes`.
    game_of_requests: std::sync::Mutex<Option<u64>>,
}

impl StubRequestLayers {
    pub fn game_headers(&self, path: &str) -> Vec<Option<String>> {
        self.headers
            .lock()
            .expect("stub game headers lock poisoned")
            .iter()
            .filter(|(sent_to, _)| sent_to == path)
            .map(|(_, header)| header.clone())
            .collect()
    }

    pub fn set_game_of_requests(&self, game: u64) {
        *self
            .game_of_requests
            .lock()
            .expect("stub request game lock poisoned") = Some(game);
    }

    /// `read_changes` gives the stamp and the active Game of the stub's `GET /changes`
    /// state. The layer reads it before the handler for the Game check and after the
    /// handler for the stamp, as the engine's layers do.
    pub async fn answer(
        &self,
        request: Request,
        next: Next,
        read_changes: impl Fn() -> (String, Option<u64>),
    ) -> Response<Body> {
        if request.method() != Method::POST {
            return next.run(request).await;
        }
        let path = request.uri().path().to_string();
        let named = request
            .headers()
            .get("x-chronicler-game")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        self.headers
            .lock()
            .expect("stub game headers lock poisoned")
            .push((path.clone(), named.clone()));
        let active = self
            .game_of_requests
            .lock()
            .expect("stub request game lock poisoned")
            .or(read_changes().1);
        let stale = named.is_some_and(|named| Some(named) != active.map(|game| game.to_string()));
        let mut response = if stale && is_game_bound(&path) {
            refused(RefusalKind::GameChanged)
        } else {
            next.run(request).await
        };
        if let Ok(value) = HeaderValue::from_str(&read_changes().0) {
            response.headers_mut().insert("x-revision", value);
        }
        response
    }
}
