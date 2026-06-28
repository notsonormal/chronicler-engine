use crate::model::message::{Message, Swipe};
use crate::model::state::message_types::MessageType;

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

#[test]
fn test_message_swipe_fields_roundtrip() {
    // Create message with multiple swipes
    let mut msg = Message::new(
        Some("AI".to_string()),
        "Original text",
        MessageType::Narration,
        Some("Room A".to_string()),
        Some("EventX".to_string()),
    );

    // Add second swipe with different values
    msg.swipes.push(Swipe {
        text: "Swiped text".to_string(),
        snapshot_id: None,
        location_header: Some("Room B".to_string()),
        event_header: Some("EventY".to_string()),
    });

    // Initially at index 0
    assert_eq!(msg.text(), "Original text");
    assert_eq!(msg.location_header(), Some("Room A"));
    assert_eq!(msg.event_header(), Some("EventX"));
    assert_eq!(msg.snapshot_id(), None);

    // Set snapshot_id on first swipe
    msg.set_snapshot_id(Some(42));
    assert_eq!(msg.snapshot_id(), Some(42));
    assert_eq!(msg.swipes[0].snapshot_id, Some(42));

    // Switch to second swipe
    msg.set_active_swipe(1);
    assert_eq!(msg.text(), "Swiped text");
    assert_eq!(msg.location_header(), Some("Room B"));
    assert_eq!(msg.event_header(), Some("EventY"));
    assert_eq!(msg.snapshot_id(), None); // Second swipe has no snapshot_id yet

    // Set snapshot_id on second swipe
    msg.set_snapshot_id(Some(99));
    assert_eq!(msg.snapshot_id(), Some(99));
    assert_eq!(msg.swipes[1].snapshot_id, Some(99));

    // Switch back to first swipe - should have original snapshot_id
    msg.set_active_swipe(0);
    assert_eq!(msg.snapshot_id(), Some(42));

    // Update text on active swipe
    msg.update_active_swipe_text("Updated original");
    assert_eq!(msg.text(), "Updated original");
    assert_eq!(msg.swipes[0].text, "Updated original");
}

#[test]
fn test_message_set_snapshot_id_writes_active_swipe() {
    let mut msg = Message::new(None, "Text", MessageType::Narration, None, None);
    msg.swipes.push(Swipe {
        text: "Alt".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    });
    msg.set_active_swipe(1);

    msg.set_snapshot_id(Some(123));

    // Must write to active swipe, not separate field
    assert_eq!(msg.swipes[1].snapshot_id, Some(123));
    assert_eq!(msg.swipes[0].snapshot_id, None);
    assert_eq!(msg.snapshot_id(), Some(123));
}
