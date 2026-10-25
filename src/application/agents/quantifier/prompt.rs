//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Quantifier prompt construction

use crate::domain::model::template::TemplateVars;
use crate::domain::model::state::message_types::MessageType;
use crate::application::agents::quantifier::types::QuantifierPromptContext;

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
        let vars = TemplateVars::new(self.context.player_name);
        let mut prompt = vars.render(
            self.context
                .quantifier_prompt_override
                .as_deref()
                .unwrap_or(""),
        );

        if !prompt.is_empty() {
            prompt.push_str("\n\n");
        }
        prompt.push_str("<available_character_ids>\n");
        for character in self.context.all_known_characters {
            prompt.push_str(&format!(
                "  <Character id=\"{}\" name=\"{}\"/>\n",
                character.id, character.sheet.name
            ));
        }
        prompt.push_str("</available_character_ids>\n\n<available_rooms>\n");

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

        if let Some(nav_desc) = &self.context.room.navigation_description {
            prompt.push_str(&format!("  <Navigation>{nav_desc}</Navigation>\n"));
        }

        prompt.push_str("</CurrentRoom>\n\n");

        if !self.context.previous_characters_present.is_empty() {
            prompt.push_str("<PreviousCharactersPresent>\n");
            for character in self.context.previous_characters_present {
                prompt.push_str(&format!(
                    "  <Character id=\"{}\" name=\"{}\">{}</Character>\n",
                    character.id, character.sheet.name, character.sheet.description
                ));
            }
            prompt.push_str("</PreviousCharactersPresent>\n\n");
        }

        if !self.context.recent_history.is_empty() {
            prompt.push_str("<RecentHistory>\n");
            for entry in self.context.recent_history {
                let sender_label = match entry.message_type {
                    MessageType::Narration => "Narrator",
                    MessageType::Input => self.context.player_name,
                    MessageType::System => "System",
                };
                if sender_label.is_empty() {
                    prompt.push_str(&format!("  <Entry>{}</Entry>\n", entry.text));
                } else {
                    prompt.push_str(&format!(
                        "  <Entry sender=\"{}\">{}</Entry>\n",
                        sender_label, entry.text
                    ));
                }
            }
            prompt.push_str("</RecentHistory>\n\n");
        }

        prompt.push_str(&format!(
            "<LatestNarration>\n  {}: {}\n</LatestNarration>\n\n",
            self.context.player_name, self.context.player_action
        ));

        prompt.push_str(
            r#"Based on the context above, determine:
- Which Characters are present in the current room
- Whether the player actually entered, left, or remained in place

IMPORTANT: Base your decision ONLY on what happens in <LatestNarration>, not on what the player attempted in <RecentHistory>.

Respond ONLY with the JSON format specified in the system instructions.
"#,
        );

        prompt
    }
}
