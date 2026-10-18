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
            snapshot_id: db_swipe.snapshot_id.map(|id| id as u64),
            room_report: db_swipe.room_report.clone(),
            event_header: db_swipe.event_header.clone(),
            impersonated: db_swipe.impersonated != 0,
            guide_text: db_swipe.guide_text.clone(),
            impersonate_direction: db_swipe.impersonate_direction.clone(),
            state: from_json("swipe state", db_swipe.state_json.as_deref())?,
            maker: from_json("swipe maker", db_swipe.maker_json.as_deref())?,
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
        let kind = from_json("message kind", db.kind_json.as_deref())?;
        let timestamp = DateTime::parse_from_rfc3339(&db.timestamp)
            .map_err(|e| EngineError::Config(format!("Failed to parse message timestamp: {e}")))?
            .with_timezone(&Utc);

        let mut message = Message::from_db(
            db.id as u64,
            message_type,
            kind,
            timestamp,
            db.active_swipe_index as usize,
            db.is_deleted != 0,
        );

        for db_swipe in swipes {
            message.swipes.push(Swipe::try_from(db_swipe)?);
        }

        if !message.swipes.is_empty() {
            let fallback_applied = message.ensure_valid_swipe_index();
            if fallback_applied {
                message.set_active_swipe(0);
            } else {
                message.set_active_swipe(message.active_swipe_index);
            }
        }

        Ok(message)
    }
}

impl TryFrom<(&Message, i64)> for DbMessage {
    type Error = EngineError;

    fn try_from((msg, game_id): (&Message, i64)) -> Result<Self, Self::Error> {
        let message_type_json = serde_json::to_string(&msg.message_type).map_err(|e| {
            EngineError::Config(format!("Failed to serialize message message_type: {e}"))
        })?;
        let kind_json =
            crate::adapters::driven::storage::utils::to_json("message kind", msg.kind.as_ref())?;

        Ok(DbMessage {
            id: msg.id as i64,
            game_id,
            message_type_json,
            kind_json,
            timestamp: msg.timestamp.to_rfc3339(),
            active_swipe_index: msg.active_swipe_index as i64,
            is_deleted: if msg.is_deleted { 1 } else { 0 },
        })
    }
}
