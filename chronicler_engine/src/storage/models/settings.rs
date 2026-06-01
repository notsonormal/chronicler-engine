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
