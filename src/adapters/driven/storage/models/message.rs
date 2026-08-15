//! [DOC: docs/diataxis/reference/storage.md]
//! Database row struct for the `messages` table

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
