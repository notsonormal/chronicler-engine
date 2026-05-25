use crate::narrative::agents::quantifier::types::QuantifierPromptContext;

/// [DOC: docs/reference/quantifier_prompt.md]
pub struct QuantifierPromptBuilder<'a> {
    context: QuantifierPromptContext<'a>,
}

impl<'a> QuantifierPromptBuilder<'a> {
    pub fn new(context: QuantifierPromptContext<'a>) -> Self {
        Self { context }
    }

    pub fn build(&self) -> (String, String) {
        (self.build_system_prompt(), self.build_user_prompt())
    }

    fn build_system_prompt(&self) -> String {
        let mut prompt = self
            .context
            .quantifier_prompt_override
            .clone()
            .unwrap_or_default();

        if !prompt.is_empty() {
            prompt.push_str("\n\n");
        }
        prompt.push_str("<available_npc_ids>\n");
        for npc in self.context.all_known_npcs {
            prompt.push_str(&format!(
                "  <Npc id=\"{}\" name=\"{}\"/>\n",
                npc.id, npc.sheet.name
            ));
        }
        prompt.push_str("</available_npc_ids>\n\n<available_rooms>\n");

        for room in self.context.all_rooms {
            prompt.push_str(&format!(
                "  <Room id=\"{}\" name=\"{}\"/>\n",
                room.id, room.name
            ));
        }

        prompt.push_str("</available_rooms>\n");

        prompt
    }

    fn build_user_prompt(&self) -> String {
        let mut prompt = String::new();

        prompt.push_str("<CurrentRoom>\n");
        prompt.push_str(&format!("  <Name>{}</Name>\n", self.context.room.name));
        prompt.push_str(&format!(
            "  <Description>{}</Description>\n",
            self.context.room.description
        ));

        // Include navigation hint to improve movement detection
        if let Some(nav_desc) = &self.context.room.navigation_description {
            prompt.push_str(&format!("  <Navigation>{nav_desc}</Navigation>\n"));
        }

        prompt.push_str("</CurrentRoom>\n\n");

        if !self.context.previous_room_npcs.is_empty() {
            prompt.push_str("<PreviousRoomNpcs>\n");
            for npc in self.context.previous_room_npcs {
                prompt.push_str(&format!(
                    "  <Npc id=\"{}\" name=\"{}\">{}</Npc>\n",
                    npc.id, npc.sheet.name, npc.sheet.description
                ));
            }
            prompt.push_str("</PreviousRoomNpcs>\n\n");
        }

        if !self.context.recent_history.is_empty() {
            prompt.push_str("<RecentHistory>\n");
            for entry in self.context.recent_history {
                let sender = entry.sender.as_deref().unwrap_or("Narrator");
                prompt.push_str(&format!(
                    "  <Entry sender=\"{}\">{}</Entry>\n",
                    sender, entry.text
                ));
            }
            prompt.push_str("</RecentHistory>\n\n");
        }

        prompt.push_str(&format!(
            "<LatestNarration>\n  {}: {}\n</LatestNarration>\n\n",
            self.context.player_name, self.context.player_action
        ));

        prompt.push_str(
            r#"Based on the context above, determine:
- Which NPCs are present in the current room
- Whether the player actually entered, left, or remained in place

IMPORTANT: Base your decision ONLY on what happens in <LatestNarration>, not on what the player attempted in <RecentHistory>.

Respond ONLY with the JSON format specified in the system instructions.
"#,
        );

        prompt
    }
}
