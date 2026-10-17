//! [DOC: docs/diataxis/reference/storage.md]
//! ActiveGame — the active-Game pointer of the server process.

use std::sync::{Arc, Mutex};

use crate::adapters::driven::storage::Storage;
use crate::domain::model::game::GameId;

/// The one active Game of this server process. `GameCatalogue` is its only
/// reader and its only writer: `CurrentGame` reads through the catalogue once
/// per request, and `set`'s visibility keeps the write side a compile-time
/// fact.
pub struct ActiveGame {
    storage: Arc<Storage>,
    game: Mutex<Option<GameId>>,
}

impl ActiveGame {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self {
            storage,
            game: Mutex::new(None),
        }
    }

    pub fn get(&self) -> Option<GameId> {
        *self.game.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The pointer touches the Game's `updated_at`, because the newest Game of a
    /// World is picked by that column.
    ///
    /// `pub(super)`: only `application/games/` may move the pointer, so a
    /// request cannot end up writing two Games. The composition root and the
    /// test builders point the app through `GameCatalogue::new`'s initial
    /// Game; every later move goes through `GameCatalogue::switch_game`.
    pub(super) fn set(&self, game: GameId) {
        let mut current = self.game.lock().unwrap_or_else(|e| e.into_inner());
        if *current == Some(game) {
            return;
        }
        if let Err(e) = self.storage.touch_game_updated_at(game) {
            tracing::error!("Failed to update games.updated_at for game {game}: {e}");
        }
        *current = Some(game);
    }
}
