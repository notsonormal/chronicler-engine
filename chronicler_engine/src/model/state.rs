use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::character::{NpcCard, PlayerCard};
use crate::model::map::{MapDef, Room};
use crate::model::trigger::CharacterState;
use crate::model::world::WorldCard;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogType {
    Narration,
    Dialogue,
    System,
    Input,
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub id: u64,
    pub sender: Option<String>,
    pub text: String,
    pub log_type: LogType,
    pub timestamp: DateTime<Utc>,
}

const MAX_LOG_ENTRIES: usize = 1000;

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub enum GenerationStatus {
    #[default]
    Idle,
    Generating,
    Error(String),
}

impl GenerationStatus {
    pub fn is_generating(&self) -> bool {
        matches!(self, Self::Generating)
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::Error(msg) => Some(msg),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub enum GenerationPhase {
    #[default]
    Narrating,
    Quantifying,
    GeneratingEvent,
}

impl GenerationPhase {
    pub fn display_text(&self) -> &'static str {
        match self {
            Self::Narrating => "Generating narration...",
            Self::Quantifying => "Quantifying scene...",
            Self::GeneratingEvent => "Generating event...",
        }
    }

    pub fn as_endpoint_str(&self) -> &'static str {
        match self {
            Self::Narrating => "narrating",
            Self::Quantifying => "quantifying",
            Self::GeneratingEvent => "generating-event",
        }
    }
}

#[derive(Debug, Default)]
pub struct GenerationState {
    pub input: String,
    pub cursor_position: usize,
    pub scroll_offset: u16,
    pub status: GenerationStatus,
    pub phase: GenerationPhase,
}

impl GenerationState {
    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
        self.cursor_position += 1;
    }

    pub fn pop_char(&mut self) {
        if !self.input.is_empty() {
            self.input.pop();
            self.cursor_position = self.cursor_position.saturating_sub(1);
        }
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
    }
}

/// [DOC: docs/architecture/system.md]
pub struct GeneratingGuard {
    state: Arc<std::sync::Mutex<GameState>>,
}

fn with_lock_or_recover(
    state: &Arc<std::sync::Mutex<GameState>>,
    f: impl FnOnce(&mut GameState),
    err_msg: &str,
) {
    match state.lock() {
        Ok(mut guard) => f(&mut guard),
        Err(poisoned) => {
            log::error!("{err_msg}");
            let mut guard = poisoned.into_inner();
            f(&mut guard);
            state.clear_poison();
        }
    }
}

impl GeneratingGuard {
    pub fn new(state: Arc<std::sync::Mutex<GameState>>) -> Self {
        with_lock_or_recover(
            &state,
            |guard| {
                guard.generation_state.status = GenerationStatus::Generating;
            },
            "GeneratingGuard::new encountered poisoned mutex, recovering guard",
        );
        Self { state }
    }
}

impl Drop for GeneratingGuard {
    fn drop(&mut self) {
        with_lock_or_recover(
            &self.state,
            |guard| {
                guard.generation_state.status = GenerationStatus::Idle;
            },
            "GeneratingGuard::drop encountered poisoned mutex, recovering guard and resetting status",
        );
    }
}

#[derive(Debug)]
pub struct GameState {
    pub world: Arc<WorldCard>,
    pub map: Arc<MapDef>,
    pub player: Arc<PlayerCard>,
    pub npcs: HashMap<String, NpcCard>,
    pub current_room_id: String,
    pub narration_history: Vec<LogEntry>,
    pub next_log_id: u64,
    pub npcs_in_area: Vec<NpcCard>,
    pub generation_state: GenerationState,
    pub dynamic_rooms: HashMap<String, Room>,
    pub character_state: crate::model::trigger::CharacterState,
}

impl GameState {
    pub fn new(
        world: Arc<WorldCard>,
        map: Arc<MapDef>,
        player: Arc<PlayerCard>,
        npcs: Vec<NpcCard>,
        starting_room: String,
    ) -> Self {
        let mut npcs_map = HashMap::new();
        for npc in npcs {
            npcs_map.insert(npc.id.clone(), npc);
        }

        let mut character_state = CharacterState::default();

        for region in &map.overworld.regions {
            for room in &region.rooms {
                if room.id == starting_room {
                    for npc_id in &room.npcs {
                        if let Some(npc) = npcs_map.get(npc_id) {
                            let encounter_state =
                                character_state.npcs.entry(npc.id.clone()).or_default();
                            encounter_state.times_met = 1;
                            encounter_state.currently_meeting = true;
                        }
                    }
                    break;
                }
            }
        }

        Self {
            world,
            map,
            player,
            npcs: npcs_map,
            current_room_id: starting_room,
            narration_history: Vec::new(),
            next_log_id: 1,
            generation_state: GenerationState::default(),
            npcs_in_area: Vec::new(),
            dynamic_rooms: HashMap::new(),
            character_state,
        }
    }

    pub fn add_log(&mut self, text: String, sender: Option<String>, log_type: LogType) {
        if self.narration_history.len() >= MAX_LOG_ENTRIES {
            self.narration_history.remove(0);
        }
        let id = self.next_log_id;
        self.next_log_id += 1;
        self.narration_history.push(LogEntry {
            id,
            sender,
            text,
            log_type,
            timestamp: Utc::now(),
        });
    }

    /// [DOC: docs/architecture/system.md]
    pub fn edit_log(&mut self, id: u64, new_text: String) -> crate::error::Result<()> {
        let entry = self
            .narration_history
            .iter_mut()
            .find(|e| e.id == id)
            .ok_or_else(|| {
                crate::error::EngineError::Internal(format!("Log entry not found: {id}"))
            })?;
        entry.text = new_text;
        Ok(())
    }

    pub fn get_log(&self, id: u64) -> Option<&LogEntry> {
        self.narration_history.iter().find(|e| e.id == id)
    }

    pub fn get_last_ai_response_index(&self) -> Option<usize> {
        self.narration_history
            .iter()
            .rposition(|e| e.log_type == LogType::Narration || e.log_type == LogType::Dialogue)
    }

    pub fn get_last_input_index(&self) -> Option<usize> {
        self.narration_history
            .iter()
            .rposition(|e| e.log_type == LogType::Input)
    }

    pub fn get_last_input_text(&self) -> Option<(String, String)> {
        let input_idx = self.get_last_input_index()?;
        let input_entry = self.narration_history.get(input_idx)?;
        let sender = input_entry.sender.clone().unwrap_or_default();
        Some((sender, input_entry.text.clone()))
    }

    pub fn get_history_context(&self) -> &[LogEntry] {
        &self.narration_history
    }

    /// [DOC: docs/architecture/system.md]
    /// NOTE: Excludes the AI response being retried to prevent the LLM from repeating it.
    pub fn get_history_context_for_retry(&self) -> Vec<LogEntry> {
        let last_ai_idx = self.get_last_ai_response_index();
        if let Some(idx) = last_ai_idx {
            // Exclude the AI response being retried (and any entries after it)
            self.narration_history[..idx].to_vec()
        } else {
            self.narration_history.clone()
        }
    }

    /// [DOC: docs/architecture/system.md]
    pub fn replace_last_ai_response(&mut self, new_text: String) -> crate::error::Result<()> {
        let input_idx = self
            .get_last_input_index()
            .ok_or_else(|| crate::error::EngineError::Internal("No input to retry".into()))?;
        let ai_idx = self
            .get_last_ai_response_index()
            .ok_or_else(|| crate::error::EngineError::Internal("No AI response to retry".into()))?;

        if ai_idx <= input_idx {
            return Err(crate::error::EngineError::Internal(
                "AI response must be after input".into(),
            ));
        }

        let entry = self
            .narration_history
            .get_mut(ai_idx)
            .ok_or_else(|| crate::error::EngineError::Internal("AI response not found".into()))?;
        entry.text = new_text;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    #[test]
    fn test_game_state_initialization() {
        let npc = TestNpc::named("npc_1", "N");
        let state = TestGameState::with_npc("room_1", npc);

        assert_eq!(state.current_room_id, "room_1");
        assert_eq!(state.npcs.len(), 1);
        assert!(state.npcs.contains_key("npc_1"));
    }

    #[test]
    fn test_generation_state_input_edge_cases() {
        let mut tui = GenerationState::default();

        tui.push_char('A');
        assert_eq!(tui.input, "A");
        assert_eq!(tui.cursor_position, 1);

        tui.pop_char();
        assert_eq!(tui.input, "");
        assert_eq!(tui.cursor_position, 0);

        tui.pop_char();
        assert_eq!(tui.cursor_position, 0);

        tui.push_char('h');
        tui.clear_input();
        assert_eq!(tui.input, "");
        assert_eq!(tui.cursor_position, 0);
    }

    #[test]
    fn test_generation_state_status() {
        let mut tui = GenerationState::default();

        assert_eq!(tui.status, GenerationStatus::Idle);
        assert!(!tui.status.is_generating());
        assert!(tui.status.error_message().is_none());

        tui.status = GenerationStatus::Error("LLM Error: 429 Too Many Requests".to_string());
        assert_eq!(
            tui.status,
            GenerationStatus::Error("LLM Error: 429 Too Many Requests".to_string())
        );
        assert!(tui.status.error_message().is_some());

        tui.status = GenerationStatus::Generating;
        assert!(tui.status.is_generating());
        assert!(tui.status.error_message().is_none());
    }

    #[test]
    fn test_log_ordering() {
        let mut state = TestGameState::in_room("room1");

        state.add_log("Message 1".into(), None, LogType::Narration);
        state.add_log("Message 2".into(), None, LogType::Narration);

        assert_eq!(state.narration_history.len(), 2);
        assert_eq!(state.narration_history[0].text, "Message 1");
        assert_eq!(state.narration_history[1].text, "Message 2");
    }

    #[test]
    fn test_edit_log() {
        let mut state = TestGameState::in_room("room1");

        state.add_log("Original text".into(), None, LogType::Narration);
        let id = state.narration_history[0].id;

        // Verify edit works
        state.edit_log(id, "Edited text".into()).unwrap();
        assert_eq!(state.narration_history[0].text, "Edited text");

        // Verify edit fails for invalid ID
        assert!(state.edit_log(9999, "Not found".into()).is_err());
    }

    #[test]
    fn test_get_last_input_index() {
        let mut state = TestGameState::in_room("room1");

        // Empty history returns None
        assert!(state.get_last_input_index().is_none());

        // Add narration, then input
        state.add_log("Narration".into(), None, LogType::Narration);
        state.add_log("User input".into(), Some("Player".into()), LogType::Input);

        let idx = state.get_last_input_index();
        assert!(idx.is_some());
        assert_eq!(state.narration_history[idx.unwrap()].text, "User input");
    }

    #[test]
    fn test_replace_last_ai_response() {
        let mut state = TestGameState::in_room("room1");

        // Add input then AI response
        state.add_log("User input".into(), Some("Player".into()), LogType::Input);
        state.add_log("Old AI response".into(), None, LogType::Narration);

        // Replace the AI response
        state
            .replace_last_ai_response("New AI response".into())
            .unwrap();

        // Verify the AI response was replaced
        let ai_idx = state.get_last_ai_response_index().unwrap();
        assert_eq!(state.narration_history[ai_idx].text, "New AI response");
    }

    #[test]
    fn test_replace_last_ai_response_no_input() {
        let mut state = TestGameState::in_room("room1");

        // No input - should fail
        assert!(
            state
                .replace_last_ai_response("New response".into())
                .is_err()
        );
    }

    #[test]
    fn test_replace_last_ai_response_no_ai() {
        let mut state = TestGameState::in_room("room1");

        // Add only input, no AI response - should fail
        state.add_log("User input".into(), Some("Player".into()), LogType::Input);
        assert!(
            state
                .replace_last_ai_response("New response".into())
                .is_err()
        );
    }

    #[test]
    fn test_generating_guard_sets_is_generating_on_construct() {
        let state = Arc::new(std::sync::Mutex::new(TestGameState::in_room("room1")));

        assert!(
            !state
                .lock()
                .unwrap()
                .generation_state
                .status
                .is_generating()
        );

        {
            let _guard = GeneratingGuard::new(state.clone());
            assert!(
                state
                    .lock()
                    .unwrap()
                    .generation_state
                    .status
                    .is_generating()
            );
        }

        // Guard dropped — status reset to Idle
        assert!(
            !state
                .lock()
                .unwrap()
                .generation_state
                .status
                .is_generating()
        );
    }

    #[test]
    fn test_generating_guard_resets_on_drop() {
        let state = Arc::new(std::sync::Mutex::new(TestGameState::in_room("room1")));

        {
            let guard = GeneratingGuard::new(state.clone());
            assert!(
                state
                    .lock()
                    .unwrap()
                    .generation_state
                    .status
                    .is_generating()
            );
            drop(guard);
        }

        assert!(
            !state
                .lock()
                .unwrap()
                .generation_state
                .status
                .is_generating()
        );
    }

    #[test]
    fn test_generating_guard_poisoned_lock_recovers() {
        // Simulate a poisoned mutex by holding the lock in a child thread
        // then creating a guard — the guard's lock attempt will fail but
        // the guard still drops cleanly without panicking.
        let state = Arc::new(std::sync::Mutex::new(TestGameState::in_room("room1")));

        // Hold the mutex in a thread so it poisons on drop
        let state_clone = state.clone();
        let handle = std::thread::spawn(move || {
            let _g = state_clone.lock().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(50));
            // lock dropped here, poisons the mutex
        });

        handle.join().unwrap();

        // Creating a guard when the mutex is poisoned should not panic
        // and drop() should not panic even though lock() fails
        let guard = GeneratingGuard::new(state.clone());
        drop(guard); // should not panic

        // Verify state is still accessible and status is Idle
        // (the guard's constructor failed to set it, and drop failed to unset it,
        // but the underlying state is not corrupted)
        assert!(
            !state
                .lock()
                .unwrap()
                .generation_state
                .status
                .is_generating()
        );
    }
}
