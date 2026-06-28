//! [DOC: docs/system/prompt_system.md]
//! Multi-stage prompt builder

use crate::error::EngineError;
use crate::model::map::Room;
use crate::model::prompt_preset::PromptPreset;
use crate::model::state::message_types::MessageEntry;
use crate::model::template::{render_template, TemplateVars};
use crate::model::world::WorldCard;
use crate::narrative::prompt::budget;
use crate::narrative::prompt::budget::truncate_to_budget;
use crate::narrative::prompt::context::fit_messages_to_context;
use crate::narrative::prompt::types::{NpcContext, PromptContext};

/// Assembles prompt text from a PromptPreset into XML-wrapped sections.
/// Section order: role → instructions → writing_style → global_rules → output_format
pub fn assemble_prompt_text(
    preset: &PromptPreset,
    global_rules: &[String],
    response_length: Option<&str>,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    push_section(&mut parts, preset.role.as_deref(), "role");
    push_section(&mut parts, preset.instructions.as_deref(), "instructions");
    push_section(&mut parts, preset.writing_style.as_deref(), "writing_style");

    if !global_rules.is_empty() {
        let rules_text = global_rules
            .iter()
            .map(|r| format!("- {r}"))
            .collect::<Vec<_>>()
            .join("\n");
        parts.push(wrap_xml(&rules_text, "global_rules"));
    }

    if let Some(output_format) = &preset.output_format {
        let mut output_text = output_format.clone();
        if let Some(length) = response_length {
            output_text.push_str("\n\nResponse Length:\n");
            output_text.push_str(length);
        }
        parts.push(wrap_xml(&output_text, "output_format"));
    }

    parts.join("\n\n")
}

fn push_section(parts: &mut Vec<String>, content: Option<&str>, tag: &str) {
    if let Some(content) = content {
        parts.push(wrap_xml(content, tag));
    }
}

fn wrap_xml(content: &str, tag: &str) -> String {
    let indented = content
        .lines()
        .map(|line| {
            if line.is_empty() {
                line.to_string()
            } else {
                format!("    {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("<{tag}>\n{indented}\n</{tag}>")
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssembledPrompt {
    pub system_prompt: String,
    pub user_prompt: String,
    pub max_tokens: u32,
}

pub struct LayeredPromptAssembler {
    max_context_tokens: u32,
    max_tokens: Option<u32>,
}

impl LayeredPromptAssembler {
    pub fn new(max_context_tokens: u32) -> Self {
        Self {
            max_context_tokens,
            max_tokens: None,
        }
    }

    pub fn with_max_tokens(mut self, max: u32) -> Self {
        self.max_tokens = Some(max);
        self
    }
}

impl LayeredPromptAssembler {
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
            player: context.player,
            user_message: context.user_message,
            history: context.history,
            system_prompt,
            post_history_prompt,
            template_vars: &context.template_vars,
        };

        let (system, user, max_tokens) =
            renderer.render_and_fit(self.max_context_tokens, self.max_tokens)?;

        Ok(AssembledPrompt {
            system_prompt: system,
            user_prompt: user,
            max_tokens,
        })
    }
}

fn build_system_prompt(
    preset: &PromptPreset,
    global_rules: &[String],
    vars: &TemplateVars,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    let role = preset.role.as_ref().map(|r| render_template(r, vars));
    push_section(&mut parts, role.as_deref(), "role");
    let instructions = preset
        .instructions
        .as_ref()
        .map(|i| render_template(i, vars));
    push_section(&mut parts, instructions.as_deref(), "instructions");

    if !global_rules.is_empty() {
        let rules_text = global_rules
            .iter()
            .map(|r| format!("- {}", render_template(r, vars)))
            .collect::<Vec<_>>()
            .join("\n");
        parts.push(wrap_xml(&rules_text, "global_rules"));
    }

    parts.join("\n\n")
}

fn build_post_history_prompt(
    preset: &PromptPreset,
    response_length: Option<&str>,
    vars: &TemplateVars,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    let writing_style = preset
        .writing_style
        .as_ref()
        .map(|w| render_template(w, vars));
    push_section(&mut parts, writing_style.as_deref(), "writing_style");

    if let Some(output_format) = &preset.output_format {
        let mut output_text = render_template(output_format, vars);
        if let Some(length) = response_length {
            output_text.push_str("\n\nResponse Length:\n");
            output_text.push_str(length);
        }
        parts.push(wrap_xml(&output_text, "output_format"));
    }

    parts.join("\n\n")
}

struct LayerRenderer<'a> {
    world: &'a WorldCard,
    room: &'a Room,
    npcs: NpcContext<'a>,
    player: &'a crate::model::character::PlayerCard,
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
            self.render_player_layer(),
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
                let presence = if in_area_ids.contains(npc.id.as_str()) {
                    "(in room)"
                } else {
                    "(elsewhere)"
                };
                output.push_str(&format!("- {} {}\n", npc.sheet.name, presence));

                let summary_text = npc
                    .sheet
                    .summary
                    .clone()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| {
                        npc.sheet
                            .description
                            .lines()
                            .take(3)
                            .collect::<Vec<_>>()
                            .join("\n")
                    });
                let summary_text = render_template(&summary_text, self.template_vars);

                for line in summary_text.lines() {
                    output.push_str(&format!("  {line}\n"));
                }
                output.push('\n');
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

                let present_relations: Vec<_> = npc
                    .relationships
                    .iter()
                    .filter(|r| in_area_ids.contains(r.with.as_str()))
                    .collect();

                if !present_relations.is_empty() {
                    output.push_str("Relationships:\n");
                    for rel in present_relations {
                        let partner_name = self
                            .npcs
                            .all_npcs
                            .iter()
                            .find(|n| n.id == rel.with)
                            .map(|n| n.sheet.name.as_str())
                            .unwrap_or(&rel.with);
                        output.push_str(&format!(
                            "  → {}: {}\n",
                            partner_name,
                            render_template(rel.display_text(), self.template_vars)
                        ));
                    }
                }

                output.push('\n');
            }
        }
        output.push_str("</NpcsInRoom>\n");

        output
    }

    fn render_player_layer(&self) -> String {
        let mut output = String::from("<PlayerCharacter>\n");
        output.push_str("Name: ");
        output.push_str(&self.player.sheet.name);
        output.push_str("\n\n");
        output.push_str("Description: ");
        output.push_str(&render_template(
            &self.player.sheet.description,
            self.template_vars,
        ));
        output.push_str("\n\n");
        output.push_str("Personality: ");
        output.push_str(&render_template(
            &self.player.sheet.personality,
            self.template_vars,
        ));
        output.push_str("\n\n");
        output.push_str("Background: ");
        output.push_str(&render_template(
            &self.player.sheet.scenario,
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
            let sender = entry.sender.as_deref().unwrap_or("Narrator");
            history_text.push_str(&format!("{}: {}\n", sender, entry.text));
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

/// Sanitize injection patterns (`{{...}}`) → `[FILTERED]`.
pub(crate) fn sanitize_for_prompt(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut result = String::with_capacity(input.len());
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '{' && i + 1 < chars.len() && chars[i + 1] == '{' {
            let mut j = i + 2;
            while j + 1 < chars.len() {
                if chars[j] == '}' && chars[j + 1] == '}' {
                    break;
                }
                j += 1;
            }
            if j + 1 < chars.len() && j > i + 2 {
                result.push_str("[FILTERED]");
                i = j + 2;
            } else {
                result.push('{');
                result.push('{');
                i += 2;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}
