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

#[derive(Debug, Default)]
pub struct LlmMessageBuilder {
    agent_name: String,
    backend_name: String,
    model_name: String,
    system_prompt: String,
    user_prompt: String,
    raw_request_json: String,
    raw_response_json: String,
    parsed_response: String,
    error_message: Option<String>,
}

impl LlmMessageBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn agent_name(mut self, value: impl Into<String>) -> Self {
        self.agent_name = value.into();
        self
    }

    pub fn backend_name(mut self, value: impl Into<String>) -> Self {
        self.backend_name = value.into();
        self
    }

    pub fn model_name(mut self, value: impl Into<String>) -> Self {
        self.model_name = value.into();
        self
    }

    pub fn system_prompt(mut self, value: impl Into<String>) -> Self {
        self.system_prompt = value.into();
        self
    }

    pub fn user_prompt(mut self, value: impl Into<String>) -> Self {
        self.user_prompt = value.into();
        self
    }

    pub fn raw_request_json(mut self, value: impl Into<String>) -> Self {
        self.raw_request_json = value.into();
        self
    }

    pub fn raw_response_json(mut self, value: impl Into<String>) -> Self {
        self.raw_response_json = value.into();
        self
    }

    pub fn parsed_response(mut self, value: impl Into<String>) -> Self {
        self.parsed_response = value.into();
        self
    }

    pub fn error_message(mut self, value: Option<impl Into<String>>) -> Self {
        self.error_message = value.map(Into::into);
        self
    }

    pub fn build(self) -> LlmMessage {
        LlmMessage {
            id: 0,
            agent_name: self.agent_name,
            backend_name: self.backend_name,
            model_name: self.model_name,
            system_prompt: self.system_prompt,
            user_prompt: self.user_prompt,
            raw_request_json: self.raw_request_json,
            raw_response_json: self.raw_response_json,
            parsed_response: self.parsed_response,
            error_message: self.error_message,
            created_at: Utc::now(),
        }
    }
}
