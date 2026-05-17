pub struct DbMessage {
    pub id: i64,
    pub game_id: i64,
    pub sender: Option<String>,
    pub text: String,
    pub log_type_json: String,
    pub timestamp: String,
    pub location_header: Option<String>,
    pub event_header: Option<String>,
    pub snapshot_id: Option<i64>,
}
