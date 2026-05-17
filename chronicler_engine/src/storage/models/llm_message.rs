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
