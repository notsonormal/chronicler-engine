//! The record a generation writes, one branch per test.

use crate::domain::model::agent::Kind;
use crate::domain::model::message::Maker;
use crate::domain::model::state::message_types::{MessageKind, MessageType};
use crate::domain::model::state::trigger_context::{EventRecord, TriggerRef};

use super::generation_record::{maker, message_kind, message_type};

fn event_record() -> EventRecord {
    EventRecord {
        trigger: TriggerRef {
            npc_id: "npc_1".to_string(),
            trigger_idx: 0,
            trigger_name: "The gate opens".to_string(),
            trigger_repeat: false,
        },
        name: "The gate opens".to_string(),
        direction: "the gate swings open".to_string(),
    }
}

#[test]
fn message_kind_of_a_narration_is_a_narration() {
    assert_eq!(
        message_kind(Kind::Narration, None),
        Some(MessageKind::Narration)
    );
}

#[test]
fn message_kind_of_an_event_narration_holds_its_record() {
    assert_eq!(
        message_kind(Kind::Event, Some(&event_record())),
        Some(MessageKind::EventNarration(event_record()))
    );
}

#[test]
fn message_kind_of_an_event_without_a_record_is_none() {
    assert_eq!(message_kind(Kind::Event, None), None);
}

#[test]
fn message_kind_of_an_impersonate_input_is_an_impersonate_input() {
    assert_eq!(
        message_kind(Kind::Impersonate, None),
        Some(MessageKind::ImpersonateInput)
    );
}

#[test]
fn message_kind_of_a_user_regen_is_a_typed_input() {
    assert_eq!(
        message_kind(Kind::UserRegen, None),
        Some(MessageKind::TypedInput)
    );
}

#[test]
fn message_type_of_a_narration_is_a_narration() {
    assert_eq!(message_type(Kind::Narration), MessageType::Narration);
}

#[test]
fn message_type_of_an_event_narration_is_a_narration() {
    assert_eq!(message_type(Kind::Event), MessageType::Narration);
}

#[test]
fn message_type_of_an_impersonate_input_is_an_input() {
    assert_eq!(message_type(Kind::Impersonate), MessageType::Input);
}

#[test]
fn message_type_of_a_user_regen_is_an_input() {
    assert_eq!(message_type(Kind::UserRegen), MessageType::Input);
}

#[test]
fn maker_of_a_narration_is_the_narrator() {
    assert_eq!(maker(Kind::Narration), Maker::Narrator);
}

#[test]
fn maker_of_an_event_narration_is_the_narrator() {
    assert_eq!(maker(Kind::Event), Maker::Narrator);
}

#[test]
fn maker_of_an_impersonate_input_is_the_player() {
    assert_eq!(maker(Kind::Impersonate), Maker::Player);
}

#[test]
fn maker_of_a_user_regen_is_the_player() {
    assert_eq!(maker(Kind::UserRegen), Maker::Player);
}
