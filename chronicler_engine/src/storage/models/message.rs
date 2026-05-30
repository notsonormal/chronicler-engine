pub struct DbMessage {
    pub id: i64,
    pub game_id: i64,
    pub sender: Option<String>,
    pub message_type_json: String,
    pub timestamp: String,
    pub active_swipe_index: i64,
    pub is_deleted: i64,
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
