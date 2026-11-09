//! [DOC: docs/diataxis/reference/storage.md]
//! ActiveGame — the active-Game pointer of the server process.

use std::sync::{Arc, Mutex, MutexGuard};

use crate::adapters::driven::storage::Storage;
use crate::domain::model::game::GameId;
use crate::error::EngineError;

/// `GameCatalogue` is its only reader and its only writer; `lock`'s visibility
/// keeps the write side a compile-time fact.
///
/// The pointer always names a Game that exists: a switch, a delete, a create
/// and a reset hold a `PointerHold` from their check to their last write.
///
/// The lock order is this mutex, then the guard of `GameWriter`, then the
/// backend mutex, then the connection mutex. Never the guard, then this mutex.
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

    pub(super) fn lock(&self) -> PointerHold<'_> {
        PointerHold {
            storage: &self.storage,
            game: self.game.lock().unwrap_or_else(|e| e.into_inner()),
        }
    }
}

pub(super) struct PointerHold<'a> {
    storage: &'a Storage,
    game: MutexGuard<'a, Option<GameId>>,
}

impl PointerHold<'_> {
    pub(super) fn get(&self) -> Option<GameId> {
        *self.game
    }

    /// The touch comes first, so a failed touch leaves the pointer where it was.
    pub(super) fn set(&mut self, game: GameId) -> Result<(), EngineError> {
        if *self.game == Some(game) {
            return Ok(());
        }
        self.storage.touch_game_updated_at(game)?;
        *self.game = Some(game);
        Ok(())
    }

    /// A Game that was just written already has a fresh `updated_at`, so the move skips the touch.
    pub(super) fn set_new(&mut self, game: GameId) {
        *self.game = Some(game);
    }
}
