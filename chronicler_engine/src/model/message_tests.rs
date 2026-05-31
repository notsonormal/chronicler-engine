use crate::model::message::Message;
use crate::model::state::MessageType;

#[test]
fn test_message_new_sets_fields() {
    let msg = Message::new(
        Some("Player".to_string()),
        "Hello world",
        MessageType::Input,
        Some("Location".to_string()),
        Some("Event".to_string()),
    );

    assert_eq!(msg.id, 0);
    assert_eq!(msg.sender, Some("Player".to_string()));
    assert_eq!(msg.text(), "Hello world");
    assert_eq!(msg.message_type, MessageType::Input);
    assert_eq!(msg.location_header(), Some("Location"));
    assert_eq!(msg.event_header(), Some("Event"));
}

#[test]
fn test_message_text_roundtrip() {
    let mut msg = Message::new(None, "Original", MessageType::Narration, None, None);
    msg.update_active_swipe_text("Updated".to_string());
    assert_eq!(msg.text(), "Updated");
}

#[test]
fn test_message_new_generates_timestamp() {
    let before = chrono::Utc::now();
    let msg = Message::new(None, "Hello", MessageType::Narration, None, None);
    let after = chrono::Utc::now();

    assert!(msg.timestamp >= before);
    assert!(msg.timestamp <= after);
}
#[test]
fn test_message_set_event_header() {
    let mut msg = Message::new(
        None,
        "Test narration",
        MessageType::Narration,
        Some("Test Location".to_string()),
        Some("Original Event".to_string()),
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
        Some("AI".to_string()),
        MessageType::Narration,
        timestamp,
        0,
        false,
    );
    assert_eq!(msg.id, 42);
    assert_eq!(msg.sender, Some("AI".to_string()));
    assert_eq!(msg.message_type, MessageType::Narration);
    assert_eq!(msg.timestamp, timestamp);
    assert_eq!(msg.active_swipe_index, 0);
    assert!(!msg.is_deleted);
    assert_eq!(msg.text(), "");
    assert_eq!(msg.swipe_count(), 0);
}
#[test]
fn test_message_from_db_with_deleted() {
    let timestamp = chrono::Utc::now();
    let msg = Message::from_db(99, None, MessageType::Input, timestamp, 2, true);
    assert_eq!(msg.id, 99);
    assert_eq!(msg.sender, None);
    assert_eq!(msg.message_type, MessageType::Input);
    assert_eq!(msg.active_swipe_index, 2);
    assert!(msg.is_deleted);
}
