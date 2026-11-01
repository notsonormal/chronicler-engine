//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Multi-stage prompt assembler: orchestrates layer rendering + context fitting.

use crate::error::EngineError;
use crate::domain::model::character::{CharacterCard, PersonaCard};
use crate::domain::model::history_cut::HistoryCut;
use crate::domain::model::map::Room;
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::settings::{NarrativePerspective, NarrativeTense};
use crate::domain::model::state::message_types::{StoryRow, MessageType};
use crate::domain::model::template::TemplateVars;
use crate::domain::model::world::WorldCard;
use crate::application::prompting::token_budget::{estimate_tokens, TokenBudget, CHARS_PER_TOKEN};
use crate::application::prompting::builders::sections::{
    build_post_history_prompt, build_system_prompt, render_known_character_entry,
    render_present_relationships, sanitize_for_prompt,
};
use crate::application::prompting::types::CharacterContext;

#[derive(Debug, Clone, PartialEq)]
pub struct AssembledPrompt {
    pub system_prompt: String,
    pub user_prompt: String,
    pub max_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct PromptContext<'a> {
    pub world: &'a WorldCard,
    /// Resolved posture (game-first, world fallback); `assemble` alone stamps it
    /// onto the template vars.
    pub narrative_perspective: NarrativePerspective,
    pub narrative_tense: NarrativeTense,
    pub room: &'a Room,
    pub characters: CharacterContext<'a>,
    pub persona: &'a PersonaCard,
    pub user_message: &'a str,
    pub history: &'a [StoryRow],
    pub template_vars: TemplateVars,
    /// Transient: never persisted; rendered as the final prompt layer.
    pub guide: Option<String>,
    /// Drops the player-character layer; the impersonate preset supplies the voice.
    pub impersonate: bool,
}

/// The layered prompt builder. Every call carries its own `TokenBudget`, so the
/// builder holds no window and no connection.
#[derive(Default)]
pub struct PromptAssembler;

impl PromptAssembler {
    pub fn assemble(
        &self,
        context: &PromptContext,
        preset: &PromptPreset,
        global_rules: &[String],
        budget: TokenBudget,
    ) -> Result<AssembledPrompt, EngineError> {
        let mut template_vars = context.template_vars.clone();
        Self::apply_posture(
            &mut template_vars,
            context.narrative_perspective,
            context.narrative_tense,
        );

        let system_prompt = build_system_prompt(preset, global_rules, &template_vars);
        let post_history_prompt = build_post_history_prompt(preset, &template_vars);

        let renderer = LayerRenderer {
            world: context.world,
            room: context.room,
            characters: context.characters,
            persona: context.persona,
            user_message: context.user_message,
            history: context.history,
            system_prompt,
            post_history_prompt,
            template_vars: &template_vars,
            guide: context.guide.as_deref(),
            impersonate: context.impersonate,
        };

        let (system, user, actual_max_tokens) = renderer.render_and_fit(budget)?;

        Ok(AssembledPrompt {
            system_prompt: system,
            user_prompt: user,
            max_tokens: actual_max_tokens,
        })
    }

    /// Sole writer of the narrative-voice template vars.
    fn apply_posture(
        template_vars: &mut TemplateVars,
        perspective: NarrativePerspective,
        tense: NarrativeTense,
    ) {
        template_vars.narrative_perspective = perspective.as_str().to_string();
        template_vars.narrative_tense = tense.as_str().to_string();
    }
}

impl PromptContext<'_> {
    #[allow(clippy::too_many_arguments)]
    pub fn new<'a>(
        world: &'a WorldCard,
        narrative_perspective: NarrativePerspective,
        narrative_tense: NarrativeTense,
        room: &'a Room,
        characters: CharacterContext<'a>,
        persona: &'a PersonaCard,
        user_message: &'a str,
        history: &'a [StoryRow],
    ) -> PromptContext<'a> {
        PromptContext {
            world,
            narrative_perspective,
            narrative_tense,
            room,
            characters,
            persona,
            user_message,
            history,
            template_vars: TemplateVars::from_persona(persona),
            guide: None,
            impersonate: false,
        }
    }

    pub fn with_guide(mut self, guide: Option<String>) -> Self {
        self.guide = guide;
        self
    }

    pub fn with_impersonate(mut self, impersonate: bool) -> Self {
        self.impersonate = impersonate;
        if impersonate {
            self.guide = None;
        }
        self
    }
}

struct LayerRenderer<'a> {
    world: &'a WorldCard,
    room: &'a Room,
    characters: CharacterContext<'a>,
    persona: &'a crate::domain::model::character::PersonaCard,
    user_message: &'a str,
    history: &'a [StoryRow],
    system_prompt: String,
    post_history_prompt: String,
    template_vars: &'a TemplateVars,
    guide: Option<&'a str>,
    impersonate: bool,
}

impl<'a> LayerRenderer<'a> {
    fn render_and_fit(&self, budget: TokenBudget) -> Result<(String, String, u32), EngineError> {
        let system = self.render_system_layer();
        let entries: Vec<String> = self
            .history
            .iter()
            .map(|e| self.render_history_entry(e))
            .collect();
        let cut = self.cut_history(&entries, budget, estimate_tokens(&system));
        let user = self.render_user_prompt(&entries, &cut);

        let max_tokens = budget.response_tokens(&system, &user)?;
        Ok((system, user, max_tokens))
    }

    fn cut_history(
        &self,
        entries: &[String],
        budget: TokenBudget,
        system_tokens: usize,
    ) -> HistoryCut {
        let entry_tokens: Vec<usize> = entries.iter().map(|e| estimate_tokens(e)).collect();
        let fixed_user = Self::join_layers(self.fixed_user_layers(Self::history_slot("")));
        let space = budget.history_space(system_tokens, estimate_tokens(&fixed_user));
        let mut cut = HistoryCut::of(&entry_tokens, space);

        // The overflow guard: a part outside the space (a long Player Input) can
        // still pass the input limit. It drops more whole Messages, once. It counts
        // characters, because a token cost rounds each entry up and the sum of those
        // would drop too few.
        let allowed_chars = budget.input_limit().saturating_sub(system_tokens) * CHARS_PER_TOKEN;
        let user = self.render_user_prompt(entries, &cut);
        let excess_chars = user.chars().count().saturating_sub(allowed_chars);
        if excess_chars > 0 {
            let entry_chars: Vec<usize> = entries.iter().map(|e| e.chars().count()).collect();
            cut.drop_to_free(&entry_chars, excess_chars);
        }
        cut
    }

    fn render_user_prompt(&self, entries: &[String], cut: &HistoryCut) -> String {
        let layers = self
            .fixed_user_layers(self.render_history_layer(entries, cut))
            .into_iter()
            .chain(self.tail_layers());
        Self::join_layers(layers)
    }

    /// The layers that do not follow the Player Input. With an empty history slot, they
    /// are the fixed part that the history space subtracts.
    fn fixed_user_layers(&self, history_layer: String) -> Vec<String> {
        vec![
            self.render_game_state_layer(),
            self.render_character_cards_layer(),
            self.render_persona_layer(),
            self.render_world_info_layer(),
            history_layer,
            self.post_history_prompt.clone(),
        ]
    }

    /// The layers a new Input changes: they stay out of the history space.
    fn tail_layers(&self) -> Vec<String> {
        vec![self.render_user_layer(), self.render_guide_layer()]
    }

    fn join_layers(layers: impl IntoIterator<Item = String>) -> String {
        let user = layers
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");
        format!("{user}\n\n")
    }

    fn render_system_layer(&self) -> String {
        self.system_prompt.clone()
    }

    fn render_game_state_layer(&self) -> String {
        let mut output = String::from("<GameState>\n");
        output.push_str("Current Room: ");
        output.push_str(&self.room.name);
        output.push_str("\n\n");
        output.push_str(&self.template_vars.render(&self.room.description));
        output.push_str("\n\n");
        output.push_str("</GameState>\n");
        output
    }

    fn render_character_cards_layer(&self) -> String {
        let mut output = String::new();
        let in_area_ids: std::collections::HashSet<_> = self
            .characters
            .characters_present
            .iter()
            .map(|n| n.id.as_str())
            .collect();

        output.push_str("<KnownCharacters>");
        if self.characters.all_characters.is_empty() {
            output.push_str("No characters in this world.\n");
        } else {
            for character in CharacterCard::in_id_order(self.characters.all_characters) {
                output.push_str(&render_known_character_entry(
                    character,
                    &in_area_ids,
                    self.template_vars,
                ));
            }
        }
        output.push_str("</KnownCharacters>\n\n");

        output.push_str("<CharactersPresent>");
        if self.characters.characters_present.is_empty() {
            output.push_str("No Characters are present in this Room.\n");
        } else {
            for character in CharacterCard::in_id_order(self.characters.characters_present) {
                output.push_str("--- ");
                output.push_str(&character.sheet.name);
                output.push_str(" ---\n");
                output.push_str("Description: ");
                output.push_str(&self.template_vars.render(&character.sheet.description));
                output.push('\n');
                output.push_str("Personality: ");
                output.push_str(&self.template_vars.render(&character.sheet.personality));
                output.push('\n');
                if !character.sheet.scenario.is_empty() {
                    // The card field's own name, not the glossary's Scenario
                    // (the start of a Game): the label is text for the model.
                    output.push_str("Scenario: ");
                    output.push_str(&self.template_vars.render(&character.sheet.scenario));
                    output.push('\n');
                }

                let relations_block = render_present_relationships(
                    character,
                    self.characters.all_characters,
                    &in_area_ids,
                    self.template_vars,
                );
                if let Some(relations_block) = relations_block {
                    output.push_str(&relations_block);
                }

                output.push('\n');
            }
        }
        output.push_str("</CharactersPresent>\n");

        output
    }

    fn render_persona_layer(&self) -> String {
        if self.impersonate {
            return String::new();
        }
        let mut output = String::from("<Persona>\n");
        output.push_str("Name: ");
        output.push_str(&self.persona.sheet.name);
        output.push_str("\n\n");
        output.push_str("Description: ");
        output.push_str(&self.template_vars.render(&self.persona.sheet.description));
        output.push_str("\n\n");
        output.push_str("Personality: ");
        output.push_str(&self.template_vars.render(&self.persona.sheet.personality));
        output.push_str("\n\n");
        output.push_str("Background: ");
        output.push_str(&self.template_vars.render(&self.persona.sheet.scenario));
        output.push('\n');

        output.push_str("</Persona>\n");
        output
    }

    fn render_world_info_layer(&self) -> String {
        let mut output = String::from("<WorldLore>\n");
        output.push_str("World: ");
        output.push_str(&self.world.name);
        output.push_str("\n\n");
        output.push_str(&self.template_vars.render(&self.world.description));
        output.push_str("\n\n");

        output.push_str("</WorldLore>\n");
        output
    }

    fn render_history_entry(&self, entry: &StoryRow) -> String {
        let sender = match entry.message_type {
            MessageType::Narration => "Narrator",
            MessageType::Input => self.persona.sheet.name.as_str(),
            MessageType::System => "System",
        };
        format!("{sender}: {}\n", entry.text)
    }

    fn history_slot(body: &str) -> String {
        format!("<ConversationHistory>\n{body}</ConversationHistory>\n")
    }

    fn render_history_layer(&self, entries: &[String], cut: &HistoryCut) -> String {
        if entries.is_empty() {
            return Self::history_slot(
                "(No history yet - this is the start of the conversation)\n",
            );
        }
        if cut.kept_from() >= entries.len() {
            return Self::history_slot("(History truncated to fit context window)\n");
        }
        Self::history_slot(&entries[cut.kept_from()..].concat())
    }

    fn render_user_layer(&self) -> String {
        let sanitized = sanitize_for_prompt(self.user_message);
        Self::render_non_blank_layer("<PlayerInput>", "", &sanitized, "</PlayerInput>")
    }

    /// A blank body renders no tags: an empty tagged block reads as an empty
    /// instruction. The lead stands in front of the body, and a blank body drops it.
    fn render_non_blank_layer(open: &str, lead: &str, body: &str, close: &str) -> String {
        if body.trim().is_empty() {
            return String::new();
        }
        format!("{open}\n{lead}{body}\n{close}\n")
    }

    fn render_guide_layer(&self) -> String {
        Self::render_non_blank_layer(
            "<Guide>",
            "Take the following into special consideration for your next message: ",
            self.guide.unwrap_or_default().trim(),
            "</Guide>",
        )
    }
}
