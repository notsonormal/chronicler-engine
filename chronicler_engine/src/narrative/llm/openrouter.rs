use std::sync::{Arc, RwLock};

use crate::error::{EngineError, LlmFailure};
use crate::model::character::NpcCard;
use crate::model::settings::{AppSettings, Connection};
use crate::narrative::llm_client::call_openrouter_with_model;
use crate::narrative::prompt::{PromptBuilder, PromptContext};
use crate::storage::llm_message_storage::LlmMessageStorage;

use super::backend::{LlmBackend, LlmCallResult, merge_single_user_message};

#[derive(Clone, Default)]
pub struct OpenRouterBackend {
    api_key: String,
    model: String,
    single_user_message: bool,
    max_tokens: Option<u32>,
    max_context_tokens: u32,
    storage: Option<Arc<dyn LlmMessageStorage>>,
    settings: Option<Arc<RwLock<AppSettings>>>,
}

impl OpenRouterBackend {
    pub fn from_connection(
        connection: &Connection,
        storage: Option<Arc<dyn LlmMessageStorage>>,
        settings: Option<Arc<RwLock<AppSettings>>>,
    ) -> Self {
        let api_key = connection.resolve_api_key().unwrap_or_default();
        Self {
            api_key,
            model: connection.model.clone(),
            single_user_message: connection.single_user_message,
            max_tokens: connection.max_tokens,
            max_context_tokens: connection.resolve_max_context_tokens(),
            storage,
            settings,
        }
    }

    fn call(
        &self,
        system_prompt: &str,
        user_text: &str,
        max_tokens: Option<u32>,
    ) -> Result<crate::narrative::llm_client::ChatCompletionResult, EngineError> {
        let (system, user) = if self.single_user_message {
            ("", merge_single_user_message(system_prompt, user_text))
        } else {
            (system_prompt, user_text.to_string())
        };
        let max_tokens = max_tokens.or(self.max_tokens);
        let result =
            call_openrouter_with_model(&self.api_key, system, &user, &self.model, max_tokens)?;
        if result.text.trim().is_empty() {
            return Err(EngineError::Llm(LlmFailure::EmptyResponse));
        }
        Ok(result)
    }

    fn response_length(&self) -> String {
        self.settings
            .as_ref()
            .and_then(|s| s.read().ok().map(|g| g.response_length.clone()))
            .unwrap_or_default()
    }

    /// Build a prompt from context using this backend's token limits, then call the LLM.
    fn narrate_from_context(
        &self,
        agent_name: &str,
        context: &PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        let response_length = self.response_length();
        let builder = PromptBuilder::from_context(context)
            .with_max_context_tokens(self.max_context_tokens)
            .with_max_tokens(
                self.max_tokens
                    .unwrap_or(crate::narrative::prompt::budget::MAX_RESPONSE_TOKENS),
            )
            .with_response_length(&response_length);
        let (system_prompt, user_text, max_tokens) = builder.build_split()?;
        self.complete(agent_name, &system_prompt, &user_text, Some(max_tokens))
    }
}

impl LlmBackend for OpenRouterBackend {
    fn model(&self) -> &str {
        &self.model
    }

    fn generate_dialogue(
        &self,
        agent_name: &str,
        context: &PromptContext,
        npc: &NpcCard,
    ) -> Result<LlmCallResult, EngineError> {
        log::info!("[LLM] Generating dialogue for NPC: {}", npc.sheet.name);

        let user_msg = format!(
            "The player says to {}: \"{}\"",
            npc.sheet.name, context.user_message
        );

        let npc_context = PromptContext {
            world: context.world,
            room: context.room,
            all_npcs: &[npc.clone()],
            npcs_in_area: &[npc.clone()],
            player: context.player,
            user_message: &user_msg,
            history: context.history,
        };

        self.narrate_from_context(agent_name, &npc_context)
    }

    fn narrate_action(
        &self,
        agent_name: &str,
        context: &PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        log::info!(
            "[LLM] Generating action narration for: {}",
            context.user_message
        );

        self.narrate_from_context(agent_name, context)
    }

    fn narrate_arrival(
        &self,
        agent_name: &str,
        context: &PromptContext,
    ) -> Result<LlmCallResult, EngineError> {
        log::info!(
            "[LLM] Generating arrival narration for room: {}",
            context.room.name
        );

        let user_msg = format!(
            "{} enters the {}.",
            context.player.sheet.name, context.room.name
        );

        let arrival_context = PromptContext {
            world: context.world,
            room: context.room,
            all_npcs: context.all_npcs,
            npcs_in_area: context.npcs_in_area,
            player: context.player,
            user_message: &user_msg,
            history: context.history,
        };

        self.narrate_from_context(agent_name, &arrival_context)
    }

    fn narrate_continuation(
        &self,
        agent_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        _trigger_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        log::info!("[LLM] Generating continuation narration");

        Ok(self.wrap_and_save(
            agent_name,
            self.call(system_prompt, user_prompt, max_tokens)?,
        ))
    }

    fn complete(
        &self,
        agent_name: &str,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        log::info!("[LLM] Generating action from prompt");

        Ok(self.wrap_and_save(
            agent_name,
            self.call(system_prompt, user_prompt, max_tokens)?,
        ))
    }

    fn name(&self) -> &str {
        "OpenRouter"
    }

    fn save_message(&self, message: &crate::model::llm_message::LlmMessage) {
        if let Some(storage) = &self.storage {
            let _ = storage.save(message);
        }
    }
}
