use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct LlmMessage {
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
    pub created_at: DateTime<Utc>,
}

impl LlmMessage {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        agent_name: impl Into<String>,
        backend_name: impl Into<String>,
        model_name: impl Into<String>,
        system_prompt: impl Into<String>,
        user_prompt: impl Into<String>,
        raw_request_json: impl Into<String>,
        raw_response_json: impl Into<String>,
        parsed_response: impl Into<String>,
        error_message: Option<impl Into<String>>,
    ) -> Self {
        Self {
            id: 0,
            agent_name: agent_name.into(),
            backend_name: backend_name.into(),
            model_name: model_name.into(),
            system_prompt: system_prompt.into(),
            user_prompt: user_prompt.into(),
            raw_request_json: raw_request_json.into(),
            raw_response_json: raw_response_json.into(),
            parsed_response: parsed_response.into(),
            error_message: error_message.map(Into::into),
            created_at: Utc::now(),
        }
    }
}
