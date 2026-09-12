//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Options prompt construction

use crate::domain::model::template::TemplateVars;
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::utils::template::render_template;
use crate::application::agents::options::types::OptionsPromptContext;

/// Used when the active options preset is unreadable so the agent still
/// produces parseable output instead of a bare scene dump.
const FALLBACK_SYSTEM_PROMPT: &str = "You are an options generator for a text adventure game. \
You produce pickable next-action suggestions the player can choose from. Provide \
{{option_count}} brief distinct single-sentence suggestions for the next story beat on \
{{user}} perspective. Each suggestion surrounded by `<suggestion>` tags. Do not include \
any other content in your response.";

pub struct OptionsPromptBuilder<'a> {
    context: OptionsPromptContext<'a>,
}

impl<'a> OptionsPromptBuilder<'a> {
    pub fn new(context: OptionsPromptContext<'a>) -> Self {
        Self { context }
    }

    pub fn build(&self) -> (String, String) {
        (self.build_system_prompt(), self.build_user_prompt())
    }

    fn build_system_prompt(&self) -> String {
        let vars = TemplateVars::new(self.context.player_name);
        let base = self
            .context
            .options_prompt_override
            .as_deref()
            .filter(|text| !text.trim().is_empty())
            .unwrap_or(FALLBACK_SYSTEM_PROMPT);
        render_template(base, &vars)
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

        if !self.context.npcs_in_area.is_empty() {
            prompt.push_str("<NpcsInArea>\n");
            for npc in self.context.npcs_in_area {
                prompt.push_str(&format!(
                    "  <Npc id=\"{}\" name=\"{}\">{}</Npc>\n",
                    npc.id, npc.sheet.name, npc.sheet.description
                ));
            }
            prompt.push_str("</NpcsInArea>\n\n");
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
            "Based on the scene above, respond with exactly {} options in the format the \
system instructions specify. Each option is a possible player action — something {} could \
do next — not a story outcome.\n",
            self.context.option_count, self.context.player_name
        ));

        prompt
    }
}
