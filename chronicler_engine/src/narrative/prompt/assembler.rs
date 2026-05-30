use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::map::Room;
use crate::model::prompt_preset::PromptPreset;
use crate::model::state::MessageEntry;
use crate::model::world::WorldCard;
use crate::narrative::prompt::budget;
use crate::narrative::prompt::budget::truncate_to_budget;
use crate::narrative::prompt::context::fit_messages_to_context;
use crate::narrative::prompt::sanitize::sanitize_for_prompt;
use crate::narrative::prompt::types::PromptContext;

#[derive(Debug, Clone, PartialEq)]
pub struct AssembledPrompt {
    pub system_prompt: String,
    pub user_prompt: String,
    pub max_tokens: u32,
}

/// Decouples prompt construction from LLM transport.
pub trait PromptAssembler: Send + Sync {
    fn assemble(
        &self,
        context: &PromptContext,
        preset: &PromptPreset,
        global_rules: &[String],
        response_length: Option<&str>,
    ) -> Result<AssembledPrompt, EngineError>;
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

impl PromptAssembler for LayeredPromptAssembler {
    fn assemble(
        &self,
        context: &PromptContext,
        preset: &PromptPreset,
        global_rules: &[String],
        response_length: Option<&str>,
    ) -> Result<AssembledPrompt, EngineError> {
        let system_prompt = build_system_prompt(preset, global_rules);
        let post_history_prompt = build_post_history_prompt(preset, response_length);

        let renderer = LayerRenderer {
            world: context.world,
            room: context.room,
            all_npcs: context.all_npcs,
            npcs_in_area: context.npcs_in_area,
            player: context.player,
            user_message: context.user_message,
            history: context.history,
            system_prompt,
            post_history_prompt,
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

// ---------------------------------------------------------------------------
// System / post-history prompt builders (direct from preset — no delimiter)
// ---------------------------------------------------------------------------

fn build_system_prompt(preset: &PromptPreset, global_rules: &[String]) -> String {
    let mut parts: Vec<String> = Vec::new();

    push_section(&mut parts, preset.role.as_deref(), "role");
    push_section(&mut parts, preset.instructions.as_deref(), "instructions");

    if !global_rules.is_empty() {
        let rules_text = global_rules
            .iter()
            .map(|r| format!("- {r}"))
            .collect::<Vec<_>>()
            .join("\n");
        parts.push(wrap_xml(&rules_text, "global_rules"));
    }

    parts.join("\n\n")
}

fn build_post_history_prompt(preset: &PromptPreset, response_length: Option<&str>) -> String {
    let mut parts: Vec<String> = Vec::new();

    push_section(&mut parts, preset.writing_style.as_deref(), "writing_style");

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

// ---------------------------------------------------------------------------
// Layer rendering (migrated from PromptBuilder)
// ---------------------------------------------------------------------------

struct LayerRenderer<'a> {
    world: &'a WorldCard,
    room: &'a Room,
    all_npcs: &'a [NpcCard],
    npcs_in_area: &'a [NpcCard],
    player: &'a crate::model::character::PlayerCard,
    user_message: &'a str,
    history: &'a [MessageEntry],
    system_prompt: String,
    post_history_prompt: String,
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
        output.push_str(&self.room.description);
        output.push_str("\n\n");
        output.push_str("</GameState>\n");
        output
    }

    fn render_npc_cards_layer(&self) -> String {
        let mut output = String::new();
        let in_area_ids: std::collections::HashSet<_> =
            self.npcs_in_area.iter().map(|n| n.id.as_str()).collect();

        // Section 1: All known NPCs with condensed cards
        output.push_str("<KnownNpcs>\n");
        if self.all_npcs.is_empty() {
            output.push_str("No characters in this world.\n");
        } else {
            for npc in self.all_npcs {
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

                // Indent each line of the summary
                for line in summary_text.lines() {
                    output.push_str(&format!("  {line}\n"));
                }
                output.push('\n');
            }
        }
        output.push_str("</KnownNpcs>\n\n");

        // Section 2: Present NPCs with full character cards
        output.push_str("<NpcsInRoom>\n");
        if self.npcs_in_area.is_empty() {
            output.push_str("No NPCs are present in this location.\n");
        } else {
            for npc in self.npcs_in_area {
                output.push_str("--- ");
                output.push_str(&npc.sheet.name);
                output.push_str(" ---\n");
                output.push_str("Description: ");
                output.push_str(&npc.sheet.description);
                output.push('\n');
                output.push_str("Personality: ");
                output.push_str(&npc.sheet.personality);
                output.push('\n');
                if !npc.sheet.scenario.is_empty() {
                    output.push_str("Context: ");
                    output.push_str(&npc.sheet.scenario);
                    output.push('\n');
                }

                // Relationships with other NPCs present in the room
                let present_relations: Vec<_> = npc
                    .relationships
                    .iter()
                    .filter(|r| in_area_ids.contains(r.with.as_str()))
                    .collect();

                if !present_relations.is_empty() {
                    output.push_str("Relationships:\n");
                    for rel in present_relations {
                        let partner_name = self
                            .all_npcs
                            .iter()
                            .find(|n| n.id == rel.with)
                            .map(|n| n.sheet.name.as_str())
                            .unwrap_or(&rel.with);
                        output.push_str(&format!("  → {}: {}\n", partner_name, rel.display_text()));
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
        output.push_str(&self.player.sheet.description);
        output.push_str("\n\n");
        output.push_str("Personality: ");
        output.push_str(&self.player.sheet.personality);
        output.push_str("\n\n");
        output.push_str("Background: ");
        output.push_str(&self.player.sheet.scenario);
        output.push('\n');

        output.push_str("</PlayerCharacter>\n");
        output
    }

    fn render_world_info_layer(&self) -> String {
        let mut output = String::from("<WorldLore>\n");
        output.push_str("World: ");
        output.push_str(&self.world.name);
        output.push_str("\n\n");
        output.push_str(&self.world.description);
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

        // Truncate to budget
        let truncated = truncate_to_budget(&history_text, budget::MAX_HISTORY_TOKENS as usize);
        output.push_str(&truncated);

        output.push_str("</ConversationHistory>\n");
        output
    }

    fn render_user_layer(&self) -> String {
        let mut output = String::from("<PlayerInput>\n");
        output.push_str(&sanitize_for_prompt(self.user_message));
        output.push_str("\n</PlayerInput>\n");

        output
    }
}
