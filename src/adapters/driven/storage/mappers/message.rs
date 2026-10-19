//! [DOC: docs/diataxis/reference/storage.md]
//! Message mapper

use chrono::{DateTime, Utc};
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};

use crate::error::EngineError;
use crate::domain::model::message::{Message, Swipe};
use crate::domain::model::state::message_types::{MessageKind, MessageType};
use crate::adapters::driven::storage::models::message::DbMessage;
use crate::adapters::driven::storage::models::swipe::DbSwipe;
use crate::adapters::driven::storage::utils::from_json;

/// The `message_type` and `kind` columns hold JSON, so a read parses them in the
/// row, which keeps one message field out of every SELECT's tuple.
impl FromSql for MessageType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let json = value.as_str()?;
        serde_json::from_str(json).map_err(|e| FromSqlError::Other(Box::new(e)))
    }
}

impl FromSql for MessageKind {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let json = value.as_str()?;
        serde_json::from_str(json).map_err(|e| FromSqlError::Other(Box::new(e)))
    }
}

impl TryFrom<&DbSwipe> for Swipe {
    type Error = EngineError;

    fn try_from(db_swipe: &DbSwipe) -> Result<Self, Self::Error> {
        Ok(Swipe {
            text: db_swipe.text.clone(),
            room_report: db_swipe.room_report.clone(),
            event_header: db_swipe.event_header.clone(),
            impersonated: db_swipe.impersonated != 0,
            guide_text: db_swipe.guide_text.clone(),
            impersonate_direction: db_swipe.impersonate_direction.clone(),
            state: from_json("swipe state", &db_swipe.state_json)?,
            maker: from_json("swipe maker", &db_swipe.maker_json)?,
        })
    }
}

impl TryFrom<(&DbMessage, &[DbSwipe])> for Message {
    type Error = EngineError;

    fn try_from((db, swipes): (&DbMessage, &[DbSwipe])) -> Result<Self, Self::Error> {
        let message_type: MessageType =
            serde_json::from_str(&db.message_type_json).map_err(|e| {
                EngineError::Config(format!("Failed to parse message message_type: {e}"))
            })?;
        let kind = from_json("message kind", &db.kind_json)?;
        let timestamp = DateTime::parse_from_rfc3339(&db.timestamp)
            .map_err(|e| EngineError::Config(format!("Failed to parse message timestamp: {e}")))?
            .with_timezone(&Utc);

        let mut message = Message::from_db(
            db.id as u64,
            message_type,
            kind,
            timestamp,
            db.active_swipe_index as usize,
        );

        for db_swipe in swipes {
            message.swipes.push(Swipe::try_from(db_swipe)?);
        }

        if !message.swipes.is_empty() {
            message.ensure_valid_swipe_index();
        }

        Ok(message)
    }
}
