use crate::domain::model::message::Message;
use crate::domain::model::state::message_types::MessageType;
use crate::adapters::driven::storage::models::message::DbMessage;
use crate::adapters::driven::storage::models::swipe::DbSwipe;

/// Mirrors the row shape `Storage::insert_swipe` writes, so the roundtrip
/// exercises what production stores.
fn to_db_swipes(msg: &Message) -> Vec<DbSwipe> {
    msg.swipes
        .iter()
        .enumerate()
        .map(|(idx, swipe)| DbSwipe {
            id: 0,
            message_id: msg.id as i64,
            swipe_index: idx as i64,
            text: swipe.text.clone(),
            snapshot_id: swipe.snapshot_id.map(|id| id as i64),
            room_report: swipe.room_report.clone(),
            event_header: swipe.event_header.clone(),
            impersonated: if swipe.impersonated { 1 } else { 0 },
            guide_text: swipe.guide_text.clone(),
            impersonate_direction: swipe.impersonate_direction.clone(),
            state_json: crate::adapters::driven::storage::utils::to_json(
                "swipe state",
                swipe.state.as_ref(),
            )
            .unwrap(),
            maker_json: crate::adapters::driven::storage::utils::to_json(
                "swipe maker",
                swipe.maker.as_ref(),
            )
            .unwrap(),
        })
        .collect()
}

#[test]
fn test_message_roundtrip() {
    let mut original = Message::new(
        "Hello world",
        MessageType::System,
        Some("Room A".to_string()),
        None,
    );
    original.set_snapshot_id(Some(3));
    original.swipes = vec![crate::domain::model::message::Swipe {
        text: "Hello world".to_string(),
        snapshot_id: Some(3),
        room_report: Some("Room A".to_string()),
        event_header: None,
        impersonated: false,
        guide_text: None,
        impersonate_direction: None,
        state: None,
        maker: None,
    }];
    let db = DbMessage::try_from((&original, 1)).unwrap();
    let swipes = to_db_swipes(&original);
    let back = Message::try_from((&db, &swipes[..])).unwrap();

    assert_eq!(original.id, back.id);
    assert_eq!(original.text(), back.text());
    assert_eq!(original.message_type, back.message_type);
    assert_eq!(original.timestamp, back.timestamp);
    assert_eq!(original.room_report(), back.room_report());
    assert_eq!(original.event_header(), back.event_header());
    assert_eq!(original.snapshot_id(), back.snapshot_id());
    assert_eq!(db.game_id, 1);
}

#[test]
fn test_message_unpersisted_roundtrip() {
    let mut original = Message::new(
        "Input text",
        MessageType::Input,
        None,
        Some("Event".to_string()),
    );
    original.swipes = vec![crate::domain::model::message::Swipe {
        text: "Input text".to_string(),
        snapshot_id: None,
        room_report: None,
        event_header: Some("Event".to_string()),
        impersonated: false,
        guide_text: None,
        impersonate_direction: None,
        state: None,
        maker: None,
    }];
    let db = DbMessage::try_from((&original, 2)).unwrap();
    let swipes = to_db_swipes(&original);
    let back = Message::try_from((&db, &swipes[..])).unwrap();

    assert_eq!(back.id, 0);
    assert_eq!(back.message_type, MessageType::Input);
    assert_eq!(db.game_id, 2);
}

#[test]
fn test_message_log_type_json_serialization() {
    let mut msg = Message::new("test", MessageType::Input, None, None);
    msg.swipes = vec![crate::domain::model::message::Swipe {
        text: "test".to_string(),
        snapshot_id: None,
        room_report: None,
        event_header: None,
        impersonated: false,
        guide_text: None,
        impersonate_direction: None,
        state: None,
        maker: None,
    }];
    let db = DbMessage::try_from((&msg, 1)).unwrap();
    let _swipes = to_db_swipes(&msg);

    assert_eq!(db.message_type_json, "\"Input\"");
}

#[test]
fn test_active_swipe_index_out_of_bounds_fallback() {
    let mut original = Message::new(
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
            room_report: Some("Room A".to_string()),
            event_header: None,
            impersonated: false,
            guide_text: None,
            impersonate_direction: None,
            state: None,
            maker: None,
        },
        crate::domain::model::message::Swipe {
            text: "Second swipe".to_string(),
            snapshot_id: Some(2),
            room_report: Some("Room B".to_string()),
            event_header: Some("Event B".to_string()),
            impersonated: false,
            guide_text: None,
            impersonate_direction: None,
            state: None,
            maker: None,
        },
    ];
    let db = DbMessage::try_from((&original, 1)).unwrap();
    let swipes = to_db_swipes(&original);

    let mut db_stale = db;
    db_stale.active_swipe_index = 99;

    let back = Message::try_from((&db_stale, &swipes[..])).unwrap();
    assert_eq!(back.text(), "First swipe");
    assert_eq!(back.room_report(), Some("Room A"));
    assert_eq!(back.snapshot_id(), Some(1));
}

#[test]
fn test_swipe_stored_inputs_roundtrip() {
    let mut original = Message::new("I ask about the artifact.", MessageType::Input, None, None);
    original.swipes = vec![crate::domain::model::message::Swipe {
        text: "I ask about the artifact.".to_string(),
        snapshot_id: None,
        room_report: None,
        event_header: None,
        impersonated: true,
        guide_text: None,
        impersonate_direction: Some("as the player".to_string()),
        state: None,
        maker: None,
    }];

    let db = DbMessage::try_from((&original, 1)).unwrap();
    let swipes = to_db_swipes(&original);
    let back = Message::try_from((&db, &swipes[..])).unwrap();

    assert!(back.impersonated());
    assert_eq!(back.impersonate_direction(), Some("as the player"));
}
