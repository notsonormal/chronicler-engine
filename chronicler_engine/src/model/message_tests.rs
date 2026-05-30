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
    assert_eq!(msg.text, "Hello world");
    assert_eq!(msg.message_type, MessageType::Input);
    assert_eq!(msg.location_header, Some("Location".to_string()));
    assert_eq!(msg.event_header, Some("Event".to_string()));
}

#[test]
fn test_message_text_roundtrip() {
    let mut msg = Message::new(
        None,
        "Original",
        MessageType::Narration,
        None,
        None,
    );
    msg.text = "Updated".to_string();
    assert_eq!(msg.text, "Updated");
}

#[test]
fn test_message_new_generates_timestamp() {
    let before = chrono::Utc::now();
    let msg = Message::new(None, "Hello", MessageType::Narration, None, None);
    let after = chrono::Utc::now();

    assert!(msg.timestamp >= before);
    assert!(msg.timestamp <= after);
}
