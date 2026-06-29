use crate::domain::model::message::Message;
use crate::domain::model::state::message_types::MessageType;
use crate::adapters::driven::storage::mappers::message::{
    db_message_to_model, model_message_to_db, model_swipes_to_db,
};

#[test]
fn test_message_roundtrip() {
    let mut original = Message::new(
        Some("System".to_string()),
        "Hello world",
        MessageType::System,
        Some("Room A".to_string()),
        None,
    );
    original.set_snapshot_id(Some(3));
    original.swipes = vec![crate::domain::model::message::Swipe {
        text: "Hello world".to_string(),
        snapshot_id: Some(3),
        location_header: Some("Room A".to_string()),
        event_header: None,
    }];
    let db = model_message_to_db(&original, 1).unwrap();
    let swipes = model_swipes_to_db(&original);
    let back = db_message_to_model(&db, &swipes).unwrap();

    assert_eq!(original.id, back.id);
    assert_eq!(original.sender, back.sender);
    assert_eq!(original.text(), back.text());
    assert_eq!(original.message_type, back.message_type);
    assert_eq!(original.timestamp, back.timestamp);
    assert_eq!(original.location_header(), back.location_header());
    assert_eq!(original.event_header(), back.event_header());
    assert_eq!(original.snapshot_id(), back.snapshot_id());
    assert_eq!(db.game_id, 1);
}

#[test]
fn test_message_unpersisted_roundtrip() {
    let mut original = Message::new(
        None,
        "Input text",
        MessageType::Input,
        None,
        Some("Event".to_string()),
    );
    original.swipes = vec![crate::domain::model::message::Swipe {
        text: "Input text".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: Some("Event".to_string()),
    }];
    let db = model_message_to_db(&original, 2).unwrap();
    let swipes = model_swipes_to_db(&original);
    let back = db_message_to_model(&db, &swipes).unwrap();

    assert_eq!(back.id, 0);
    assert!(back.sender.is_none());
    assert_eq!(back.message_type, MessageType::Input);
    assert_eq!(db.game_id, 2);
}

#[test]
fn test_message_log_type_json_serialization() {
    let mut msg = Message::new(None, "test", MessageType::Dialogue, None, None);
    msg.swipes = vec![crate::domain::model::message::Swipe {
        text: "test".to_string(),
        snapshot_id: None,
        location_header: None,
        event_header: None,
    }];
    let db = model_message_to_db(&msg, 1).unwrap();
    let _swipes = model_swipes_to_db(&msg);

    assert_eq!(db.message_type_json, "\"Dialogue\"");
}

#[test]
fn test_active_swipe_index_out_of_bounds_fallback() {
    let mut original = Message::new(
        Some("Narrator".to_string()),
        "First swipe",
        MessageType::Narration,
        Some("Room A".to_string()),
        None,
    );
    original.set_snapshot_id(Some(1));
    original.swipes = vec![
        crate::domain::model::message::Swipe {
            text: "First swipe".to_string(),
            snapshot_id: Some(1),
            location_header: Some("Room A".to_string()),
            event_header: None,
        },
        crate::domain::model::message::Swipe {
            text: "Second swipe".to_string(),
            snapshot_id: Some(2),
            location_header: Some("Room B".to_string()),
            event_header: Some("Event B".to_string()),
        },
    ];
    let db = model_message_to_db(&original, 1).unwrap();
    let swipes = model_swipes_to_db(&original);

    // Simulate stale active_swipe_index by mutating the db row
    let mut db_stale = db;
    db_stale.active_swipe_index = 99;

    let back = db_message_to_model(&db_stale, &swipes).unwrap();
    assert_eq!(back.text(), "First swipe");
    assert_eq!(back.location_header(), Some("Room A"));
    assert_eq!(back.snapshot_id(), Some(1));
}
