/// Database row for `settings` table (singleton, id=1).
pub struct DbSettings {
    pub id: i64,
    pub connections: String, // JSON: Vec<Connection>
    pub narration_connection_id: String,
    pub quantifier_connection_id: String,
    pub response_length: String,
    pub text_check: String, // JSON: TextCheckSettings
    pub agents: String,     // JSON: Vec<AgentConfig>
    pub active_system_prompt_preset_id: String,
    pub active_quantifier_prompt_preset_id: String,
    pub created_at: String,
    pub updated_at: String,
}

impl DbSettings {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(DbSettings {
            id: row.get(0)?,
            connections: row.get(1)?,
            narration_connection_id: row.get(2)?,
            quantifier_connection_id: row.get(3)?,
            response_length: row.get(4)?,
            text_check: row.get(5)?,
            agents: row.get(6)?,
            active_system_prompt_preset_id: row.get(7)?,
            active_quantifier_prompt_preset_id: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    }
}
