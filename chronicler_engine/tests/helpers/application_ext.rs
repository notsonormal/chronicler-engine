//! Test-only `AppState` extension trait for driving pipeline scenarios.

#![allow(dead_code)]

use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::domain::model::state::game_state::GameState;
use chronicler_engine::domain::model::state::message_types::MessageType;

pub trait PipelineHelpers {
    fn wait_for_generation_complete(&self, timeout_ms: u64) -> bool;
    fn latest_state(&self) -> GameState;
    fn save_test_state(&self, state: &GameState);
    fn add_input_and_save(&self, text: &str);
    fn latest_snapshot(
        &self,
    ) -> chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot;
}

impl PipelineHelpers for AppState {
    fn wait_for_generation_complete(&self, timeout_ms: u64) -> bool {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);
        while start.elapsed() < timeout {
            let state = self.persistence_gate.load_or_fresh();
            if !state.narrative.input_buffer.status.is_generating() {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
        false
    }

    fn latest_state(&self) -> GameState {
        let mut state = self.persistence_gate.load_or_fresh();
        self.persistence_gate.load_messages_into_state(&mut state);
        state
    }

    fn save_test_state(&self, state: &GameState) {
        use chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot;
        let snapshot = GameStateSnapshot::from_game_state(state);
        let snapshot_id = self.storage.save_snapshot(&snapshot).unwrap();
        let existing = self.persistence_gate.load_messages().unwrap_or_default();
        for msg in existing {
            let _ = self.storage.delete_message(msg.id);
        }
        for mut msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
            if msg.snapshot_id().is_none() {
                msg.set_snapshot_id(Some(snapshot_id));
            }
            if let Some(swipe) = msg.swipes.first_mut() {
                swipe.snapshot_id = Some(snapshot_id);
            }
            let id = self.storage.insert_message(&msg).unwrap();
            for (idx, swipe) in msg.swipes.iter().enumerate() {
                let _ = self.storage.insert_swipe(id, swipe, idx);
            }
        }
    }

    fn add_input_and_save(&self, text: &str) {
        let mut state = self.latest_state();
        let player_name = self
            .storage
            .get_game(self.game_catalogue.current_game_id())
            .ok()
            .flatten()
            .and_then(|game| self.storage.get_persona(&game.persona_key).ok().flatten())
            .map(|persona| persona.sheet.name)
            .unwrap_or_else(|| "Player".to_string());
        state.add_message(text.to_string(), Some(player_name), MessageType::Input);
        self.save_test_state(&state);
    }

    fn latest_snapshot(
        &self,
    ) -> chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot {
        let state = self.persistence_gate.load_or_fresh();
        chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &state,
        )
    }
}
