use chronicler_engine::error::{EngineError, LlmFailure};
use chronicler_engine::model::character::NpcCard;
use chronicler_engine::narrative::llm::backend::{LlmBackend, LlmCallResult};
use chronicler_engine::narrative::prompt::PromptContext;

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

/// Simulates an HTTP error from an LLM provider (401, 429, 503, etc.)
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
    fn generate_dialogue(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
        _npc: &NpcCard,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Http {
            status: self.status,
            body: self.body.clone(),
        }))
    }
    fn narrate_action(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Http {
            status: self.status,
            body: self.body.clone(),
        }))
    }
    fn narrate_arrival(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Http {
            status: self.status,
            body: self.body.clone(),
        }))
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

/// Simulates a network-level failure (DNS, timeout, connection refused)
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
    fn generate_dialogue(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
        _npc: &NpcCard,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Network {
            url: self.url.clone(),
            detail: self.detail.clone(),
        }))
    }
    fn narrate_action(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Network {
            url: self.url.clone(),
            detail: self.detail.clone(),
        }))
    }
    fn narrate_arrival(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Network {
            url: self.url.clone(),
            detail: self.detail.clone(),
        }))
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

/// Simulates a parse error (model returned non-JSON)
pub struct ParseErrorBackend {
    pub raw_response: String,
}

impl LlmBackend for ParseErrorBackend {
    fn model(&self) -> &str {
        "mock"
    }
    fn generate_dialogue(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
        _npc: &NpcCard,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::ParseError {
            raw_response: self.raw_response.clone(),
            expected_format: "valid JSON",
        }))
    }
    fn narrate_action(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::ParseError {
            raw_response: self.raw_response.clone(),
            expected_format: "valid JSON",
        }))
    }
    fn narrate_arrival(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::ParseError {
            raw_response: self.raw_response.clone(),
            expected_format: "valid JSON",
        }))
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

/// Simulates a timeout
pub struct TimeoutBackend;

impl LlmBackend for TimeoutBackend {
    fn model(&self) -> &str {
        "mock"
    }
    fn generate_dialogue(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
        _npc: &NpcCard,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Timeout))
    }
    fn narrate_action(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Timeout))
    }
    fn narrate_arrival(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Timeout))
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

/// Quantifier backend that fails entirely (simulates LLM call failure in quantifier)
pub struct FailingQuantifierBackend;

impl LlmBackend for FailingQuantifierBackend {
    fn model(&self) -> &str {
        "mock"
    }
    fn name(&self) -> &str {
        "FailingQuantifier"
    }
    fn generate_dialogue(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
        _npc: &NpcCard,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Network {
            url: "http://quantifier".to_string(),
            detail: "Connection refused".to_string(),
        }))
    }
    fn narrate_action(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Network {
            url: "http://quantifier".to_string(),
            detail: "Connection refused".to_string(),
        }))
    }
    fn narrate_arrival(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Network {
            url: "http://quantifier".to_string(),
            detail: "Connection refused".to_string(),
        }))
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

/// Quantifier backend that returns low confidence (simulates poor model output)
pub struct LowConfidenceQuantifierBackend;

impl LlmBackend for LowConfidenceQuantifierBackend {
    fn model(&self) -> &str {
        "mock"
    }
    fn name(&self) -> &str {
        "LowConfidenceQuantifier"
    }
    fn generate_dialogue(
        &self,
        agent_name: &str,
        _ctx: &PromptContext,
        _npc: &NpcCard,
    ) -> Result<LlmCallResult, EngineError> {
        Ok(empty_llm_result(
            "LowConfidenceQuantifier",
            "mock",
            agent_name,
        ))
    }
    fn narrate_action(
        &self,
        agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Ok(empty_llm_result(
            "LowConfidenceQuantifier",
            "mock",
            agent_name,
        ))
    }
    fn narrate_arrival(
        &self,
        agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Ok(empty_llm_result(
            "LowConfidenceQuantifier",
            "mock",
            agent_name,
        ))
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
        let text = r#"{"npcs_in_room": []}"#;
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

/// Quantifier backend that returns a movement to a non-existent room
pub struct MisleadingMovementQuantifierBackend;

impl LlmBackend for MisleadingMovementQuantifierBackend {
    fn model(&self) -> &str {
        "mock"
    }
    fn name(&self) -> &str {
        "MisleadingMovementQuantifier"
    }
    fn generate_dialogue(
        &self,
        agent_name: &str,
        _ctx: &PromptContext,
        _npc: &NpcCard,
    ) -> Result<LlmCallResult, EngineError> {
        Ok(empty_llm_result(
            "MisleadingMovementQuantifier",
            "mock",
            agent_name,
        ))
    }
    fn narrate_action(
        &self,
        agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Ok(empty_llm_result(
            "MisleadingMovementQuantifier",
            "mock",
            agent_name,
        ))
    }
    fn narrate_arrival(
        &self,
        agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        Ok(empty_llm_result(
            "MisleadingMovementQuantifier",
            "mock",
            agent_name,
        ))
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
