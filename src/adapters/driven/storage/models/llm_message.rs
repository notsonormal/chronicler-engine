//! [DOC: docs/diataxis/reference/storage.md]
//! LLM message database model

pub struct DbLlmMessage {
    pub id: i64,
    pub agent_name: String,
    pub backend_name: String,
    pub model_name: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub raw_request_json: String,
    pub raw_response_json: String,
    pub parsed_response: String,
    pub error_message: Option<String>,
    pub created_at: String,
}

impl DbLlmMessage {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(DbLlmMessage {
            id: row.get(0)?,
            agent_name: row.get(1)?,
            backend_name: row.get(2)?,
            model_name: row.get(3)?,
            system_prompt: row.get(4)?,
            user_prompt: row.get(5)?,
            raw_request_json: row.get(6)?,
            raw_response_json: row.get(7)?,
            parsed_response: row.get(8)?,
            error_message: row.get(9)?,
            created_at: row.get(10)?,
        })
    }
}
