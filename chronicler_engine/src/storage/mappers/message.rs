use chrono::{DateTime, Utc};

use crate::error::EngineError;
use crate::model::message::Message;
use crate::storage::models::message::DbMessage;

impl TryFrom<&DbMessage> for Message {
    type Error = EngineError;

    fn try_from(db: &DbMessage) -> Result<Self, Self::Error> {
        let log_type = serde_json::from_str(&db.log_type_json)
            .map_err(|e| EngineError::Config(format!("Failed to parse message log_type: {e}")))?;
        let timestamp = DateTime::parse_from_rfc3339(&db.timestamp)
            .map_err(|e| EngineError::Config(format!("Failed to parse message timestamp: {e}")))?
            .with_timezone(&Utc);

        Ok(Message {
            id: db.id as u64,
            sender: db.sender.clone(),
            text: db.text.clone(),
            log_type,
            timestamp,
            location_header: db.location_header.clone(),
            event_header: db.event_header.clone(),
            snapshot_id: db.snapshot_id.map(|id| id as u64),
        })
    }
}

pub fn message_to_db(msg: &Message, game_id: i64) -> Result<DbMessage, EngineError> {
    let log_type_json = serde_json::to_string(&msg.log_type)
        .map_err(|e| EngineError::Config(format!("Failed to serialize message log_type: {e}")))?;

    Ok(DbMessage {
        id: msg.id as i64,
        game_id,
        sender: msg.sender.clone(),
        text: msg.text.clone(),
        log_type_json,
        timestamp: msg.timestamp.to_rfc3339(),
        location_header: msg.location_header.clone(),
        event_header: msg.event_header.clone(),
        snapshot_id: msg.snapshot_id.map(|id| id as i64),
    })
}
