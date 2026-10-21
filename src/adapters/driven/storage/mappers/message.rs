//! [DOC: docs/diataxis/reference/storage.md]
//! Message mapper

use chrono::{DateTime, Utc};
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};

use crate::error::{EngineError, InternalError};
use crate::domain::model::message::{Message, Swipe};
use crate::domain::model::state::message_types::{MessageKind, MessageType};
use crate::adapters::driven::storage::models::message::DbMessage;
use crate::adapters::driven::storage::models::swipe::DbSwipe;

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
            event_header: db_swipe.event_header.clone(),
            impersonated: db_swipe.impersonated != 0,
            guide_text: db_swipe.guide_text.clone(),
            impersonate_direction: db_swipe.impersonate_direction.clone(),
            state: serde_json::from_str(&db_swipe.state_json)?,
            maker: serde_json::from_str(&db_swipe.maker_json)?,
        })
    }
}

impl TryFrom<(&DbMessage, &[DbSwipe])> for Message {
    type Error = EngineError;

    fn try_from((db, swipes): (&DbMessage, &[DbSwipe])) -> Result<Self, Self::Error> {
        let message_type: MessageType = serde_json::from_str(&db.message_type_json)?;
        let kind = serde_json::from_str(&db.kind_json)?;
        let timestamp = DateTime::parse_from_rfc3339(&db.timestamp)
            .map_err(|e| EngineError::Parse(format!("Failed to parse message timestamp: {e}")))?
            .with_timezone(&Utc);

        let mut message = Message::from_db(
            db.id as u64,
            message_type,
            kind,
            timestamp,
            db.active_swipe_index as usize,
        );

        if swipes.is_empty() {
            return Err(EngineError::Internal(InternalError::new(format!(
                "message {} has no swipe",
                db.id
            ))));
        }
        for db_swipe in swipes {
            message.swipes.push(Swipe::try_from(db_swipe)?);
        }
        message.ensure_valid_swipe_index();

        Ok(message)
    }
}
