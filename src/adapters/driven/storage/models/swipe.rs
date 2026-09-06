//! [DOC: docs/diataxis/reference/storage.md]
//! Database row struct for the `swipes` table

pub struct DbSwipe {
    pub id: i64,
    pub message_id: i64,
    pub swipe_index: i64,
    pub text: String,
    pub snapshot_id: Option<i64>,
    pub location_header: Option<String>,
    pub event_header: Option<String>,
    pub replay: Option<String>,
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
            replay: row.get(7)?,
        })
    }
}
