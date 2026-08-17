//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Multi-stage prompt assembler: orchestrates layer rendering + context fitting.

use std::sync::{Arc, RwLock};

use crate::error::EngineError;
use crate::domain::model::character::PersonaCard;
use crate::domain::model::map::Room;
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::message_types::{MessageEntry, MessageType};
use crate::domain::model::template::TemplateVars;
use crate::domain::model::utils::template::render_template;
use crate::domain::model::world::WorldCard;
use crate::application::prompting::token_budget as budget;
use crate::application::prompting::token_budget::truncate_to_budget;
use crate::application::prompting::builders::sections::{
    build_post_history_prompt, build_system_prompt, render_known_npc_entry,
    render_present_relationships, sanitize_for_prompt,
};
use crate::application::prompting::types::NpcContext;
use crate::application::prompting::utils::context::fit_messages_to_context;

#[derive(Debug, Clone, PartialEq)]
pub struct AssembledPrompt {
    pub system_prompt: String,
    pub user_prompt: String,
    pub max_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct PromptContext<'a> {
    pub world: &'a WorldCard,
    pub room: &'a Room,
    pub npcs: NpcContext<'a>,
    pub persona: &'a PersonaCard,
    pub user_message: &'a str,
    pub history: &'a [MessageEntry],
    pub template_vars: TemplateVars,
}

pub struct PromptAssembler {
    max_context_tokens: u32,
    max_tokens: Option<u32>,
    settings: Option<Arc<RwLock<AppSettings>>>,
}

impl PromptAssembler {
    pub fn new(max_context_tokens: u32) -> Self {
        Self {
            max_context_tokens,
            max_tokens: None,
            settings: None,
        }
    }

    pub fn with_max_tokens(mut self, max: u32) -> Self {
        self.max_tokens = Some(max);
        self
    }

    pub fn with_settings(mut self, settings: Arc<RwLock<AppSettings>>) -> Self {
        self.settings = Some(settings);
        self
    }
}

impl PromptAssembler {
    pub fn assemble(
        &self,
        context: &PromptContext,
        preset: &PromptPreset,
        global_rules: &[String],
        response_length: Option<&str>,
    ) -> Result<AssembledPrompt, EngineError> {
        let system_prompt = build_system_prompt(preset, global_rules, &context.template_vars);
        let post_history_prompt =
            build_post_history_prompt(preset, response_length, &context.template_vars);

        let renderer = LayerRenderer {
            world: context.world,
            room: context.room,
            npcs: context.npcs,
            persona: context.persona,
            user_message: context.user_message,
            history: context.history,
            system_prompt,
            post_history_prompt,
            template_vars: &context.template_vars,
        };

        let (max_context_tokens, requested_max_tokens) = self.resolve_budget();
        let (system, user, actual_max_tokens) =
            renderer.render_and_fit(max_context_tokens, requested_max_tokens)?;

        Ok(AssembledPrompt {
            system_prompt: system,
            user_prompt: user,
            max_tokens: actual_max_tokens,
        })
    }

    fn resolve_budget(&self) -> (u32, Option<u32>) {
        let Some(settings) = &self.settings else {
            return (self.max_context_tokens, self.max_tokens);
        };
        let guard = settings.read().unwrap_or_else(|e| e.into_inner());
        let conn = guard.narration_connection();
        (conn.resolve_max_context_tokens(), conn.max_tokens)
    }
}

impl PromptContext<'_> {
    pub fn new<'a>(
        world: &'a WorldCard,
        room: &'a Room,
        npcs: NpcContext<'a>,
        persona: &'a PersonaCard,
        user_message: &'a str,
        history: &'a [MessageEntry],
    ) -> PromptContext<'a> {
        PromptContext {
            world,
            room,
            npcs,
            persona,
            user_message,
            history,
            template_vars: TemplateVars::new(&persona.sheet.name),
        }
    }

    pub fn build_narration_prompt(
        &self,
        preset: &PromptPreset,
        global_rules: &[String],
        response_length: Option<&str>,
        max_context_tokens: u32,
        max_tokens: Option<u32>,
    ) -> Result<AssembledPrompt, EngineError> {
        let mut assembler = PromptAssembler::new(max_context_tokens);
        if let Some(max) = max_tokens {
            assembler = assembler.with_max_tokens(max);
        }
        assembler.assemble(self, preset, global_rules, response_length)
    }
}

struct LayerRenderer<'a> {
    world: &'a WorldCard,
    room: &'a Room,
    npcs: NpcContext<'a>,
    persona: &'a crate::domain::model::character::PersonaCard,
    user_message: &'a str,
    history: &'a [MessageEntry],
    system_prompt: String,
    post_history_prompt: String,
    template_vars: &'a TemplateVars,
}

impl<'a> LayerRenderer<'a> {
    fn render_and_fit(
        &self,
        max_context_tokens: u32,
        requested_max_tokens: Option<u32>,
    ) -> Result<(String, String, u32), EngineError> {
        let system = self.render_system_layer();

        let user = [
            self.render_game_state_layer(),
            self.render_npc_cards_layer(),
            self.render_persona_layer(),
            self.render_world_info_layer(),
            self.render_history_layer(),
            self.post_history_prompt.clone(),
            self.render_user_layer(),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
        let user = format!("{user}\n\n");

        let (fitted_system, fitted_user, actual_max_tokens) =
            fit_messages_to_context(&system, &user, max_context_tokens, requested_max_tokens)?;

        Ok((fitted_system, fitted_user, actual_max_tokens))
    }

    fn render_system_layer(&self) -> String {
        self.system_prompt.clone()
    }

    fn render_game_state_layer(&self) -> String {
        let mut output = String::from("<GameState>\n");
        output.push_str("Current Location: ");
        output.push_str(&self.room.name);
        output.push_str("\n\n");
        output.push_str(&render_template(&self.room.description, self.template_vars));
        output.push_str("\n\n");
        output.push_str("</GameState>\n");
        output
    }

    fn render_npc_cards_layer(&self) -> String {
        let mut output = String::new();
        let in_area_ids: std::collections::HashSet<_> = self
            .npcs
            .npcs_in_area
            .iter()
            .map(|n| n.id.as_str())
            .collect();

        output.push_str("<KnownNpcs>");
        if self.npcs.all_npcs.is_empty() {
            output.push_str("No characters in this world.\n");
        } else {
            for npc in self.npcs.all_npcs {
                output.push_str(&render_known_npc_entry(
                    npc,
                    &in_area_ids,
                    self.template_vars,
                ));
            }
        }
        output.push_str("</KnownNpcs>\n\n");

        output.push_str("<NpcsInRoom>");
        if self.npcs.npcs_in_area.is_empty() {
            output.push_str("No NPCs are present in this location.\n");
        } else {
            for npc in self.npcs.npcs_in_area {
                output.push_str("--- ");
                output.push_str(&npc.sheet.name);
                output.push_str(" ---\n");
                output.push_str("Description: ");
                output.push_str(&render_template(&npc.sheet.description, self.template_vars));
                output.push('\n');
                output.push_str("Personality: ");
                output.push_str(&render_template(&npc.sheet.personality, self.template_vars));
                output.push('\n');
                if !npc.sheet.scenario.is_empty() {
                    output.push_str("Context: ");
                    output.push_str(&render_template(&npc.sheet.scenario, self.template_vars));
                    output.push('\n');
                }

                let relations_block = render_present_relationships(
                    npc,
                    self.npcs.all_npcs,
                    &in_area_ids,
                    self.template_vars,
                );
                if let Some(relations_block) = relations_block {
                    output.push_str(&relations_block);
                }

                output.push('\n');
            }
        }
        output.push_str("</NpcsInRoom>\n");

        output
    }

    fn render_persona_layer(&self) -> String {
        let mut output = String::from("<PlayerCharacter>\n");
        output.push_str("Name: ");
        output.push_str(&self.persona.sheet.name);
        output.push_str("\n\n");
        output.push_str("Description: ");
        output.push_str(&render_template(
            &self.persona.sheet.description,
            self.template_vars,
        ));
        output.push_str("\n\n");
        output.push_str("Personality: ");
        output.push_str(&render_template(
            &self.persona.sheet.personality,
            self.template_vars,
        ));
        output.push_str("\n\n");
        output.push_str("Background: ");
        output.push_str(&render_template(
            &self.persona.sheet.scenario,
            self.template_vars,
        ));
        output.push('\n');

        output.push_str("</PlayerCharacter>\n");
        output
    }

    fn render_world_info_layer(&self) -> String {
        let mut output = String::from("<WorldLore>\n");
        output.push_str("World: ");
        output.push_str(&self.world.name);
        output.push_str("\n\n");
        output.push_str(&render_template(
            &self.world.description,
            self.template_vars,
        ));
        output.push_str("\n\n");

        output.push_str("</WorldLore>\n");
        output
    }

    fn render_history_layer(&self) -> String {
        let mut output = String::from("<ConversationHistory>\n");

        if self.history.is_empty() {
            output.push_str("(No history yet - this is the start of the conversation)\n");
            output.push_str("</ConversationHistory>\n");
            return output;
        }

        let mut history_text = String::new();
        for entry in self.history {
            match entry.message_type {
                MessageType::Narrator => history_text.push_str(&format!("{}\n", entry.text)),
                _ => {
                    let sender = entry.sender.as_deref().unwrap_or("Narrator");
                    history_text.push_str(&format!("{}: {}\n", sender, entry.text));
                }
            }
        }

        let truncated = truncate_to_budget(&history_text, budget::MAX_HISTORY_TOKENS as usize);
        output.push_str(&truncated);

        output.push_str("</ConversationHistory>\n");
        output
    }

    fn render_user_layer(&self) -> String {
        let mut output = String::from("<PlayerInput>\n");
        let sanitized = sanitize_for_prompt(self.user_message);
        output.push_str(&sanitized);
        output.push_str("\n</PlayerInput>\n");

        output
    }
}
