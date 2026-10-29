//! The handles of the stub server: what a test reads and moves on a running stub, one handle for each concern.

use super::*;

#[derive(Clone)]
pub struct StubLifecycleHandle {
    pub(super) addr: SocketAddr,
    pub(super) shutdown: Arc<std::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    pub(super) state: Arc<StubState>,
}

impl StubLifecycleHandle {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn stop(&self) {
        if let Some(tx) = self
            .shutdown
            .lock()
            .expect("stub shutdown lock poisoned")
            .take()
        {
            let _ = tx.send(());
        }
    }

    pub async fn restart(&self) {
        let listener = tokio::net::TcpListener::bind(self.addr)
            .await
            .unwrap_or_else(|e| panic!("rebind stub on {}: {e}", self.addr));
        let tx = serve_stub(listener, Arc::clone(&self.state));
        *self.shutdown.lock().expect("stub shutdown lock poisoned") = Some(tx);
    }
}

/// The numbers of `GET /changes`, and what the page asked and loaded.
#[derive(Clone)]
pub struct StubChangesHandle {
    pub(super) state: Arc<StubState>,
}

impl StubChangesHandle {
    /// Moves the number of one kind.
    pub fn bump(&self, kind: ChangeKind) {
        self.state.bump(&[kind]);
    }

    pub fn set_server(&self, server: &str) {
        self.state
            .changes
            .lock()
            .expect("stub changes lock poisoned")
            .versions
            .server = server.to_string();
    }

    /// Switches the active Game and moves the revision, as the engine's switch does.
    pub fn set_game(&self, game: Option<u64>) {
        let mut changes = self
            .state
            .changes
            .lock()
            .expect("stub changes lock poisoned");
        changes.set_game(game);
        changes.versions.revision += 1;
    }

    /// The `X-Chronicler-Game` header of each `POST` to `path`, in arrival order.
    pub fn game_headers(&self, path: &str) -> Vec<Option<String>> {
        self.state.action_layers.game_headers(path)
    }

    /// Makes the action routes treat `game` as the active Game while `GET /changes`
    /// still names the old one: the page that has not asked since a switch.
    pub fn set_game_of_actions(&self, game: u64) {
        self.state.action_layers.set_game_of_actions(game);
    }

    pub fn asks(&self) -> usize {
        self.state
            .change_asks
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /// The requests the stub received for a fragment path or `/status/busy`,
    /// failed ones included.
    pub fn loads(&self, path: &str) -> usize {
        self.state
            .fragment_loads
            .lock()
            .expect("stub load counts lock poisoned")
            .get(path)
            .copied()
            .unwrap_or(0)
    }

    /// The parts that each `GET /screen` named, in arrival order.
    pub fn screen_requests(&self) -> Vec<Vec<String>> {
        self.state
            .screen_requests
            .lock()
            .expect("stub screen requests lock poisoned")
            .clone()
    }

    /// Fails every fragment, `/status/busy` and `/screen` request with a 500 and
    /// leaves `GET /changes` working, so a screen load fails after a good ask.
    pub fn set_loads_failing(&self, failing: bool) {
        self.state
            .loads_failing
            .store(failing, std::sync::atomic::Ordering::SeqCst);
    }

    /// Holds the story-log answer back, so a load is in flight for a while.
    pub fn set_story_log_delay_ms(&self, millis: u64) {
        self.state
            .story_log_delay_ms
            .store(millis, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Clone)]
pub struct StubGamesHandle {
    pub(super) state: Arc<StubState>,
}

impl StubGamesHandle {
    /// The Game posture save fails until armed, so the tests that drive a failed
    /// save keep their outcome.
    pub fn set_posture_saves_succeed(&self, succeeds: bool) {
        self.state
            .games
            .lock()
            .expect("stub games lock poisoned")
            .posture_saves_succeed = succeeds;
    }

    pub fn set_rename_failing(&self, failing: bool) {
        self.state
            .games
            .lock()
            .expect("stub games lock poisoned")
            .rename_failing = failing;
    }

    /// The engine has no route that refuses a rename, so this is the way to see a page take
    /// a refusal from a dialog form.
    pub fn set_rename_refused(&self, refused: bool) {
        self.state
            .games
            .lock()
            .expect("stub games lock poisoned")
            .rename_refused = refused;
    }

    /// The saved Games are `count` Games named "Saga 01", "Saga 02", ..., listed in
    /// that order. The `games` number moves.
    pub fn set_saved_games(&self, count: u64) {
        self.state
            .games
            .lock()
            .expect("stub games lock poisoned")
            .saved = (1..=count)
            .map(|number| (100 + number, format!("Saga {number:02}")))
            .collect();
        self.state.bump(&[ChangeKind::GAMES]);
    }

    pub fn saved_names(&self) -> Vec<String> {
        self.state
            .games
            .lock()
            .expect("stub games lock poisoned")
            .saved
            .iter()
            .map(|(_, name)| name.clone())
            .collect()
    }
}

#[derive(Clone)]
pub struct StubLlmHandle {
    pub(super) state: Arc<StubState>,
}

impl StubLlmHandle {
    /// A new LLM message is recorded: the next load of the panel lists it first.
    pub fn record_message(&self) {
        self.state
            .llm_message_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.state.bump(&[ChangeKind::LLM_MESSAGES]);
    }

    /// Retention removed every record, but the panel still lists the rows it loaded:
    /// the record route answers a 404 from now on.
    pub fn prune_records(&self) {
        self.state
            .llm_records_pruned
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Clone)]
pub struct StubStoryLogHandle {
    pub(super) state: Arc<StubState>,
}

impl StubStoryLogHandle {
    pub fn set_extra_oldest(&self, present: bool) {
        self.state
            .extra_oldest
            .store(present, std::sync::atomic::Ordering::SeqCst);
        self.state.bump(&[ChangeKind::STORY]);
    }

    pub fn append_narration(&self, text: &str) {
        *self
            .state
            .newest_narration
            .lock()
            .expect("stub narration lock poisoned") = Some(text.to_string());
        self.state.bump(&[ChangeKind::STORY, ChangeKind::GAMES]);
    }

    pub fn remove_newest(&self) {
        self.state
            .deleted
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.state.bump(&[ChangeKind::STORY]);
    }
}

#[derive(Clone)]
pub struct StubOptionsHandle {
    pub(super) state: Arc<StubState>,
}

impl StubOptionsHandle {
    pub fn serve_alternate_set(&self, alternate: bool) {
        self.state
            .dock_options_alt
            .store(alternate, std::sync::atomic::Ordering::SeqCst);
        self.state.bump(&[ChangeKind::OPTIONS]);
    }

    pub fn set_failure(&self, failed: bool) {
        self.state
            .options_failed
            .store(failed, std::sync::atomic::Ordering::SeqCst);
        self.state.bump(&[ChangeKind::OPTIONS]);
    }
}

#[derive(Clone)]
pub struct StubSaveHandle {
    pub(super) state: Arc<StubState>,
}

impl StubSaveHandle {
    pub fn set_save_succeeds(&self, succeeds: bool) {
        self.state
            .save_succeeds
            .store(succeeds, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn saved_texts(&self) -> Vec<String> {
        self.state
            .saved_texts
            .lock()
            .expect("stub saved texts lock poisoned")
            .clone()
    }

    pub fn set_save_invalid(&self, invalid: bool) {
        self.state
            .save_invalid
            .store(invalid, std::sync::atomic::Ordering::SeqCst);
    }

    /// A connection Add or Edit refuses until armed, so the tests that drive a
    /// refused save keep their outcome.
    pub fn set_connection_saves_succeed(&self, succeeds: bool) {
        self.state
            .connection_saves_succeed
            .store(succeeds, std::sync::atomic::Ordering::SeqCst);
    }

    /// A role save answers 200 but the engine keeps the value it had, so the next
    /// render differs from the player's choice.
    pub fn set_role_saves_keep_stored_value(&self, keeps: bool) {
        self.state
            .role_saves_keep_stored_value
            .store(keeps, std::sync::atomic::Ordering::SeqCst);
    }

    /// A role save commits and moves its number at once, but its answer waits
    /// until `release_role_save_answer`: the save stays in flight in the page
    /// while the engine already shows its effect.
    pub fn hold_role_save_answers(&self) {
        self.state
            .role_save_answers_held
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn release_role_save_answer(&self) {
        self.state.role_save_answer_permits.add_permits(1);
    }

    pub fn set_story_writes_refused_as_busy(&self, refused: bool) {
        self.state
            .story_writes_refused_as_busy
            .store(refused, std::sync::atomic::Ordering::SeqCst);
    }

    /// The retry and the retrigger answer as an engine that took them: its Job holds
    /// the Game, so the busy state moves. They fail until armed.
    pub fn set_generation_requests_accepted(&self, accepted: bool) {
        self.state
            .generation_requests_accepted
            .store(accepted, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_delete_succeeds(&self, succeeds: bool) {
        self.state
            .delete_succeeds
            .store(succeeds, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Clone)]
pub struct StubRowDeleteHandle {
    pub(super) state: Arc<StubState>,
}

impl StubRowDeleteHandle {
    /// A preset delete refuses and a world delete fails until armed, so the
    /// tests that drive those paths keep their outcome.
    pub fn set_row_deletes_succeed(&self, succeeds: bool) {
        self.state
            .row_deletes_succeed
            .store(succeeds, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_world_delete_refused(&self, refused: bool) {
        self.state
            .world_delete_refused
            .store(refused, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Clone)]
pub struct StubFailureHandle {
    pub(super) state: Arc<StubState>,
}

impl StubFailureHandle {
    pub fn set_header_degraded(&self, degraded: bool) {
        self.state
            .header_degraded
            .store(degraded, std::sync::atomic::Ordering::SeqCst);
        self.state.bump(&[ChangeKind::LLM_MESSAGES]);
    }

    pub fn set_asks_and_loads_failing(&self, failing: bool) {
        self.state
            .asks_and_loads_failing
            .store(failing, std::sync::atomic::Ordering::SeqCst);
    }

    /// Every panel loads once per page load, or when its section opens, so arming
    /// this has no effect on a panel that has loaded.
    pub fn set_panel_loads_failing(&self, failing: bool) {
        self.state
            .panel_loads_failing
            .store(failing, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_world_posture_failing(&self, failing: bool) {
        self.state
            .worlds
            .lock()
            .expect("stub worlds lock poisoned")
            .posture_failing = failing;
    }

    pub fn set_preset_card_refused(&self, refused: bool) {
        self.state
            .preset_card_refused
            .store(refused, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Clone)]
pub struct StubSwipeHandle {
    pub(super) state: Arc<StubState>,
}

impl StubSwipeHandle {
    pub fn set_two_swipes(&self) {
        self.state
            .two_swipes
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.state.bump(&[ChangeKind::STORY]);
    }

    pub fn set_switch_failing(&self) {
        self.state
            .switch_fails
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

#[derive(Clone)]
pub struct StubStatusHandle {
    pub(super) state: Arc<StubState>,
}

impl StubStatusHandle {
    /// Moves the busy number, so the status display loads on the next ask.
    pub fn set(&self, status: StubStatus) {
        *self.state.status.lock().expect("stub status lock poisoned") = status;
        self.state.bump(&[ChangeKind::BUSY_STATE]);
    }
}

#[derive(Clone)]
pub struct StubRetriggerHandle {
    pub(super) state: Arc<StubState>,
}

#[derive(Clone)]
pub struct StubActionHandle {
    pub(super) state: Arc<StubState>,
}

impl StubActionHandle {
    pub fn count(&self) -> usize {
        self.state
            .action_requests
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl StubRetriggerHandle {
    pub fn count(&self) -> usize {
        self.state
            .retrigger_requests
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}
