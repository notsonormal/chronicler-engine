use chronicler_engine::error::{EngineError, LlmFailure};
use chronicler_engine::model::character::NpcCard;
use chronicler_engine::narrative::agents::quantifier::backends::QuantifierBackendTrait;
use chronicler_engine::narrative::agents::quantifier::types::{
    MovementParseResult, MovementType, QuantifierConfidence, QuantifierParseResult,
    QuantifierPromptContext, QuantifierResult,
};
use chronicler_engine::narrative::llm::backend::LlmBackend;
use chronicler_engine::narrative::prompt::PromptContext;

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
    fn generate_dialogue(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
        _npc: &NpcCard,
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Http {
            status: self.status,
            body: self.body.clone(),
        }))
    }
    fn narrate_action(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Http {
            status: self.status,
            body: self.body.clone(),
        }))
    }
    fn narrate_arrival(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
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
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Http {
            status: self.status,
            body: self.body.clone(),
        }))
    }
    fn narrate_action_from_prompt(
        &self,
        _agent_name: &str,
        _s: &str,
        _u: &str,
        _m: Option<u32>,
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
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
    fn generate_dialogue(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
        _npc: &NpcCard,
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Network {
            url: self.url.clone(),
            detail: self.detail.clone(),
        }))
    }
    fn narrate_action(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Network {
            url: self.url.clone(),
            detail: self.detail.clone(),
        }))
    }
    fn narrate_arrival(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
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
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Network {
            url: self.url.clone(),
            detail: self.detail.clone(),
        }))
    }
    fn narrate_action_from_prompt(
        &self,
        _agent_name: &str,
        _s: &str,
        _u: &str,
        _m: Option<u32>,
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
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
    fn generate_dialogue(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
        _npc: &NpcCard,
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::ParseError {
            raw_response: self.raw_response.clone(),
            expected_format: "valid JSON",
        }))
    }
    fn narrate_action(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::ParseError {
            raw_response: self.raw_response.clone(),
            expected_format: "valid JSON",
        }))
    }
    fn narrate_arrival(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
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
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::ParseError {
            raw_response: self.raw_response.clone(),
            expected_format: "valid JSON",
        }))
    }
    fn narrate_action_from_prompt(
        &self,
        _agent_name: &str,
        _s: &str,
        _u: &str,
        _m: Option<u32>,
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
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
    fn generate_dialogue(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
        _npc: &NpcCard,
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Timeout))
    }
    fn narrate_action(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Timeout))
    }
    fn narrate_arrival(
        &self,
        _agent_name: &str,
        _ctx: &PromptContext,
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Timeout))
    }
    fn narrate_continuation(
        &self,
        _agent_name: &str,
        _s: &str,
        _u: &str,
        _t: &str,
        _m: Option<u32>,
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Timeout))
    }
    fn narrate_action_from_prompt(
        &self,
        _agent_name: &str,
        _s: &str,
        _u: &str,
        _m: Option<u32>,
    ) -> Result<chronicler_engine::narrative::llm::backend::LlmCallResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Timeout))
    }
    fn name(&self) -> &str {
        "Timeout"
    }
}

/// Quantifier backend that fails entirely (simulates LLM call failure in quantifier)
pub struct FailingQuantifierBackend;

impl QuantifierBackendTrait for FailingQuantifierBackend {
    fn quantify_room(
        &self,
        _context: &QuantifierPromptContext,
        _fallback_npc_ids: &[String],
    ) -> Result<QuantifierResult, EngineError> {
        Err(EngineError::Llm(LlmFailure::Network {
            url: "http://quantifier".to_string(),
            detail: "Connection refused".to_string(),
        }))
    }
}

/// Quantifier backend that returns low confidence (simulates poor model output)
pub struct LowConfidenceQuantifierBackend;

impl QuantifierBackendTrait for LowConfidenceQuantifierBackend {
    fn quantify_room(
        &self,
        _context: &QuantifierPromptContext,
        _fallback_npc_ids: &[String],
    ) -> Result<QuantifierResult, EngineError> {
        Ok(QuantifierResult {
            npcs: QuantifierParseResult {
                npc_ids: vec![],
                confidence: QuantifierConfidence::Low,
            },
            movement: MovementParseResult {
                movement_type: None,
                destination: None,
                confidence: QuantifierConfidence::Low,
            },
        })
    }
}

/// Quantifier backend that returns a movement to a non-existent room
pub struct MisleadingMovementQuantifierBackend;

impl QuantifierBackendTrait for MisleadingMovementQuantifierBackend {
    fn quantify_room(
        &self,
        _context: &QuantifierPromptContext,
        _fallback_npc_ids: &[String],
    ) -> Result<QuantifierResult, EngineError> {
        Ok(QuantifierResult {
            npcs: QuantifierParseResult {
                npc_ids: vec![],
                confidence: QuantifierConfidence::High,
            },
            movement: MovementParseResult {
                movement_type: Some(MovementType::Entering),
                destination: Some("nonexistent_room".to_string()),
                confidence: QuantifierConfidence::High,
            },
        })
    }
}
