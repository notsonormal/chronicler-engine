use crate::error::EngineError;
use crate::narrative::prompt::budget;
use crate::narrative::prompt::budget::{estimate_tokens, truncate_to_budget};
use crate::narrative::prompt::context::fit_messages_to_context;
use crate::narrative::prompt::sanitize::sanitize_for_prompt;
use crate::narrative::prompt::types::{PromptBuilder, PromptContext};

impl<'a> PromptBuilder<'a> {
    pub fn from_context(context: &PromptContext<'a>) -> Self {
        Self {
            world: context.world,
            room: context.room,
            all_npcs: context.all_npcs,
            npcs_in_area: context.npcs_in_area,
            player: context.player,
            user_message: context.user_message,
            history: context.history,
            max_context_tokens: None,
            requested_max_tokens: None,
            system_prompt: context.system_prompt.clone(),
        }
    }

    pub fn with_max_context_tokens(mut self, max: u32) -> Self {
        self.max_context_tokens = Some(max);
        self
    }

    pub fn with_max_tokens(mut self, max: u32) -> Self {
        self.requested_max_tokens = Some(max);
        self
    }

    pub fn build(&self) -> std::result::Result<(String, u32), EngineError> {
        let (system, user, max_tokens) = self.build_split()?;
        let prompt = format!("{system}\n\n---\n\n{user}");
        Ok((prompt, max_tokens))
    }

    pub fn build_split(&self) -> std::result::Result<(String, String, u32), EngineError> {
        let system = self.render_system_layer();

        let mut user = String::new();
        user.push_str(&self.render_game_state_layer());
        user.push_str("\n\n");
        user.push_str(&self.render_npc_cards_layer());
        user.push_str("\n\n");
        user.push_str(&self.render_player_layer());
        user.push_str("\n\n");
        user.push_str(&self.render_world_info_layer());
        user.push_str("\n\n");
        user.push_str(&self.render_history_layer());
        user.push_str("\n\n");
        user.push_str(&self.render_user_layer());
        user.push_str("\n\n");
        // Layer 7 (output format) removed — now part of system prompt via preset

        if let Some(max_context) = self.max_context_tokens {
            let (fitted_system, fitted_user, actual_max_tokens) =
                fit_messages_to_context(&system, &user, max_context, self.requested_max_tokens)?;
            return Ok((fitted_system, fitted_user, actual_max_tokens));
        }

        // Fallback: verify against default budget
        let total_tokens = estimate_tokens(&system) + estimate_tokens(&user);
        if total_tokens > budget::MAX_CONTEXT_TOKENS as usize {
            return Err(EngineError::ContextOverflow {
                requested: total_tokens,
                max: budget::MAX_CONTEXT_TOKENS as usize,
            });
        }

        let max_tokens = self
            .requested_max_tokens
            .unwrap_or(budget::MAX_RESPONSE_TOKENS);
        Ok((system, user, max_tokens))
    }

    pub fn build_system_only(&self) -> String {
        self.render_system_layer()
    }

    pub fn build_user_only(&self) -> String {
        let mut user = String::new();
        user.push_str(&self.render_game_state_layer());
        user.push_str("\n\n");
        user.push_str(&self.render_npc_cards_layer());
        user.push_str("\n\n");
        user.push_str(&self.render_player_layer());
        user.push_str("\n\n");
        user.push_str(&self.render_world_info_layer());
        user.push_str("\n\n");
        user.push_str(&self.render_history_layer());
        user.push_str("\n\n");
        user.push_str(&self.render_user_layer());
        user.push_str("\n\n");
        // Layer 7 (output format) removed — now part of system prompt via preset
        user
    }

    /// Layer 0: System prompt - assembled XML from preset (includes global rules and response length)
    fn render_system_layer(&self) -> String {
        self.system_prompt.clone()
    }

    /// Layer 1: Game state - room name, description, player inventory
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

    /// Layer 2: NPC cards - known NPCs roster + in-room NPCs with full detail
    fn render_npc_cards_layer(&self) -> String {
        let mut output = String::new();

        // Section 1: All known NPCs with condensed cards
        output.push_str("<KnownNpcs>\n");
        if self.all_npcs.is_empty() {
            output.push_str("No characters in this world.\n");
        } else {
            let in_area_ids: std::collections::HashSet<_> =
                self.npcs_in_area.iter().map(|n| n.id.as_str()).collect();

            for npc in self.all_npcs {
                let presence = if in_area_ids.contains(npc.id.as_str()) {
                    "(in room)"
                } else {
                    "(elsewhere)"
                };
                output.push_str(&format!("- {} {}\n", npc.sheet.name, presence));

                // Use explicit summary if available, otherwise first 3 lines of description
                let summary_text = npc
                    .sheet
                    .summary
                    .as_deref()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
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
            let in_area_ids: std::collections::HashSet<_> =
                self.npcs_in_area.iter().map(|n| n.id.as_str()).collect();

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

    /// Layer 3: Player persona - from player card
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

    /// Layer 4: World info - global rules as lorebook
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

    /// Layer 5: Full history - conversation log with truncation
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

    /// Layer 6: User message - current input
    fn render_user_layer(&self) -> String {
        let mut output = String::from("<PlayerInput>\n");
        output.push_str(&sanitize_for_prompt(self.user_message));
        output.push_str("\n</PlayerInput>\n");

        output
    }
}
