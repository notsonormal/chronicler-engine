//! The rules a `Tail` derives from the end of the story.

use super::message::{Maker, Message, Swipe, SwipeAt};
use super::state::message_types::{MessageKind, MessageType};
use super::state::movement::MovementState;
use super::state::scene_state::SceneState;
use super::state::swipe_state::SwipeState;
use super::state::trigger_context::{EventRecord, TriggerRef};
use super::tail::Tail;
use super::trigger::NpcEncounterLog;

fn state_in(room: &str, trigger_context: Option<EventRecord>) -> SwipeState {
    SwipeState {
        movement: MovementState {
            current_room_id: room.to_string(),
            dynamic_rooms: Default::default(),
        },
        scene: SceneState::default(),
        npc_encounter_log: NpcEncounterLog::default(),
        trigger_context,
    }
}

fn swipe(text: &str, state: SwipeState) -> Swipe {
    Swipe::new(text, Maker::Narrator, state)
}

fn trigger() -> TriggerRef {
    TriggerRef {
        npc_id: "npc_1".to_string(),
        trigger_idx: 0,
        trigger_name: "The gate opens".to_string(),
        trigger_repeat: false,
    }
}

fn event_record() -> EventRecord {
    EventRecord {
        trigger: trigger(),
        header_name: "The gate opens".to_string(),
        direction: "the gate swings open".to_string(),
    }
}

fn tail_after_one_message(kind: MessageKind) -> Tail {
    let mut last = Message::new(
        MessageType::Narration,
        kind,
        swipe("first", state_in("room2", None)),
    );
    last.id = 7;
    last.swipes = vec![
        swipe("first", state_in("room2", None)),
        swipe("second", state_in("room3", None)),
    ];
    last.active_swipe_index = 1;
    Tail {
        last,
        state_before_last: Some((
            SwipeAt {
                message: 6,
                swipe: 0,
            },
            state_in("room1", None),
        )),
        count: 2,
    }
}

fn tail_of_the_opening() -> Tail {
    let mut opening = Message::new(
        MessageType::Narration,
        MessageKind::Narration,
        swipe("opening", state_in("room1", None)),
    );
    opening.id = 3;
    opening.swipes = vec![
        swipe("opening", state_in("room1", None)),
        swipe("retold", state_in("room2", None)),
    ];
    opening.active_swipe_index = 1;
    Tail {
        last: opening,
        state_before_last: None,
        count: 1,
    }
}

#[test]
fn start_at_names_the_active_swipe_of_the_message_before_the_last_one() {
    let tail = tail_after_one_message(MessageKind::Narration);

    assert_eq!(
        tail.start_at(),
        SwipeAt {
            message: 6,
            swipe: 0
        },
        "a Retry of the last Message starts from the state it was written from"
    );
}

#[test]
fn start_at_names_the_first_swipe_of_the_opening() {
    let tail = tail_of_the_opening();

    assert_eq!(
        tail.start_at(),
        SwipeAt {
            message: 3,
            swipe: 0
        },
        "a Retry of the opening starts from its first Swipe, not its active one"
    );
}

#[test]
fn current_state_is_the_state_of_the_active_swipe() {
    let tail = tail_after_one_message(MessageKind::Narration);

    assert_eq!(
        tail.current_state()
            .map(|state| state.movement.current_room_id.as_str()),
        Some("room3")
    );
}

#[test]
fn current_state_is_none_when_the_active_swipe_is_missing() {
    let mut tail = tail_after_one_message(MessageKind::Narration);
    tail.last.active_swipe_index = 9;

    assert!(tail.current_state().is_none());
}

#[test]
fn retrigger_allowed_is_true_for_a_narration_with_a_trigger_context() {
    let mut tail = tail_after_one_message(MessageKind::Narration);
    tail.last.swipes[1].state = state_in("room3", Some(event_record()));

    assert!(tail.retrigger_allowed());
}

#[test]
fn retrigger_allowed_is_true_for_an_impersonate_input_with_a_trigger_context() {
    let mut tail = tail_after_one_message(MessageKind::ImpersonateInput);
    tail.last.swipes[1].state = state_in("room3", Some(event_record()));

    assert!(tail.retrigger_allowed());
}

#[test]
fn retrigger_allowed_is_false_for_an_event_narration_with_a_trigger_context() {
    let mut tail = tail_after_one_message(MessageKind::EventNarration(event_record()));
    tail.last.swipes[1].state = state_in("room3", Some(event_record()));

    assert!(!tail.retrigger_allowed());
}

#[test]
fn retrigger_allowed_is_false_for_a_typed_input_with_a_trigger_context() {
    let mut tail = tail_after_one_message(MessageKind::TypedInput);
    tail.last.swipes[1].state = state_in("room3", Some(event_record()));

    assert!(!tail.retrigger_allowed());
}

#[test]
fn retrigger_allowed_is_false_without_a_trigger_context() {
    let tail = tail_after_one_message(MessageKind::Narration);

    assert!(!tail.retrigger_allowed());
}

#[test]
fn retrigger_allowed_is_false_without_a_current_state() {
    let mut tail = tail_after_one_message(MessageKind::Narration);
    tail.last.active_swipe_index = 9;

    assert!(!tail.retrigger_allowed());
}

#[test]
fn can_delete_is_true_with_more_than_one_message() {
    let tail = tail_after_one_message(MessageKind::Narration);

    assert!(tail.can_delete());
}

#[test]
fn can_delete_is_false_with_one_message() {
    let tail = tail_of_the_opening();

    assert!(!tail.can_delete(), "the story keeps its last Message");
}

#[test]
fn current_at_names_the_active_swipe_of_the_last_message() {
    let tail = tail_after_one_message(MessageKind::Narration);

    assert_eq!(
        tail.current_at(),
        SwipeAt {
            message: 7,
            swipe: 1
        },
        "a new Message starts from the current state"
    );
}
