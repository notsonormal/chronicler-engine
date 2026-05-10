use crate::error::EngineError;
use crate::model::llm_backend::LlmBackendType;
use crate::model::settings::Connection;
use crate::narrative::llm_client::{call_ollama, call_openrouter_with_model};

use super::core::{action_boundary_contains, quantify_room_with_llm_call};
use super::types::{
    MovementParseResult, QuantifierConfidence, QuantifierParseResult, QuantifierPromptContext,
    QuantifierResult,
};

/// [DOC: docs/system/llm_processing.md]
pub struct QuantifierBackend {
    api_key: String,
    model: String,
    max_tokens: Option<u32>,
    #[allow(dead_code)]
    max_context_tokens: u32,
}

impl QuantifierBackend {
    fn from_connection(connection: &Connection) -> Self {
        Self {
            api_key: connection.resolve_api_key().unwrap_or_default(),
            model: connection.model.clone(),
            max_tokens: connection.max_tokens,
            max_context_tokens: connection.resolve_max_context_tokens(),
        }
    }
}

impl QuantifierBackendTrait for QuantifierBackend {
    fn quantify_room(
        &self,
        context: &QuantifierPromptContext,
        fallback_npc_ids: &[String],
    ) -> Result<QuantifierResult, EngineError> {
        let api_key = self.api_key.clone();
        let model = self.model.clone();
        let max_tokens = self.max_tokens;
        quantify_room_with_llm_call(context, fallback_npc_ids, &model, |system, user, m| {
            call_openrouter_with_model(&api_key, system, user, m, max_tokens)
        })
    }
}

pub struct OllamaQuantifierBackend {
    base_url: String,
    model: String,
    max_tokens: Option<u32>,
    #[allow(dead_code)]
    max_context_tokens: u32,
}

impl OllamaQuantifierBackend {
    fn from_connection(connection: &Connection) -> Self {
        Self {
            base_url: connection.resolve_base_url(),
            model: connection.model.clone(),
            max_tokens: connection.max_tokens,
            max_context_tokens: connection.resolve_max_context_tokens(),
        }
    }
}

impl QuantifierBackendTrait for OllamaQuantifierBackend {
    fn quantify_room(
        &self,
        context: &QuantifierPromptContext,
        fallback_npc_ids: &[String],
    ) -> Result<QuantifierResult, EngineError> {
        let base_url = self.base_url.clone();
        let model = self.model.clone();
        let max_tokens = self.max_tokens;
        quantify_room_with_llm_call(context, fallback_npc_ids, &model, |system, user, m| {
            call_ollama(&base_url, m, system, user, max_tokens)
        })
    }
}
pub trait QuantifierBackendTrait: Send + Sync {
    fn quantify_room(
        &self,
        context: &QuantifierPromptContext,
        fallback_npc_ids: &[String],
    ) -> Result<QuantifierResult, EngineError>;
}

pub struct RealQuantifierBackend {
    inner: QuantifierBackend,
}

impl QuantifierBackendTrait for RealQuantifierBackend {
    fn quantify_room(
        &self,
        context: &QuantifierPromptContext,
        fallback_npc_ids: &[String],
    ) -> Result<QuantifierResult, EngineError> {
        self.inner.quantify_room(context, fallback_npc_ids)
    }
}

/// [DOC: docs/reference/quantifier_prompt.md]
#[derive(Default)]
pub struct MockQuantifierBackend {
    /// NPC IDs to return (overrides auto-detection).
    pub npcs_to_return: Vec<String>,
    /// Movement to return (optional).
    pub movement_to_return: Option<MovementParseResult>,
}

impl QuantifierBackendTrait for MockQuantifierBackend {
    fn quantify_room(
        &self,
        context: &QuantifierPromptContext,
        _fallback_npc_ids: &[String],
    ) -> Result<QuantifierResult, EngineError> {
        // Auto-detect NPCs from player action text if none specified
        let npc_ids = if !self.npcs_to_return.is_empty() {
            self.npcs_to_return.clone()
        } else {
            // Word-boundary matching - look for known NPC names as whole words in player action
            let action_lower = context.player_action.to_lowercase();
            let word_boundary_chars: std::collections::HashSet<char> = [
                ' ', '.', ',', '!', '?', '\n', '\t', '\r', '\'', '"', ':', ';',
            ]
            .into_iter()
            .collect();

            context
                .all_known_npcs
                .iter()
                .filter_map(|npc| {
                    let npc_name = npc.sheet.name.to_lowercase();
                    if action_boundary_contains(&action_lower, &npc_name, &word_boundary_chars) {
                        Some(npc.id.clone())
                    } else {
                        None
                    }
                })
                .collect()
        };

        Ok(QuantifierResult {
            npcs: QuantifierParseResult {
                npc_ids,
                confidence: QuantifierConfidence::High,
            },
            movement: self
                .movement_to_return
                .clone()
                .unwrap_or(MovementParseResult {
                    movement_type: None,
                    destination: None,
                    confidence: QuantifierConfidence::High,
                }),
        })
    }
}

pub fn get_quantifier_backend_for(connection: &Connection) -> Box<dyn QuantifierBackendTrait> {
    match connection.provider {
        LlmBackendType::Mock => Box::new(MockQuantifierBackend::default()),
        LlmBackendType::Ollama => Box::new(OllamaQuantifierBackend::from_connection(connection)),
        LlmBackendType::OpenRouter | LlmBackendType::DeepSeek => Box::new(RealQuantifierBackend {
            inner: QuantifierBackend::from_connection(connection),
        }),
    }
}

/// [DOC: docs/system/quantifier.md]
pub fn get_quantifier_backend() -> Box<dyn QuantifierBackendTrait> {
    let settings = crate::settings::load_settings().unwrap_or_default();
    let connection = settings
        .get_quantifier_connection()
        .cloned()
        .unwrap_or_else(|| Connection::new("default", "Default", LlmBackendType::Mock));
    get_quantifier_backend_for(&connection)
}
