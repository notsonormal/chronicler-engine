use crate::model::state::{GenerationStatus, InputBuffer, LogType};
use crate::test_support::*;

#[test]
fn test_game_state_initialization() {
    let npc = TestNpc::named("npc_1", "N");
    let state = TestGameState::with_npc("room_1", npc);

    assert_eq!(state.movement.current_room_id, "room_1");
    assert_eq!(state.npcs.len(), 1);
    assert!(state.npcs.contains_key("npc_1"));
}

#[test]
fn test_generation_state_input_edge_cases() {
    let mut tui = InputBuffer::default();

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
    let mut tui = InputBuffer::default();

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

    assert_eq!(state.narrative.history().len(), 2);
    assert_eq!(state.narrative.history()[0].text, "Message 1");
    assert_eq!(state.narrative.history()[1].text, "Message 2");
}

#[test]
fn test_edit_log() {
    let mut state = TestGameState::in_room("room1");

    state.add_log("Original text".into(), None, LogType::Narration);
    let id = state.narrative.history()[0].id;

    // Verify edit works
    state.edit_log(id, "Edited text".into()).unwrap();
    assert_eq!(state.narrative.history()[0].text, "Edited text");

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
    assert_eq!(state.narrative.history()[idx.unwrap()].text, "User input");
}

#[test]
fn test_delete_last_log() {
    let mut state = TestGameState::in_room("room1");

    state.add_log("Message 1".into(), Some("A".into()), LogType::Narration);
    state.add_log("Message 2".into(), Some("B".into()), LogType::Narration);
    state.add_log("Message 3".into(), Some("C".into()), LogType::Narration);

    // Verify delete last works
    state.delete_last_log().unwrap();
    assert_eq!(state.narrative.history().len(), 2);
    assert_eq!(state.narrative.history()[0].text, "Message 1");
    assert_eq!(state.narrative.history()[1].text, "Message 2");

    // Verify delete fails for empty history
    state.delete_last_log().unwrap();
    state.delete_last_log().unwrap();
    assert!(state.delete_last_log().is_err());
}

#[test]
fn test_delete_last_log_recalculates_ids() {
    let mut state = TestGameState::in_room("room1");

    // Add Input + Narration (mimics handler flow)
    state.add_log("go north".into(), Some("Player".into()), LogType::Input);
    state.add_log("You walk north.".into(), None, LogType::Narration);

    assert_eq!(state.narrative.history.len(), 2);

    // Delete Narration
    state.delete_last_log().unwrap();
    assert_eq!(state.narrative.history.len(), 1);
    assert_eq!(state.narrative.history.as_slice()[0].text, "go north");

    // Delete Input
    state.delete_last_log().unwrap();
    assert!(state.narrative.history.is_empty());

    // Verify a new Input can be added after delete
    state.add_log("go south".into(), Some("Player".into()), LogType::Input);
    assert_eq!(state.narrative.history.len(), 1);
    assert_eq!(state.narrative.history.last().unwrap().text, "go south");
}

#[test]
fn test_add_log_absorbs_pending_location() {
    let mut state = TestGameState::in_room("room1");
    state.narrative.pending_location = Some("Entrance Hall".to_string());
    state.add_log("You walk in.".into(), None, LogType::Narration);

    let history = state.narrative.history();
    let entry = history.last().unwrap();
    assert_eq!(entry.location_header, Some("Entrance Hall".to_string()));
    assert!(state.narrative.pending_location.is_none());
}

#[test]
fn test_add_log_absorbs_pending_event() {
    let mut state = TestGameState::in_room("room1");
    state.narrative.pending_event = Some("Gabriella Introduction".to_string());
    state.add_log("Gabriella steps forward.".into(), None, LogType::Narration);

    let history = state.narrative.history();
    let entry = history.last().unwrap();
    assert_eq!(
        entry.event_header,
        Some("Gabriella Introduction".to_string())
    );
    assert!(state.narrative.pending_event.is_none());
}

#[test]
fn test_is_last_ai_response_event_continuation_with_event_header() {
    let mut state = TestGameState::in_room("room1");
    state.add_log("go north".into(), Some("Player".into()), LogType::Input);
    state.add_log("You walk north.".into(), None, LogType::Narration);
    state.narrative.pending_event = Some("Carla Introduction".into());
    state.add_log("Carla appears.".into(), None, LogType::Narration);

    assert!(state.is_last_ai_response_event_continuation());
}

#[test]
fn test_is_last_ai_response_event_continuation_without_event_header() {
    let mut state = TestGameState::in_room("room1");
    state.add_log("go north".into(), Some("Player".into()), LogType::Input);
    state.add_log("You walk north.".into(), None, LogType::Narration);

    assert!(!state.is_last_ai_response_event_continuation());
}

// ─── Property-based tests ──────────────────────────────────────────────────────

use proptest::prelude::*;

fn log_text_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,50}"
}

fn log_type_strategy() -> impl Strategy<Value = LogType> {
    prop_oneof![
        Just(LogType::Narration),
        Just(LogType::Dialogue),
        Just(LogType::System),
        Just(LogType::Input),
    ]
}

proptest! {
    #[test]
    fn prop_log_appends_in_order(
        mut state in Just(TestGameState::in_room("room1")),
        entries in prop::collection::vec(
            (log_text_strategy(), log_type_strategy()),
            1..20
        )
    ) {
        let mut expected = Vec::new();
        for (text, log_type) in entries {
            state.add_log(text.clone(), None, log_type.clone());
            expected.push((text, log_type));
        }
        let history = state.narrative.history();
        prop_assert_eq!(history.len(), expected.len());
        for (i, (expected_text, expected_type)) in expected.iter().enumerate() {
            prop_assert_eq!(&history[i].text, expected_text);
            prop_assert_eq!(history[i].log_type.clone(), expected_type.clone());
        }
    }

    #[test]
    fn prop_log_turns_never_exceed_max_capacity(
        mut state in Just(TestGameState::in_room("room1")),
        entries in prop::collection::vec(
            (log_text_strategy(), log_type_strategy()),
            1000..1050
        )
    ) {
        for (text, log_type) in entries {
            state.add_log(text, None, log_type);
        }
        prop_assert!(
            state.narrative.history.len() <= 1000,
            "message count {} exceeds max 1000",
            state.narrative.history.len()
        );
    }

    #[test]
    fn prop_npcs_in_area_are_always_known(
        mut state in Just(TestGameState::in_room("room1")),
    ) {
        let npc = TestNpc::named("alice", "Alice");
        state.npcs.insert(npc.id.clone(), npc.clone());
        state.scene.npcs_in_area.push(npc.clone());

        for npc in &state.scene.npcs_in_area {
            prop_assert!(
                state.npcs.contains_key(&npc.id),
                "npcs_in_area contains unknown NPC '{}'",
                npc.id
            );
        }
    }

    #[test]
    fn prop_npc_encounter_log_references_valid_npcs(
        mut state in Just(TestGameState::in_room("room1")),
    ) {
        let npc = TestNpc::named("bob", "Bob");
        state.npcs.insert(npc.id.clone(), npc.clone());

        let entry = state.npc_encounter_log.npcs.entry(npc.id.clone()).or_default();
        entry.times_met += 1;

        for npc_id in state.npc_encounter_log.npcs.keys() {
            prop_assert!(
                state.npcs.contains_key(npc_id),
                "npc_encounter_log references unknown NPC '{}'",
                npc_id
            );
        }
    }
}
