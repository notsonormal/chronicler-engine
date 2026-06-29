use crate::domain::model::state::generation_status::{GenerationStatus, InputBuffer};
use crate::domain::model::state::message_types::MessageType;
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
fn test_generation_state_status() {
    let mut buf = InputBuffer::default();

    assert_eq!(buf.status, GenerationStatus::Idle);
    assert!(!buf.status.is_generating());
    assert!(buf.status.error_message().is_none());

    buf.status = GenerationStatus::Error("LLM Error: 429 Too Many Requests".to_string());
    assert_eq!(
        buf.status,
        GenerationStatus::Error("LLM Error: 429 Too Many Requests".to_string())
    );
    assert!(buf.status.error_message().is_some());

    buf.status = GenerationStatus::Generating;
    assert!(buf.status.is_generating());
    assert!(buf.status.error_message().is_none());
}

#[test]
fn test_log_ordering() {
    let mut state = TestGameState::in_room("room1");

    state.add_message("Message 1".into(), None, MessageType::Narration);
    state.add_message("Message 2".into(), None, MessageType::Narration);

    assert_eq!(state.narrative.history().len(), 2);
    assert_eq!(state.narrative.history()[0].text, "Message 1");
    assert_eq!(state.narrative.history()[1].text, "Message 2");
}

#[test]
fn test_delete_last_log_recalculates_ids() {
    let mut state = TestGameState::in_room("room1");

    state.add_message("go north".into(), Some("Player".into()), MessageType::Input);
    state.add_message("You walk north.".into(), None, MessageType::Narration);

    assert_eq!(state.narrative.history.len(), 2);

    state.narrative.history.delete_last().unwrap();
    assert_eq!(state.narrative.history.len(), 1);
    assert_eq!(state.narrative.history.as_slice()[0].text(), "go north");

    state.narrative.history.delete_last().unwrap();
    assert!(state.narrative.history.is_empty());
    state.add_message("go south".into(), Some("Player".into()), MessageType::Input);
    assert_eq!(state.narrative.history.last().unwrap().text(), "go south");
}

#[test]
fn test_push_message_appends_swipe_on_retry_target() {
    let mut state = TestGameState::in_room("room1");

    let target = crate::domain::model::message::Message::new(
        None,
        "Original narration",
        MessageType::Narration,
        None,
        None,
    );
    state.narrative.retry_target = Some(target);

    state.add_message("Retried narration".into(), None, MessageType::Narration);

    assert_eq!(state.narrative.history.len(), 0);

    let target = state.narrative.retry_target.unwrap();
    assert_eq!(target.swipes.len(), 2);
    assert_eq!(target.text(), "Retried narration");
    assert_eq!(target.swipes[0].text, "Original narration");
    assert_eq!(target.swipes[1].text, "Retried narration");
    assert!(target.swipes[1].snapshot_id.is_none());
}

#[test]
fn test_push_message_creates_new_message_when_event_header_mismatches() {
    let mut state = TestGameState::in_room("room1");

    let target = crate::domain::model::message::Message::new(
        None,
        "Original narration",
        MessageType::Narration,
        None,
        None,
    );
    state.narrative.retry_target = Some(target);

    state.narrative.pending_event = Some("Trigger Event".to_string());
    state.add_message("Event narration".into(), None, MessageType::Narration);

    assert_eq!(state.narrative.history.len(), 1);
    assert_eq!(
        state.narrative.history.last().unwrap().text(),
        "Event narration"
    );
    assert_eq!(
        state.narrative.history.last().unwrap().event_header(),
        Some("Trigger Event")
    );

    let target = state.narrative.retry_target.unwrap();
    assert_eq!(target.swipes.len(), 1);
    assert_eq!(target.text(), "Original narration");
}

#[test]
fn test_push_message_creates_new_message_when_no_retry_target() {
    let mut state = TestGameState::in_room("room1");

    state.add_message("Normal narration".into(), None, MessageType::Narration);

    assert_eq!(state.narrative.history.len(), 1);
    assert_eq!(
        state.narrative.history.last().unwrap().text(),
        "Normal narration"
    );
    assert!(state.narrative.retry_target.is_none());
}

use proptest::prelude::*;

fn log_text_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ]{1,50}"
}

fn log_type_strategy() -> impl Strategy<Value = MessageType> {
    prop_oneof![
        Just(MessageType::Narration),
        Just(MessageType::Dialogue),
        Just(MessageType::System),
        Just(MessageType::Input),
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
            state.add_message(text.clone(), None, log_type.clone());
            expected.push((text, log_type));
        }
        let history = state.narrative.history();
        prop_assert_eq!(history.len(), expected.len());
        for (i, (expected_text, expected_type)) in expected.iter().enumerate() {
            prop_assert_eq!(&history[i].text, expected_text);
            prop_assert_eq!(history[i].message_type.clone(), expected_type.clone());
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
            state.add_message(text, None, log_type);
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

#[test]
fn test_npcs_in_area_initialization() {
    let state = TestGameState::in_room("room_1");

    assert!(
        state.scene.npcs_in_area.is_empty(),
        "npcs_in_area should be empty on initialization"
    );
}

#[test]
fn test_npcs_in_area_can_be_populated() {
    let mut state = TestGameState::with_npc("room_1", TestNpc::named("npc_1", "N"));

    let npc = state.npcs.get("npc_1").cloned().expect("Should have npc_1");
    state.scene.npcs_in_area.push(npc);

    assert_eq!(
        state.scene.npcs_in_area.len(),
        1,
        "npcs_in_area should have 1 NPC after population"
    );
    assert_eq!(state.scene.npcs_in_area[0].id, "npc_1", "Should be npc_1");
}

#[test]
fn test_npcs_in_area_can_be_cleared() {
    let mut state = TestGameState::with_npc("room_1", TestNpc::named("npc_1", "N"));

    let npc = state.npcs.get("npc_1").cloned().expect("Should have npc_1");
    state.scene.npcs_in_area.push(npc);

    assert!(
        !state.scene.npcs_in_area.is_empty(),
        "npcs_in_area should be populated"
    );

    state.scene.npcs_in_area.clear();

    assert!(
        state.scene.npcs_in_area.is_empty(),
        "npcs_in_area should be empty after clear"
    );
}

#[test]
fn test_npcs_in_area_can_be_replaced() {
    let mut state = TestGameState::with_npc("room_1", TestNpc::named("npc_1", "N"));

    let npc1 = state.npcs.get("npc_1").cloned().expect("Should have npc_1");
    state.scene.npcs_in_area.push(npc1);

    assert_eq!(state.scene.npcs_in_area.len(), 1, "Should have 1 NPC");

    let new_npcs = vec![];
    state.scene.npcs_in_area = new_npcs;

    assert!(
        state.scene.npcs_in_area.is_empty(),
        "npcs_in_area should be empty after replacement"
    );
}
