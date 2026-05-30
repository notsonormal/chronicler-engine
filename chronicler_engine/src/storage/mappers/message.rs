use chrono::{DateTime, Utc};

use crate::error::EngineError;
use crate::model::message::{Message, Swipe};
use crate::storage::models::message::{DbMessage, DbSwipe};

/// [DOC: docs/architecture/system.md]
pub fn db_message_to_model(db: &DbMessage, swipes: &[DbSwipe]) -> Result<Message, EngineError> {
    let message_type = serde_json::from_str(&db.message_type_json)
        .map_err(|e| EngineError::Config(format!("Failed to parse message message_type: {e}")))?;
    let timestamp = DateTime::parse_from_rfc3339(&db.timestamp)
        .map_err(|e| EngineError::Config(format!("Failed to parse message timestamp: {e}")))?
        .with_timezone(&Utc);

    let mut message = Message {
        id: db.id as u64,
        sender: db.sender.clone(),
        text: String::new(),
        message_type,
        timestamp,
        location_header: None,
        event_header: None,
        snapshot_id: None,
        active_swipe_index: db.active_swipe_index as usize,
        swipes: Vec::new(),
        is_deleted: db.is_deleted != 0,
    };

    for db_swipe in swipes {
        message.swipes.push(Swipe {
            text: db_swipe.text.clone(),
            snapshot_id: db_swipe.snapshot_id.map(|id| id as u64),
            location_header: db_swipe.location_header.clone(),
            event_header: db_swipe.event_header.clone(),
        });
    }

    let fallback_to_first = message.active_swipe_index >= message.swipes.len();
    if fallback_to_first {
        log::warn!(
            "active_swipe_index {} out of bounds for message {}, falling back to first swipe",
            message.active_swipe_index,
            message.id
        );
    }
    let idx = if fallback_to_first {
        0
    } else {
        message.active_swipe_index
    };
    if let Some(swipe) = message.swipes.get(idx) {
        message.text = swipe.text.clone();
        message.location_header = swipe.location_header.clone();
        message.event_header = swipe.event_header.clone();
        message.snapshot_id = swipe.snapshot_id;
    }

    Ok(message)
}

pub fn model_message_to_db(msg: &Message, game_id: i64) -> Result<DbMessage, EngineError> {
    let message_type_json = serde_json::to_string(&msg.message_type)
        .map_err(|e| EngineError::Config(format!("Failed to serialize message message_type: {e}")))?;

    Ok(DbMessage {
        id: msg.id as i64,
        game_id,
        sender: msg.sender.clone(),
        message_type_json,
        timestamp: msg.timestamp.to_rfc3339(),
        active_swipe_index: msg.active_swipe_index as i64,
        is_deleted: if msg.is_deleted { 1 } else { 0 },
    })
}

pub fn model_swipes_to_db(msg: &Message) -> Vec<DbSwipe> {
    msg.swipes
        .iter()
        .enumerate()
        .map(|(idx, swipe)| DbSwipe {
            id: 0,
            message_id: msg.id as i64,
            swipe_index: idx as i64,
            text: swipe.text.clone(),
            snapshot_id: swipe.snapshot_id.map(|id| id as i64),
            location_header: swipe.location_header.clone(),
            event_header: swipe.event_header.clone(),
        })
        .collect()
}