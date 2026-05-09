use std::sync::Arc;

use crate::model::state::{GeneratingGuard, GenerationState, GenerationStatus, LogType};
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

    state.add_log("Narration".into(), None, LogType::Narration);
    state.add_log("User input".into(), Some("Player".into()), LogType::Input);

    let idx = state.get_last_input_index();
    assert!(idx.is_some());
    assert_eq!(state.narration_history[idx.unwrap()].text, "User input");
}

#[test]
fn test_replace_last_ai_response() {
    let mut state = TestGameState::in_room("room1");

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

    state.add_log("User input".into(), Some("Player".into()), LogType::Input);
    assert!(
        state
            .replace_last_ai_response("New response".into())
            .is_err()
    );
}

#[test]
fn test_delete_log() {
    let mut state = TestGameState::in_room("room1");

    state.add_log("Message 1".into(), Some("A".into()), LogType::Narration);
    state.add_log("Message 2".into(), Some("B".into()), LogType::Narration);
    state.add_log("Message 3".into(), Some("C".into()), LogType::Narration);

    let id_to_delete = state.narration_history[1].id;

    // Verify delete works
    state.delete_log(id_to_delete).unwrap();
    assert_eq!(state.narration_history.len(), 2);
    assert_eq!(state.narration_history[0].text, "Message 1");
    assert_eq!(state.narration_history[1].text, "Message 3");

    // Verify delete fails for invalid ID
    assert!(state.delete_log(9999).is_err());
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

    // Guard dropped â€” status reset to Idle
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
