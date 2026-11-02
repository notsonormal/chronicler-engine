//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Multi-stage prompt assembler: orchestrates layer rendering + context fitting.

use crate::error::EngineError;
use crate::domain::model::character::{CharacterCard, PersonaCard};
use crate::domain::model::history_cut::HistoryCut;
use crate::domain::model::map::Room;
use crate::domain::model::prompt_part::{PromptPart, PromptRole};
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::settings::{NarrativePerspective, NarrativeTense};
use crate::domain::model::state::message_types::{StoryRow, MessageType};
use crate::domain::model::template::TemplateVars;
use crate::domain::model::world::WorldCard;
use crate::application::prompting::token_budget::{
    estimate_tokens, TokenBudget, CHARS_PER_TOKEN, MESSAGE_OVERHEAD_TOKENS,
};
use crate::application::prompting::builders::sections::{
    build_post_history_prompt, build_system_prompt, render_known_character_entry,
    render_present_relationships, sanitize_for_prompt,
};
use crate::application::prompting::types::CharacterContext;

#[derive(Debug, Clone, PartialEq)]
pub struct AssembledPrompt {
    /// The system part first, then the user part with the blocks before the
    /// history, then one part for each history entry.
    pub parts: Vec<PromptPart>,
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

        let (parts, max_tokens) = renderer.render_and_fit(budget)?;

        Ok(AssembledPrompt { parts, max_tokens })
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

const HISTORY_OPEN: &str = "<ConversationHistory>\n";
const HISTORY_CLOSE: &str = "\n</ConversationHistory>";
const LAST_MESSAGE_OPEN: &str = "<LastMessage>\n";
const LAST_MESSAGE_CLOSE: &str = "\n</LastMessage>";
/// The user parts that exist whatever the history: the first one with the blocks
/// before the history, and a closing one with the blocks after it. The closing
/// part counts as a new part, the worst case, though it can join the last entry.
const FIXED_USER_PARTS: usize = 2;

impl<'a> LayerRenderer<'a> {
    fn render_and_fit(&self, budget: TokenBudget) -> Result<(Vec<PromptPart>, u32), EngineError> {
        let system = self.render_system_layer();
        let system_tokens = Self::part_tokens(&system);
        let entries: Vec<PromptPart> = self
            .history
            .iter()
            .map(|e| self.render_history_entry(e))
            .collect();
        let parts = self.render_cut_prompt(&system, &entries, budget, system_tokens);

        let tokens_after_system = parts
            .iter()
            .skip(1)
            .map(|part| Self::part_tokens(&part.text))
            .sum();
        let max_tokens = budget.response_tokens(system_tokens, tokens_after_system)?;
        Ok((parts, max_tokens))
    }

    /// It renders once more only when the overflow guard moves the token cut.
    fn render_cut_prompt(
        &self,
        system: &str,
        entries: &[PromptPart],
        budget: TokenBudget,
        system_tokens: usize,
    ) -> Vec<PromptPart> {
        let entry_tokens: Vec<usize> = entries.iter().map(|e| Self::part_tokens(&e.text)).collect();
        let space = budget.history_space(system_tokens, self.fixed_tokens());
        let mut cut = HistoryCut::of(&entry_tokens, space);

        // The overflow guard: a part outside the space (a long Player Input) can
        // still pass the input limit. It drops more whole Messages, once. It counts
        // characters, because a token cost rounds each entry up and the sum of those
        // would drop too few.
        let allowed_chars = budget.input_limit().saturating_sub(system_tokens) * CHARS_PER_TOKEN;
        let mut parts = self.render_prompt(system, entries, &cut);
        let excess_chars = Self::chars_after_system(&parts).saturating_sub(allowed_chars);
        if excess_chars > 0 {
            let kept_from = cut.kept_from();
            let entry_chars: Vec<usize> =
                entries.iter().map(|e| Self::part_chars(&e.text)).collect();
            cut.drop_to_free(&entry_chars, excess_chars);
            if cut.kept_from() > kept_from {
                parts = self.render_prompt(system, entries, &cut);
            }
        }
        parts
    }

    /// The cost of one prompt part: its text and the chat-template overhead.
    fn part_tokens(text: &str) -> usize {
        estimate_tokens(text) + MESSAGE_OVERHEAD_TOKENS
    }

    /// `part_tokens` in characters, for the overflow guard.
    fn part_chars(text: &str) -> usize {
        text.chars().count() + MESSAGE_OVERHEAD_TOKENS * CHARS_PER_TOKEN
    }

    fn chars_after_system(parts: &[PromptPart]) -> usize {
        parts
            .iter()
            .skip(1)
            .map(|part| Self::part_chars(&part.text))
            .sum()
    }

    /// The Player Input and the Guide stay out of it: a new Input changes them.
    fn fixed_tokens(&self) -> usize {
        let tags = format!("{HISTORY_OPEN}{HISTORY_CLOSE}{LAST_MESSAGE_OPEN}{LAST_MESSAGE_CLOSE}");
        estimate_tokens(&Self::join_layers(self.before_layers()))
            + estimate_tokens(&Self::join_layers(self.fixed_after_layers()))
            + estimate_tokens(&tags)
            + FIXED_USER_PARTS * MESSAGE_OVERHEAD_TOKENS
    }

    /// The blocks after the history join the last history entry when it is a user
    /// message, so that two user messages never follow each other.
    fn render_prompt(
        &self,
        system: &str,
        entries: &[PromptPart],
        cut: &HistoryCut,
    ) -> Vec<PromptPart> {
        let kept = entries.get(cut.kept_from()..).unwrap_or_default();
        let mut before = self.before_layers();
        if kept.is_empty() {
            before.push(Self::empty_history_layer(entries.is_empty()));
        }
        let mut parts = vec![
            PromptPart::system(system),
            PromptPart::user(Self::join_layers(before)),
        ];
        parts.extend(Self::wrap_history(kept));
        let after = Self::join_layers(self.after_layers(kept.last()));
        if after.is_empty() {
            return parts;
        }
        match parts.last_mut() {
            Some(last) if last.role == PromptRole::User => {
                last.text = format!("{}\n\n{after}", last.text);
            }
            _ => parts.push(PromptPart::user(after)),
        }
        parts
    }

    fn before_layers(&self) -> Vec<String> {
        vec![
            self.render_known_characters_layer(),
            self.render_persona_layer(),
            self.render_world_info_layer(),
        ]
    }

    fn fixed_after_layers(&self) -> Vec<String> {
        vec![
            self.render_game_state_layer(),
            self.render_characters_present_layer(),
            self.post_history_prompt.clone(),
        ]
    }

    /// A new Input changes the Player Input and the Guide, so they stay out of the
    /// history space. The Player Input is left out when the last history entry is
    /// that Input already.
    fn after_layers(&self, last_kept: Option<&PromptPart>) -> Vec<String> {
        let input_in_history = last_kept.is_some_and(|entry| {
            entry.role == PromptRole::User && entry.text.trim() == self.user_message.trim()
        });
        let mut layers = self.fixed_after_layers();
        if !input_in_history {
            layers.push(self.render_user_layer());
        }
        layers.push(self.render_guide_layer());
        layers
    }

    /// The open tag leads the first entry, the close tag ends the second-to-last,
    /// and the last entry has its own tag.
    /// A history of one entry has only the last-message tag.
    fn wrap_history(kept: &[PromptPart]) -> Vec<PromptPart> {
        let mut messages = kept.to_vec();
        let Some(last) = messages.len().checked_sub(1) else {
            return messages;
        };
        if last > 0 {
            messages[0].text = format!("{HISTORY_OPEN}{}", messages[0].text);
            messages[last - 1].text = format!("{}{HISTORY_CLOSE}", messages[last - 1].text);
        }
        messages[last].text = format!(
            "{LAST_MESSAGE_OPEN}{}{LAST_MESSAGE_CLOSE}",
            messages[last].text
        );
        messages
    }

    fn join_layers(layers: impl IntoIterator<Item = String>) -> String {
        layers
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n")
            .trim_end()
            .to_string()
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

    fn render_known_characters_layer(&self) -> String {
        let mut output = String::new();

        output.push_str("<KnownCharacters>");
        if self.characters.all_characters.is_empty() {
            output.push_str("No characters in this world.\n");
        } else {
            for character in CharacterCard::in_id_order(self.characters.all_characters) {
                output.push_str(&render_known_character_entry(character, self.template_vars));
            }
        }
        output.push_str("</KnownCharacters>\n");
        output
    }

    fn render_characters_present_layer(&self) -> String {
        let mut output = String::new();
        let present_ids: std::collections::HashSet<_> = self
            .characters
            .characters_present
            .iter()
            .map(|n| n.id.as_str())
            .collect();

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
                    &present_ids,
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

    fn render_history_entry(&self, entry: &StoryRow) -> PromptPart {
        let role = match entry.message_type {
            MessageType::Narration => PromptRole::Assistant,
            MessageType::Input => PromptRole::User,
            MessageType::System => PromptRole::System,
        };
        PromptPart {
            role,
            text: entry.text.clone(),
        }
    }

    fn empty_history_layer(no_history_yet: bool) -> String {
        let body = if no_history_yet {
            "(No history yet - this is the start of the conversation)"
        } else {
            "(History truncated to fit context window)"
        };
        format!("{HISTORY_OPEN}{body}{HISTORY_CLOSE}\n")
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
