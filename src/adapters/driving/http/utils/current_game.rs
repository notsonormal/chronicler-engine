//! [DOC: docs/diataxis/reference/storage.md]
//! `CurrentGame` — the request's Game, read once per request.

use std::future::Future;
use std::pin::Pin;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::adapters::driving::http::AppState;
use crate::application::errors::ApplicationError;
use crate::domain::model::game::GameId;

/// The active Game of this request, read from the pointer once. A request
/// under no active Game is a `400`. The read goes through `GameCatalogue`, so
/// `ActiveGame::get` keeps exactly one caller.
pub struct CurrentGame(pub GameId);

// The manual future signature is `async_trait`'s desugaring for
// `FromRequestParts::from_request_parts`, which axum declares through that
// macro; an `async fn` impl does not match it.
impl FromRequestParts<AppState> for CurrentGame {
    type Rejection = ApplicationError;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        _parts: &'life0 mut Parts,
        state: &'life1 AppState,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        let game = state.game_catalogue.active_game_id();
        Box::pin(async move {
            game.map(CurrentGame)
                .ok_or_else(|| ApplicationError::validation("No active game"))
        })
    }
}
