//! [DOC: docs/diataxis/reference/storage.md]
//! ActiveGame — the active-Game pointer of the server process.

use std::sync::{Arc, Mutex};

use crate::adapters::driven::storage::Storage;
use crate::domain::model::game::GameId;
use crate::error::EngineError;

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

    /// Moves the pointer to an existing Game. The touch of the Game's
    /// `updated_at` comes first, so a failed touch leaves the pointer where it
    /// was.
    ///
    /// `pub(super)`: only `application/games/` may move the pointer, so a
    /// request cannot end up writing two Games. The first Game comes through
    /// `GameCatalogue::new`; every later move goes through `GameCatalogue`
    /// (switch, create, reset).
    pub(super) fn set(&self, game: GameId) -> Result<(), EngineError> {
        let mut current = self.game.lock().unwrap_or_else(|e| e.into_inner());
        if *current == Some(game) {
            return Ok(());
        }
        self.storage.touch_game_updated_at(game)?;
        *current = Some(game);
        Ok(())
    }

    /// Moves the pointer to a Game that a create or a reset just wrote. Its
    /// `updated_at` is already its creation time, and the old Game of a reset is
    /// already gone, so the move has no write that can fail.
    pub(super) fn set_new(&self, game: GameId) {
        *self.game.lock().unwrap_or_else(|e| e.into_inner()) = Some(game);
    }
}
