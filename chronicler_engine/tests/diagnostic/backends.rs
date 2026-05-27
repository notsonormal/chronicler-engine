use chronicler_engine::error::{EngineError, LlmFailure};
use chronicler_engine::narrative::llm::backend::{LlmBackend, LlmCallResult};

fn empty_llm_result(backend_name: &str, model_name: &str, agent_name: &str) -> LlmCallResult {
    LlmCallResult {
        text: String::new(),
        system_prompt: String::new(),
        user_prompt: String::new(),
        raw_request_json: String::new(),
        raw_response_json: String::new(),
        backend_name: backend_name.to_string(),
        model_name: model_name.to_string(),
        agent_name: agent_name.to_string(),
    }
}

pub struct HttpErrorBackend {
    pub status: u16,
    pub body: String,
}

impl HttpErrorBackend {
    pub fn unauthorized() -> Self {
        Self {
            status: 401,
            body: "Invalid API key".to_string(),
        }
    }
    pub fn rate_limited() -> Self {
        Self {
            status: 429,
            body: "Rate limit exceeded".to_string(),
        }
    }
    pub fn service_unavailable() -> Self {
        Self {
            status: 503,
            body: "Provider maintenance".to_string(),
        }
    }
}

impl LlmBackend for HttpErrorBackend {
    fn model(&self) -> &str {
        "mock"
    }
    fn narrate_continuation(
        &self,
        _agent_name: &str,
        _s: &str,
        _u: &str,
        _t: &str,
        _m: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Http {
            status: self.status,
            body: self.body.clone(),
        }))
    }
    fn complete(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Http {
            status: self.status,
            body: self.body.clone(),
        }))
    }
    fn name(&self) -> &str {
        "HttpError"
    }
}

pub struct NetworkErrorBackend {
    pub url: String,
    pub detail: String,
}

impl NetworkErrorBackend {
    pub fn connection_refused() -> Self {
        Self {
            url: "http://localhost:11434".to_string(),
            detail: "Connection refused".to_string(),
        }
    }
}

impl LlmBackend for NetworkErrorBackend {
    fn model(&self) -> &str {
        "mock"
    }
    fn narrate_continuation(
        &self,
        _agent_name: &str,
        _s: &str,
        _u: &str,
        _t: &str,
        _m: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Network {
            url: self.url.clone(),
            detail: self.detail.clone(),
        }))
    }
    fn complete(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Network {
            url: self.url.clone(),
            detail: self.detail.clone(),
        }))
    }
    fn name(&self) -> &str {
        "NetworkError"
    }
}

pub struct ParseErrorBackend {
    pub raw_response: String,
}

impl LlmBackend for ParseErrorBackend {
    fn model(&self) -> &str {
        "mock"
    }
    fn narrate_continuation(
        &self,
        _agent_name: &str,
        _s: &str,
        _u: &str,
        _t: &str,
        _m: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::ParseError {
            raw_response: self.raw_response.clone(),
            expected_format: "valid JSON",
        }))
    }
    fn complete(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::ParseError {
            raw_response: self.raw_response.clone(),
            expected_format: "valid JSON",
        }))
    }
    fn name(&self) -> &str {
        "ParseError"
    }
}

pub struct TimeoutBackend;

impl LlmBackend for TimeoutBackend {
    fn model(&self) -> &str {
        "mock"
    }
    fn narrate_continuation(
        &self,
        _agent_name: &str,
        _s: &str,
        _u: &str,
        _t: &str,
        _m: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Timeout))
    }
    fn complete(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Timeout))
    }
    fn name(&self) -> &str {
        "Timeout"
    }
}

pub struct FailingQuantifierBackend;

impl LlmBackend for FailingQuantifierBackend {
    fn model(&self) -> &str {
        "mock"
    }
    fn name(&self) -> &str {
        "FailingQuantifier"
    }
    fn narrate_continuation(
        &self,
        _agent_name: &str,
        _s: &str,
        _u: &str,
        _t: &str,
        _m: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Network {
            url: "http://quantifier".to_string(),
            detail: "Connection refused".to_string(),
        }))
    }
    fn complete(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Network {
            url: "http://quantifier".to_string(),
            detail: "Connection refused".to_string(),
        }))
    }
}

pub struct LowConfidenceQuantifierBackend;

impl LlmBackend for LowConfidenceQuantifierBackend {
    fn model(&self) -> &str {
        "mock"
    }
    fn name(&self) -> &str {
        "LowConfidenceQuantifier"
    }
    fn narrate_continuation(
        &self,
        agent_name: &str,
        _s: &str,
        _u: &str,
        _t: &str,
        _m: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Ok(empty_llm_result(
            "LowConfidenceQuantifier",
            "mock",
            agent_name,
        ))
    }
    fn complete(
        &self,
        agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        // Return ambiguous non-JSON text that fails structured parsing
        // and contains no known NPC IDs, forcing Low confidence fallback.
        let text = "I'm uncertain which characters are present in this scene.";
        Ok(LlmCallResult {
            text: text.to_string(),
            system_prompt: String::new(),
            user_prompt: String::new(),
            raw_request_json: String::new(),
            raw_response_json: format!("{{\"content\":\"{text}\"}}"),
            backend_name: "LowConfidenceQuantifier".to_string(),
            model_name: "mock".to_string(),
            agent_name: agent_name.to_string(),
        })
    }
}

pub struct MisleadingMovementQuantifierBackend;

impl LlmBackend for MisleadingMovementQuantifierBackend {
    fn model(&self) -> &str {
        "mock"
    }
    fn name(&self) -> &str {
        "MisleadingMovementQuantifier"
    }
    fn narrate_continuation(
        &self,
        agent_name: &str,
        _s: &str,
        _u: &str,
        _t: &str,
        _m: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        Ok(empty_llm_result(
            "MisleadingMovementQuantifier",
            "mock",
            agent_name,
        ))
    }
    fn complete(
        &self,
        agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        let text = r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "nonexistent_room"}}"#;
        Ok(LlmCallResult {
            text: text.to_string(),
            system_prompt: String::new(),
            user_prompt: String::new(),
            raw_request_json: String::new(),
            raw_response_json: format!("{{\"content\":\"{text}\"}}"),
            backend_name: "MisleadingMovementQuantifier".to_string(),
            model_name: "mock".to_string(),
            agent_name: agent_name.to_string(),
        })
    }
}
