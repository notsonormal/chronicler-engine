use chrono::Utc;

use crate::model::message::Message;
use crate::model::state::LogType;
use crate::storage::mappers::message::{
    db_message_to_model, model_message_to_db, model_swipes_to_db,
};

#[test]
fn test_message_roundtrip() {
    let original = Message {
        id: 7,
        sender: Some("System".to_string()),
        text: "Hello world".to_string(),
        log_type: LogType::System,
        timestamp: Utc::now(),
        location_header: Some("Room A".to_string()),
        event_header: None,
        snapshot_id: Some(3),
        active_swipe_index: 0,
        swipes: vec![crate::model::message::Swipe {
            text: "Hello world".to_string(),
            snapshot_id: Some(3),
            location_header: Some("Room A".to_string()),
            event_header: None,
        }],
        is_deleted: false,
    };
    let db = model_message_to_db(&original, 1).unwrap();
    let swipes = model_swipes_to_db(&original);
    let back = db_message_to_model(&db, &swipes).unwrap();

    assert_eq!(original.id, back.id);
    assert_eq!(original.sender, back.sender);
    assert_eq!(original.text, back.text);
    assert_eq!(original.log_type, back.log_type);
    assert_eq!(original.timestamp, back.timestamp);
    assert_eq!(original.location_header, back.location_header);
    assert_eq!(original.event_header, back.event_header);
    assert_eq!(original.snapshot_id, back.snapshot_id);
    assert_eq!(db.game_id, 1);
}

#[test]
fn test_message_unpersisted_roundtrip() {
    let original = Message {
        id: 0,
        sender: None,
        text: "Input text".to_string(),
        log_type: LogType::Input,
        timestamp: Utc::now(),
        location_header: None,
        event_header: Some("Event".to_string()),
        snapshot_id: None,
        active_swipe_index: 0,
        swipes: vec![crate::model::message::Swipe {
            text: "Input text".to_string(),
            snapshot_id: None,
            location_header: None,
            event_header: Some("Event".to_string()),
        }],
        is_deleted: false,
    };
    let db = model_message_to_db(&original, 2).unwrap();
    let swipes = model_swipes_to_db(&original);
    let back = db_message_to_model(&db, &swipes).unwrap();

    assert_eq!(back.id, 0);
    assert!(back.sender.is_none());
    assert_eq!(back.log_type, LogType::Input);
    assert_eq!(db.game_id, 2);
}

#[test]
fn test_message_log_type_json_serialization() {
    let msg = Message {
        id: 1,
        sender: None,
        text: "test".to_string(),
        log_type: LogType::Dialogue,
        timestamp: Utc::now(),
        location_header: None,
        event_header: None,
        snapshot_id: None,
        active_swipe_index: 0,
        swipes: vec![crate::model::message::Swipe {
            text: "test".to_string(),
            snapshot_id: None,
            location_header: None,
            event_header: None,
        }],
        is_deleted: false,
    };
    let db = model_message_to_db(&msg, 1).unwrap();
    let _swipes = model_swipes_to_db(&msg);

    assert_eq!(db.log_type_json, "\"Dialogue\"");
}
