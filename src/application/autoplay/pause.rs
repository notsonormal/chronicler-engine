//! [DOC: docs/diataxis/reference/game_flow.md]
//! The test pause point of the Autoplay loop: a hold that stops the loop before an accept

use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::domain::model::game::GameId;

#[derive(Default)]
struct PauseSync {
    paused: bool,
    released: bool,
}

struct PauseState {
    sync: Mutex<PauseSync>,
    cv: Condvar,
}

/// Built into production because the `testing` feature is a default one; with no
/// hold armed, a pause point returns at once.
#[derive(Default)]
pub struct AutoplayPause {
    armed: Mutex<HashMap<GameId, Arc<PauseState>>>,
}

impl AutoplayPause {
    /// Blocks while a hold for `game` is armed; a hold fires once.
    pub fn point(&self, game: GameId) {
        let state = self
            .armed
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&game);
        let Some(state) = state else {
            return;
        };
        let mut sync = state.sync.lock().unwrap_or_else(|e| e.into_inner());
        sync.paused = true;
        state.cv.notify_all();
        while !sync.released {
            sync = state.cv.wait(sync).unwrap_or_else(|e| e.into_inner());
        }
    }

    /// Holds of two Games can be armed together. A second hold of the same Game
    /// goes in only once the first has fired.
    #[must_use]
    pub fn hold(&self, game: GameId) -> PauseHandle {
        let state = Arc::new(PauseState {
            sync: Mutex::new(PauseSync::default()),
            cv: Condvar::new(),
        });
        let mut armed = self.armed.lock().unwrap_or_else(|e| e.into_inner());
        assert!(
            !armed.contains_key(&game),
            "AutoplayPause: a hold of this Game is armed and has not fired yet; wait for it to fire before holding this Game again"
        );
        armed.insert(game, Arc::clone(&state));
        PauseHandle { state }
    }
}

pub struct PauseHandle {
    state: Arc<PauseState>,
}

impl PauseHandle {
    /// Blocks the calling thread, because the loop runs on the blocking pool and
    /// never on a runtime worker. `false` on timeout.
    pub fn wait_paused(&self, timeout_ms: u64) -> bool {
        let sync = self.state.sync.lock().unwrap_or_else(|e| e.into_inner());
        let (sync, _) = self
            .state
            .cv
            .wait_timeout_while(sync, Duration::from_millis(timeout_ms), |sync| !sync.paused)
            .unwrap_or_else(|e| e.into_inner());
        sync.paused
    }

    pub fn release(&self) {
        let mut sync = self.state.sync.lock().unwrap_or_else(|e| e.into_inner());
        sync.released = true;
        self.state.cv.notify_all();
    }
}

/// Releases the hold so a test that panics before `release` cannot strand the loop.
impl Drop for PauseHandle {
    fn drop(&mut self) {
        self.release();
    }
}
