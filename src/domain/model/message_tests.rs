use crate::domain::model::message::{Maker, Message, Swipe};
use crate::domain::model::state::message_types::{MessageKind, MessageType};
use crate::test_support::TestSwipeState;

fn swipe(text: &str) -> Swipe {
    Swipe::new(text, Maker::Narrator, TestSwipeState::in_room("room1"))
}

fn message(text: &str, message_type: MessageType) -> Message {
    Message::new(message_type, MessageKind::Narration, swipe(text))
}

fn message_with_headers(
    text: &str,
    message_type: MessageType,
    room_report: Option<&str>,
    event_header: Option<&str>,
) -> Message {
    let mut message = message(text, message_type);
    message.swipes[0].room_report = room_report.map(str::to_string);
    message.swipes[0].event_header = event_header.map(str::to_string);
    message
}

#[test]
fn test_message_new_sets_fields() {
    let msg = message_with_headers(
        "Hello world",
        MessageType::Input,
        Some("Location"),
        Some("Event"),
    );

    assert_eq!(msg.id, 0);
    assert_eq!(msg.text(), "Hello world");
    assert_eq!(msg.message_type, MessageType::Input);
    assert_eq!(msg.room_report(), Some("Location"));
    assert_eq!(msg.event_header(), Some("Event"));
}

#[test]
fn test_message_text_roundtrip() {
    let mut msg = message("Original", MessageType::Narration);
    msg.update_active_swipe_text("Updated".to_string());
    assert_eq!(msg.text(), "Updated");
}

#[test]
fn test_message_new_generates_timestamp() {
    let before = chrono::Utc::now();
    let msg = message("Hello", MessageType::Narration);
    let after = chrono::Utc::now();

    assert!(msg.timestamp >= before);
    assert!(msg.timestamp <= after);
}
#[test]
fn test_message_set_event_header() {
    let mut msg = message_with_headers(
        "Test narration",
        MessageType::Narration,
        Some("Test Location"),
        Some("Original Event"),
    );
    assert_eq!(msg.event_header(), Some("Original Event"));

    msg.set_event_header(Some("New Event".to_string()));
    assert_eq!(msg.event_header(), Some("New Event"));

    msg.set_event_header(None);
    assert_eq!(msg.event_header(), None);
}
#[test]
fn test_message_from_db() {
    let timestamp = chrono::Utc::now();
    let msg = Message::from_db(
        42,
        MessageType::Narration,
        MessageKind::Narration,
        timestamp,
        0,
    );
    assert_eq!(msg.id, 42);
    assert_eq!(msg.message_type, MessageType::Narration);
    assert_eq!(msg.timestamp, timestamp);
    assert_eq!(msg.active_swipe_index, 0);
    assert_eq!(msg.text(), "");
    assert_eq!(msg.swipe_count(), 0);
}
#[test]
fn test_message_swipe_fields_roundtrip() {
    let mut msg = message_with_headers(
        "Original text",
        MessageType::Narration,
        Some("Room A"),
        Some("EventX"),
    );

    msg.swipes.push(Swipe {
        room_report: Some("Room B".to_string()),
        event_header: Some("EventY".to_string()),
        ..swipe("Swiped text")
    });

    assert_eq!(msg.text(), "Original text");
    assert_eq!(msg.room_report(), Some("Room A"));
    assert_eq!(msg.event_header(), Some("EventX"));

    msg.set_active_swipe(1);
    assert_eq!(msg.text(), "Swiped text");
    assert_eq!(msg.room_report(), Some("Room B"));
    assert_eq!(msg.event_header(), Some("EventY"));

    msg.set_active_swipe(0);
    msg.update_active_swipe_text("Updated original");
    assert_eq!(msg.text(), "Updated original");
    assert_eq!(msg.swipes[0].text, "Updated original");
}

#[test]
fn test_is_guided_truth_table() {
    let plain = message("n", MessageType::Narration);
    assert!(!plain.is_guided());
    assert!(!plain.impersonated());
    assert_eq!(plain.guide_text(), None);
    assert_eq!(plain.impersonate_direction(), None);

    let mut guided = message("n", MessageType::Narration);
    guided.swipes[0].guide_text = Some("make it ominous".to_string());
    assert!(guided.is_guided());
    assert!(!guided.impersonated());
    assert_eq!(guided.guide_text(), Some("make it ominous"));
    assert_eq!(guided.impersonate_direction(), None);

    let mut bare_impersonate = message("i", MessageType::Input);
    bare_impersonate.swipes[0].impersonated = true;
    assert!(!bare_impersonate.is_guided());
    assert!(bare_impersonate.impersonated());
    assert_eq!(bare_impersonate.impersonate_direction(), None);

    let mut directed_impersonate = message("i", MessageType::Input);
    directed_impersonate.swipes[0].impersonated = true;
    directed_impersonate.swipes[0].impersonate_direction = Some("as the player".to_string());
    assert!(!directed_impersonate.is_guided());
    assert!(directed_impersonate.impersonated());
    assert_eq!(
        directed_impersonate.impersonate_direction(),
        Some("as the player")
    );
}

#[test]
fn test_set_stored_inputs_on_active_swipe() {
    let mut msg = message("n", MessageType::Narration);
    msg.set_stored_inputs(false, Some("steer".to_string()), None);
    assert!(msg.is_guided());

    msg.set_stored_inputs(true, None, Some("as the player".to_string()));
    assert!(msg.impersonated());
    assert!(!msg.is_guided());

    msg.set_stored_inputs(false, None, None);
    assert!(!msg.impersonated());
    assert_eq!(msg.guide_text(), None);
}
