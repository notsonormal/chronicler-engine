use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::settings::Connection;
use crate::narrative::llm_client::call_ollama;
use crate::narrative::prompt::{PromptBuilder, PromptContext};

use super::backend::LlmBackend;
use super::backend::merge_single_user_message;

#[derive(Clone, Default)]
pub struct OllamaBackend {
    base_url: String,
    model: String,
    single_user_message: bool,
    max_tokens: Option<u32>,
    max_context_tokens: u32,
}

impl OllamaBackend {
    pub fn from_connection(connection: &Connection) -> Self {
        Self {
            base_url: connection.resolve_base_url(),
            model: connection.model.clone(),
            single_user_message: connection.single_user_message,
            max_tokens: connection.max_tokens,
            max_context_tokens: connection.resolve_max_context_tokens(),
        }
    }

    fn call(
        &self,
        system_prompt: &str,
        user_text: &str,
        max_tokens: Option<u32>,
    ) -> Result<String, EngineError> {
        let (system, user) = if self.single_user_message {
            ("", merge_single_user_message(system_prompt, user_text))
        } else {
            (system_prompt, user_text.to_string())
        };
        let max_tokens = max_tokens.or(self.max_tokens);
        let result = call_ollama(&self.base_url, &self.model, system, &user, max_tokens)
            .map_err(EngineError::Narrative)?;
        if result.trim().is_empty() {
            return Err(EngineError::LlmEmptyResponse);
        }
        Ok(result)
    }

    /// Build a prompt from context using this backend's token limits, then call the LLM.
    fn narrate_from_context(&self, context: &PromptContext) -> Result<String, EngineError> {
        let settings = crate::settings::load_settings().unwrap_or_default();
        let builder = PromptBuilder::from_context(context)
            .with_max_context_tokens(self.max_context_tokens)
            .with_max_tokens(
                self.max_tokens
                    .unwrap_or(crate::narrative::prompt::budget::MAX_RESPONSE_TOKENS),
            )
            .with_response_length(&settings.response_length);
        let (system_prompt, user_text, max_tokens) = builder.build_split()?;
        self.call(&system_prompt, &user_text, Some(max_tokens))
    }
}

impl LlmBackend for OllamaBackend {
    fn generate_dialogue(
        &self,
        context: &PromptContext,
        npc: &NpcCard,
    ) -> Result<String, EngineError> {
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

        self.narrate_from_context(&npc_context)
    }

    fn narrate_action(&self, context: &PromptContext) -> Result<String, EngineError> {
        log::info!(
            "[LLM] Generating action narration for: {}",
            context.user_message
        );

        self.narrate_from_context(context)
    }

    fn narrate_arrival(&self, context: &PromptContext) -> Result<String, EngineError> {
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

        self.narrate_from_context(&arrival_context)
    }

    fn narrate_continuation(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        _trigger_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<String, EngineError> {
        log::info!("[LLM] Generating continuation narration");
        self.call(system_prompt, user_prompt, max_tokens)
    }

    fn narrate_action_from_prompt(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        max_tokens: Option<u32>,
    ) -> Result<String, EngineError> {
        log::info!("[LLM] Generating action from prompt");
        self.call(system_prompt, user_prompt, max_tokens)
    }

    fn name(&self) -> &str {
        "Ollama"
    }
}
