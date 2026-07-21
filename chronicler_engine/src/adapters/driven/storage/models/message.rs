//! [DOC: chronicler_engine/docs/diataxis/reference/storage.md]
//! Message database model

pub struct DbMessage {
    pub id: i64,
    pub game_id: i64,
    pub sender: Option<String>,
    pub message_type_json: String,
    pub timestamp: String,
    pub active_swipe_index: i64,
    pub is_deleted: i64,
}

impl DbMessage {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(DbMessage {
            id: row.get(0)?,
            game_id: row.get(1)?,
            sender: row.get(2)?,
            message_type_json: row.get(3)?,
            timestamp: row.get(4)?,
            active_swipe_index: row.get(5)?,
            is_deleted: 0,
        })
    }
}

pub struct DbSwipe {
    pub id: i64,
    pub message_id: i64,
    pub swipe_index: i64,
    pub text: String,
    pub snapshot_id: Option<i64>,
    pub location_header: Option<String>,
    pub event_header: Option<String>,
}

impl DbSwipe {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(DbSwipe {
            id: row.get(0)?,
            message_id: row.get(1)?,
            swipe_index: row.get(2)?,
            text: row.get(3)?,
            snapshot_id: row.get(4)?,
            location_header: row.get(5)?,
            event_header: row.get(6)?,
        })
    }
}
