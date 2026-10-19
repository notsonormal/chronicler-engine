//! [DOC: docs/diataxis/reference/storage.md]
//! Database row struct for the `swipes` table

pub struct DbSwipe {
    pub id: i64,
    pub message_id: i64,
    pub swipe_index: i64,
    pub text: String,
    pub room_report: Option<String>,
    pub event_header: Option<String>,
    pub impersonated: i64,
    pub guide_text: Option<String>,
    pub impersonate_direction: Option<String>,
    pub state_json: String,
    pub maker_json: String,
}

impl DbSwipe {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(DbSwipe {
            id: row.get(0)?,
            message_id: row.get(1)?,
            swipe_index: row.get(2)?,
            text: row.get(3)?,
            room_report: row.get(4)?,
            event_header: row.get(5)?,
            impersonated: row.get(6)?,
            guide_text: row.get(7)?,
            impersonate_direction: row.get(8)?,
            state_json: row.get(9)?,
            maker_json: row.get(10)?,
        })
    }
}
