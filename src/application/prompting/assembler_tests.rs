//! Seam contract of the PromptAssembler, the pure box that turns one prompt context into the prompt of one call.

use crate::application::prompting::token_budget::{
    estimate_tokens, TokenBudget, MAX_CONTEXT_TOKENS, MESSAGE_OVERHEAD_TOKENS,
    SAFETY_MARGIN_TOKENS, STORY_MEMORY_CHARS,
};
use crate::application::prompting::{AssembledPrompt, CharacterContext, PromptAssembler, PromptContext};
use crate::domain::model::character::{CharacterSheet, CharacterCard, PersonaCard, Relationship};
use crate::domain::model::map::MapDef;
use crate::domain::model::prompt_part::{PromptPart, PromptRole};
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::{
    NarrativePerspective, NarrativeTense, DEFAULT_MAX_HISTORY_TOKENS, DEFAULT_MAX_TOKENS,
};
use crate::domain::model::state::message_types::{MessageType, StoryRow};
use crate::domain::model::world::WorldCard;
use crate::error::EngineError;
use crate::test_support::fixtures::{
    default_story_row, prompt_block, TestMap, TestCharacter, TestPersona, TestWorld,
};
use crate::test_support::scripted_model::joined_text_of_parts;

const WRITING_STYLE: &str = "{{narrative_perspective}}-person limited perspective, {{narrative_tense}} tense narrative prose.";
const NO_FIELDS_PRESET: &str = "no fields";
const OUTPUT_FORMAT: &str = "Write one paragraph.\n\nResponse Length:\nShort";

fn preset() -> PromptPreset {
    PromptPreset {
        id: "system_test".to_string(),
        name: "Test System".to_string(),
        role: Some("You are a test narrator.".to_string()),
        instructions: Some("Follow the world's rules.".to_string()),
        writing_style: Some(WRITING_STYLE.to_string()),
        output_format: Some(OUTPUT_FORMAT.to_string()),
        allowed_modes: vec![],
        is_default: true,
        preset_type: PresetType::System,
        ..PromptPreset::default()
    }
}

fn empty_preset() -> PromptPreset {
    PromptPreset {
        id: NO_FIELDS_PRESET.to_string(),
        name: NO_FIELDS_PRESET.to_string(),
        role: None,
        instructions: None,
        writing_style: None,
        output_format: None,
        allowed_modes: vec![],
        is_default: false,
        preset_type: PresetType::System,
        ..PromptPreset::default()
    }
}

fn impersonate_preset() -> PromptPreset {
    PromptPreset {
        id: "impersonate_test".to_string(),
        name: "Test Impersonate".to_string(),
        role: Some("Write as {{user}}. {{persona_description}}".to_string()),
        instructions: Some("{{persona_personality}} / {{persona_background}}".to_string()),
        writing_style: None,
        output_format: None,
        allowed_modes: vec![],
        is_default: true,
        preset_type: PresetType::Impersonate,
        ..PromptPreset::default()
    }
}

fn character_card(
    id: &str,
    name: &str,
    summary: Option<&str>,
    description: &str,
    relationships: Vec<Relationship>,
) -> CharacterCard {
    CharacterCard {
        id: id.to_string(),
        sheet: CharacterSheet {
            name: name.to_string(),
            description: description.to_string(),
            personality: "Steady.".to_string(),
            scenario: "At the inn.".to_string(),
            example_dialogue: String::new(),
            summary: summary.map(|text| text.to_string()),
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![],
        relationships,
    }
}

struct Fixture {
    world: WorldCard,
    map: MapDef,
    persona: PersonaCard,
    characters: Vec<CharacterCard>,
    present: Vec<CharacterCard>,
    history: Vec<StoryRow>,
    global_rules: Vec<String>,
}

impl Fixture {
    fn new() -> Self {
        let characters = vec![TestCharacter::named("character_1", "Test Character")];
        Self {
            world: TestWorld::minimal(),
            map: TestMap::single_room("room_1"),
            persona: TestPersona::standard(),
            present: characters.clone(),
            characters,
            history: vec![StoryRow {
                id: 1,
                text: "You enter the inn.".to_string(),
                message_type: MessageType::Narration,
                timestamp: chrono::Utc::now(),
                ..default_story_row(MessageType::Narration)
            }],
            global_rules: vec!["No fast travel.".to_string()],
        }
    }

    fn with_characters(
        mut self,
        characters: Vec<CharacterCard>,
        present: Vec<CharacterCard>,
    ) -> Self {
        self.characters = characters;
        self.present = present;
        self
    }

    fn history_of(&self, count: usize) -> Vec<StoryRow> {
        (0..count)
            .map(|index| StoryRow {
                id: index as u64,
                text: format!("Entry {index} of the story, told at length so it spends tokens."),
                message_type: MessageType::Narration,
                timestamp: chrono::Utc::now(),
                ..default_story_row(MessageType::Narration)
            })
            .collect()
    }

    /// Entries whose text holds a newline, so a cut in the middle of one would show.
    fn multiline_history_of(&self, count: usize) -> Vec<StoryRow> {
        self.history_of(count)
            .into_iter()
            .map(|mut row| {
                row.text = format!(
                    "Entry {} first line.\nEntry {} second line.",
                    row.id, row.id
                );
                row
            })
            .collect()
    }

    fn look_around(&self) -> PromptContext<'_> {
        self.context(&self.history, "I look around.")
    }

    fn context<'a>(&'a self, history: &'a [StoryRow], user_message: &'a str) -> PromptContext<'a> {
        let room = self
            .map
            .get_room_by_id("room_1")
            .expect("the fixture's room exists");
        PromptContext::new(
            &self.world,
            NarrativePerspective::Second,
            NarrativeTense::Present,
            room,
            CharacterContext {
                all_characters: &self.characters,
                characters_present: &self.present,
            },
            &self.persona,
            user_message,
            history,
        )
    }
}

trait PromptText {
    fn system_prompt(&self) -> String;
    /// One flat text, for a check on where a layer stands in the order.
    fn user_prompt(&self) -> String;
}

impl PromptText for AssembledPrompt {
    fn system_prompt(&self) -> String {
        joined_text_of_parts(&self.parts, |role| role == PromptRole::System)
    }

    fn user_prompt(&self) -> String {
        joined_text_of_parts(&self.parts, |role| role != PromptRole::System)
    }
}

fn assemble_under(
    fixture: &Fixture,
    context: &PromptContext,
    preset: &PromptPreset,
    budget: TokenBudget,
) -> Result<AssembledPrompt, EngineError> {
    PromptAssembler.assemble(context, preset, &fixture.global_rules, budget)
}

fn assembled(fixture: &Fixture, context: &PromptContext, preset: &PromptPreset) -> AssembledPrompt {
    assemble_under(fixture, context, preset, TokenBudget::fallback())
        .expect("the prompt fits the default window")
}

fn entry_text(part: &PromptPart) -> String {
    let text = part.text.as_str();
    let text = text.strip_prefix("<ConversationHistory>\n").unwrap_or(text);
    let text = text.strip_prefix("<LastMessage>\n").unwrap_or(text);
    let end = ["\n</ConversationHistory>", "\n</LastMessage>"]
        .iter()
        .filter_map(|close| text.find(close))
        .min()
        .unwrap_or(text.len());
    text[..end].to_string()
}

/// Skips the system part, the part with the blocks before the history, and the
/// closing part that holds only the blocks after the history.
fn history_parts(prompt: &AssembledPrompt) -> Vec<&PromptPart> {
    prompt
        .parts
        .iter()
        .skip(2)
        .filter(|part| part.text.contains("<LastMessage>") || !part.text.contains("<GameState>"))
        .collect()
}

fn history_entries(prompt: &AssembledPrompt) -> Vec<String> {
    history_parts(prompt).into_iter().map(entry_text).collect()
}

fn first_kept_entry(prompt: &AssembledPrompt) -> Option<usize> {
    let first = history_entries(prompt).into_iter().next()?;
    let digits: String = first
        .strip_prefix("Entry ")?
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

fn prompt_tokens(prompt: &AssembledPrompt) -> usize {
    prompt
        .parts
        .iter()
        .map(|part| estimate_tokens(&part.text) + MESSAGE_OVERHEAD_TOKENS)
        .sum()
}

fn window(max_context_tokens: u32, max_tokens: u32) -> TokenBudget {
    TokenBudget {
        max_context_tokens,
        max_history_tokens: DEFAULT_MAX_HISTORY_TOKENS,
        max_tokens,
    }
}

fn history_size(max_history_tokens: u32) -> TokenBudget {
    TokenBudget {
        max_history_tokens,
        ..TokenBudget::fallback()
    }
}

const CUTTING_HISTORY_TOKENS: u32 = 16_000;

fn assembled_under(
    fixture: &Fixture,
    history: &[StoryRow],
    input: &str,
    budget: TokenBudget,
) -> AssembledPrompt {
    assemble_under(fixture, &fixture.context(history, input), &preset(), budget)
        .expect("the prompt fits its window")
}

fn assembled_cutting(fixture: &Fixture, history: &[StoryRow], input: &str) -> AssembledPrompt {
    assembled_under(
        fixture,
        history,
        input,
        history_size(CUTTING_HISTORY_TOKENS),
    )
}

// CASE assembler-prompt-layers: every prompt layer renders.
#[test]
fn test_every_prompt_layer_renders() {
    let fixture = Fixture::new();
    let prompt = assembled(&fixture, &fixture.look_around(), &preset());
    let (system, user) = (prompt.system_prompt(), prompt.user_prompt());

    assert!(
        system.contains("<role>") && system.contains("You are a test narrator."),
        "the preset's role must open the system prompt: {system}"
    );
    assert!(
        system.contains("<instructions>") && system.contains("Follow the world's rules."),
        "the preset's instructions must render: {system}"
    );
    assert!(
        system.contains("<global_rules>"),
        "the global rules belong to the system prompt: {system}"
    );
    for layer in [
        "<GameState>",
        "<KnownCharacters>",
        "<CharactersPresent>",
        "<Persona>",
        "<WorldLore>",
        "<LastMessage>",
        "<PlayerInput>",
        "<writing_style>",
        "<output_format>",
    ] {
        assert!(
            user.contains(layer),
            "the {layer} layer must render: {user}"
        );
    }
    assert!(
        user.contains(&fixture.persona.sheet.name),
        "the Persona renders: {user}"
    );
}

// CASE assembler-layer-order: the changing blocks follow the history, and the style, the format and the Input stay after them.
#[test]
fn test_the_changing_blocks_follow_the_history() {
    let fixture = Fixture::new();

    let user = assembled(&fixture, &fixture.look_around(), &preset()).user_prompt();

    let order = [
        "<KnownCharacters>",
        "<Persona>",
        "<WorldLore>",
        "<LastMessage>",
        "<GameState>",
        "<CharactersPresent>",
        "<writing_style>",
        "<output_format>",
        "<PlayerInput>",
    ];
    let positions: Vec<usize> = order
        .iter()
        .map(|layer| {
            user.find(layer)
                .unwrap_or_else(|| panic!("the {layer} layer must render: {user}"))
        })
        .collect();
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "the layers must follow {order:?}, found at {positions:?}: {user}"
    );
}

// CASE assembler-turn-untrimmed: a turn that fits the window passes through untrimmed.
#[test]
fn test_a_turn_that_fits_passes_through_untrimmed() {
    let fixture = Fixture::new();
    let user = assembled(&fixture, &fixture.look_around(), &preset()).user_prompt();

    assert!(
        user.contains("You enter the inn."),
        "the history renders verbatim when the turn fits: {user}"
    );
    assert!(
        user.contains("I look around."),
        "the player's own text renders: {user}"
    );
}

// CASE assembler-preset-fields-absent: a preset with no fields renders no section tags.
#[test]
fn test_a_preset_with_no_fields_renders_no_sections() {
    let mut fixture = Fixture::new();
    fixture.global_rules = Vec::new();
    let prompt = assembled(&fixture, &fixture.look_around(), &empty_preset());

    assert!(
        prompt.system_prompt().is_empty(),
        "no field means no system prompt: {}",
        prompt.system_prompt()
    );
    assert!(
        prompt.user_prompt().contains("<GameState>"),
        "the data layers stay"
    );
    assert!(prompt.user_prompt().contains("<PlayerInput>"));
    for absent in [
        "<role>",
        "<instructions>",
        "<global_rules>",
        "<writing_style>",
    ] {
        assert!(
            !prompt.user_prompt().contains(absent),
            "{absent} must not render"
        );
    }
}

// CASE assembler-preset-fields-present: only the preset's present fields render, and the output format holds the preset's own words.
#[test]
fn test_only_the_presets_present_fields_render() {
    let fixture = Fixture::new();
    let mut preset = preset();
    preset.instructions = None;
    preset.writing_style = None;
    let prompt = assembled(&fixture, &fixture.look_around(), &preset);

    assert!(
        prompt.system_prompt().contains("<role>"),
        "a present field renders"
    );
    assert!(prompt.system_prompt().contains("<global_rules>"));
    assert!(
        !prompt.system_prompt().contains("<instructions>"),
        "an absent field renders nothing: {}",
        prompt.system_prompt()
    );
    assert!(!prompt.user_prompt().contains("<writing_style>"));
    let format = prompt_block(&prompt.user_prompt(), "<output_format>", "</output_format>");
    assert!(
        format.contains("Write one paragraph."),
        "the present output format renders: {format}"
    );
}

fn layer_text(prompt: &str, tag: &str) -> String {
    let wrapped = prompt_block(prompt, &format!("<{tag}>"), &format!("</{tag}>"));
    let inner = wrapped
        .strip_prefix(&format!("<{tag}>\n"))
        .and_then(|rest| rest.strip_suffix(&format!("\n</{tag}>")))
        .expect("a wrapped layer holds its text between its tags");
    inner
        .lines()
        .map(|line| line.strip_prefix("    ").unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

// CASE assembler-output-format-is-the-preset-text: the output format layer holds exactly the preset's text, for a Narration and for an Impersonate.
#[test]
fn test_the_output_format_layer_is_the_preset_text() {
    let fixture = Fixture::new();
    let marker = "MARKER-ONE\n\nSecond block of the marker.";
    for generation_preset in [
        PromptPreset {
            output_format: Some(marker.to_string()),
            ..preset()
        },
        PromptPreset {
            output_format: Some(marker.to_string()),
            ..impersonate_preset()
        },
    ] {
        let impersonate = generation_preset.preset_type == PresetType::Impersonate;
        let context = fixture.look_around().with_impersonate(impersonate);

        let prompt = assembled(&fixture, &context, &generation_preset);

        assert_eq!(
            layer_text(&prompt.user_prompt(), "output_format"),
            marker,
            "impersonate {impersonate}: the engine adds nothing to the preset's output format"
        );
    }
}

// CASE assembler-history-small-budget: a small context window drops the oldest history entries, whole.
#[test]
fn test_a_small_budget_drops_the_oldest_history() {
    let fixture = Fixture::new();
    let history = fixture.history_of(200);

    let fitted = assembled_under(
        &fixture,
        &history,
        "I look around.",
        window(6144, DEFAULT_MAX_TOKENS),
    );

    let kept = fitted.user_prompt().matches("Entry ").count();
    assert!(
        0 < kept && kept < history.len(),
        "a 6144-token window keeps some of 200 entries: {kept} kept"
    );
    assert!(
        fitted.user_prompt().contains("Entry 199 "),
        "the newest entry must stay: {}",
        fitted.user_prompt()
    );
    assert!(
        !fitted.user_prompt().contains("Entry 0 "),
        "the oldest entry goes first"
    );
}

// CASE assembler-history-large-budget: the default window keeps the whole story window.
#[test]
fn test_the_default_window_carries_the_whole_history() {
    let fixture = Fixture::new();
    let history = fixture.history_of(200);
    let user = assembled(
        &fixture,
        &fixture.context(&history, "I look around."),
        &preset(),
    )
    .user_prompt();

    assert!(
        user.contains("Entry 0 "),
        "the default window keeps the oldest entry: {user}"
    );
    assert!(user.contains("Entry 199 "), "and the newest: {user}");
}

// CASE assembler-history-default-size: with no history size set, a history past the default size keeps its newest entries within it.
#[test]
fn test_a_history_past_the_default_size_keeps_its_newest_entries() {
    let fixture = Fixture::new();
    let history = fixture.history_of(5000);
    let prompt = assembled_under(&fixture, &history, "I look around.", window(1_000_000, 500));

    let kept = history_entries(&prompt);
    let kept_tokens: usize = kept.iter().map(|entry| estimate_tokens(entry)).sum();
    assert!(
        kept_tokens <= DEFAULT_MAX_HISTORY_TOKENS as usize,
        "the history stays within the default size: {kept_tokens} tokens"
    );
    assert!(
        kept.last()
            .is_some_and(|entry| entry.starts_with("Entry 4999 ")),
        "the newest entry must stay"
    );
    assert_eq!(first_kept_entry(&prompt).map(|start| 0 < start), Some(true));
}

// CASE assembler-history-size-field: a Connection history size smaller than the window cuts the history at that size.
#[test]
fn test_a_connection_history_size_cuts_the_history_at_that_size() {
    let fixture = Fixture::new();
    let history = fixture.history_of(2000);

    let sized = assembled_under(
        &fixture,
        &history,
        "I look around.",
        TokenBudget {
            max_context_tokens: 1_000_000,
            max_history_tokens: 4000,
            max_tokens: 500,
        },
    );

    let kept = history_entries(&sized);
    let kept_tokens: usize = kept.iter().map(|entry| estimate_tokens(entry)).sum();
    assert!(
        kept_tokens <= 4000,
        "the history stays within its size: {kept_tokens} tokens"
    );
    assert!(
        2000 < kept_tokens,
        "the cut leaves most of the size and not a few entries: {kept_tokens} tokens"
    );
    assert!(
        kept.last()
            .is_some_and(|entry| entry.starts_with("Entry 1999 ")),
        "the newest entry must stay"
    );
}

// CASE assembler-history-cut-drops-whole-messages: a cut history starts at an entry, and every kept entry is one whole message with no sender label.
#[test]
fn test_a_cut_history_keeps_whole_entries() {
    let fixture = Fixture::new();
    let history = fixture.multiline_history_of(2000);
    let prompt = assembled_cutting(&fixture, &history, "I look around.");

    let kept = history_entries(&prompt);
    assert!(
        !kept.is_empty() && kept.len() < history.len(),
        "the cut drops some: {} kept",
        kept.len()
    );
    for entry in &kept {
        assert!(
            entry.starts_with("Entry ")
                && entry.contains("first line.\n")
                && entry.ends_with("second line."),
            "every kept entry is whole and carries no sender label: {entry}"
        );
    }
    assert!(
        !prompt.user_prompt().contains("Narrator:"),
        "no entry carries a sender label"
    );
}

// CASE assembler-history-roles: each history entry is one message whose role follows its type, with no sender label: a Narration is assistant, an Input is user, and a System entry is system.
#[test]
fn test_each_history_entry_is_one_message_with_its_role() {
    let fixture = Fixture::new();
    let history = vec![
        StoryRow {
            text: "The narrator speaks.".to_string(),
            ..default_story_row(MessageType::Narration)
        },
        StoryRow {
            text: "I answer.".to_string(),
            ..default_story_row(MessageType::Input)
        },
        StoryRow {
            text: "A hidden instruction.".to_string(),
            ..default_story_row(MessageType::System)
        },
        StoryRow {
            text: "The narrator speaks again.".to_string(),
            ..default_story_row(MessageType::Narration)
        },
    ];

    let prompt = assembled(
        &fixture,
        &fixture.context(&history, "I look around."),
        &preset(),
    );

    let seen: Vec<(PromptRole, String)> = history_parts(&prompt)
        .iter()
        .map(|part| (part.role, entry_text(part)))
        .collect();
    assert_eq!(
        seen,
        vec![
            (PromptRole::Assistant, "The narrator speaks.".to_string()),
            (PromptRole::User, "I answer.".to_string()),
            (PromptRole::System, "A hidden instruction.".to_string()),
            (
                PromptRole::Assistant,
                "The narrator speaks again.".to_string()
            ),
        ],
        "one message for each entry, in order: {:?}",
        prompt.parts
    );
}

// CASE assembler-history-wrap: the history tag opens the first entry and closes the second-to-last, and the last entry has its own tag. A history of one entry has the last-message tag only.
#[test]
fn test_the_history_tags_wrap_the_entries() {
    let fixture = Fixture::new();
    let history = fixture.history_of(3);

    let prompt = assembled(
        &fixture,
        &fixture.context(&history, "I look around."),
        &preset(),
    );

    let parts = history_parts(&prompt);
    assert_eq!(parts.len(), 3);
    assert!(
        parts[0].text.starts_with("<ConversationHistory>\nEntry 0 "),
        "the history tag opens the first entry: {}",
        parts[0].text
    );
    assert!(
        parts[1].text.ends_with("\n</ConversationHistory>"),
        "the history tag closes the second-to-last entry: {}",
        parts[1].text
    );
    assert!(
        parts[2].text.starts_with("<LastMessage>\nEntry 2 ")
            && parts[2].text.contains("\n</LastMessage>"),
        "the last entry has its own tag: {}",
        parts[2].text
    );
    let flat = prompt.user_prompt();
    assert_eq!(flat.matches("<ConversationHistory>").count(), 1);
    assert_eq!(flat.matches("</ConversationHistory>").count(), 1);

    let one = assembled(
        &fixture,
        &fixture.context(&history[..1], "I look around."),
        &preset(),
    );
    let flat = one.user_prompt();
    assert!(flat.contains("<LastMessage>\nEntry 0 "), "{flat}");
    assert!(
        !flat.contains("ConversationHistory>\nEntry"),
        "one entry has no history tag: {flat}"
    );
}

// CASE assembler-before-blocks-in-first-user-message: the blocks before the history make the first user message, and the system message holds the preset text only.
#[test]
fn test_the_blocks_before_the_history_make_the_first_user_message() {
    let fixture = Fixture::new();
    let history = fixture.history_of(3);

    let prompt = assembled(
        &fixture,
        &fixture.context(&history, "I look around."),
        &preset(),
    );

    assert_eq!(prompt.parts[0].role, PromptRole::System);
    assert_eq!(prompt.parts[1].role, PromptRole::User);
    for block in ["<KnownCharacters>", "<Persona>", "<WorldLore>"] {
        assert!(
            prompt.parts[1].text.contains(block),
            "the first user message holds {block}: {}",
            prompt.parts[1].text
        );
        assert!(
            !prompt.parts[0].text.contains(block),
            "the system message holds no {block}"
        );
    }
    assert_eq!(
        prompt.parts[2].role,
        PromptRole::Assistant,
        "the first history entry follows, so the roles alternate"
    );
}

// CASE assembler-after-blocks-join-the-last-user-message: the blocks after the history join the last history message when it is a user message, so no two user messages follow each other.
#[test]
fn test_the_blocks_after_the_history_join_a_last_user_message() {
    let fixture = Fixture::new();
    let history = history_ending_with_the_door_input(&fixture);

    let prompt = assembled(&fixture, &fixture.context(&history, DOOR_INPUT), &preset());

    let last = prompt.parts.last().expect("a prompt has parts");
    assert_eq!(last.role, PromptRole::User);
    assert!(
        last.text
            .starts_with("<LastMessage>\nI try the door.\n</LastMessage>\n\n<GameState>"),
        "the blocks follow the last entry in one message: {}",
        last.text
    );
    assert!(
        prompt
            .parts
            .windows(2)
            .all(|pair| !(pair[0].role == PromptRole::User && pair[1].role == PromptRole::User)),
        "no two user messages follow each other: {:?}",
        prompt
            .parts
            .iter()
            .map(|part| part.role)
            .collect::<Vec<_>>()
    );
}

// CASE assembler-after-blocks-join-the-first-user-message: with no history entry, the blocks after the history join the first user message.
#[test]
fn test_the_blocks_after_the_history_join_the_first_user_message_with_no_entry() {
    let fixture = Fixture::new();

    let prompt = assembled(&fixture, &fixture.context(&[], "I look around."), &preset());

    let roles: Vec<PromptRole> = prompt.parts.iter().map(|part| part.role).collect();
    assert_eq!(roles, vec![PromptRole::System, PromptRole::User]);
    assert!(
        prompt.parts[1]
            .text
            .contains("</ConversationHistory>\n\n<GameState>")
            && prompt.parts[1]
                .text
                .contains("<PlayerInput>\nI look around.\n</PlayerInput>"),
        "the blocks after the history follow the placeholder in one message: {}",
        prompt.parts[1].text
    );
}

// CASE assembler-after-blocks-new-user-message: the blocks after the history form a new final user message when the last history message is not a user message.
#[test]
fn test_the_blocks_after_the_history_make_a_new_message_after_a_narration() {
    let fixture = Fixture::new();
    let history = fixture.history_of(2);

    let prompt = assembled(
        &fixture,
        &fixture.context(&history, "").with_impersonate(true),
        &preset(),
    );

    let roles: Vec<PromptRole> = prompt.parts.iter().map(|part| part.role).collect();
    assert_eq!(
        roles,
        vec![
            PromptRole::System,
            PromptRole::User,
            PromptRole::Assistant,
            PromptRole::Assistant,
            PromptRole::User,
        ]
    );
    let last = prompt.parts.last().expect("a prompt has parts");
    assert!(
        last.text.starts_with("<GameState>"),
        "the final message opens with the blocks after the history: {}",
        last.text
    );
}

const DOOR_INPUT: &str = "I try the door.";

fn history_ending_with_the_door_input(fixture: &Fixture) -> Vec<StoryRow> {
    let mut history = fixture.history_of(2);
    history.push(StoryRow {
        text: DOOR_INPUT.to_string(),
        ..default_story_row(MessageType::Input)
    });
    history
}

// CASE assembler-player-input-dropped-when-in-history: the Player Input block is left out when its text is the last history entry, so the Input appears once.
#[test]
fn test_the_player_input_block_is_dropped_when_the_history_ends_with_it() {
    let fixture = Fixture::new();
    let history = history_ending_with_the_door_input(&fixture);

    let answered = assembled(&fixture, &fixture.context(&history, DOOR_INPUT), &preset());

    assert!(
        !answered.user_prompt().contains("<PlayerInput>"),
        "the last entry already carries the Input: {}",
        answered.user_prompt()
    );
    assert_eq!(
        answered.user_prompt().matches(DOOR_INPUT).count(),
        1,
        "the Input appears once"
    );
}

// CASE assembler-player-input-kept-when-not-in-history: an Input that is not the last history entry keeps its Player Input block.
#[test]
fn test_an_input_that_is_not_the_last_entry_keeps_its_block() {
    let fixture = Fixture::new();
    let history = history_ending_with_the_door_input(&fixture);

    let other = assembled(&fixture, &fixture.context(&history, "I knock."), &preset());

    assert!(
        other
            .user_prompt()
            .contains("<PlayerInput>\nI knock.\n</PlayerInput>"),
        "an Input that is not the last entry keeps its block: {}",
        other.user_prompt()
    );
}

// CASE assembler-player-input-kept-for-impersonate-direction: an Impersonate direction is not a history entry, so it keeps its Player Input block.
#[test]
fn test_an_impersonate_direction_keeps_its_block() {
    let fixture = Fixture::new();
    let history = history_ending_with_the_door_input(&fixture);

    let direction = assembled(
        &fixture,
        &fixture
            .context(&history, "Ask about the gate.")
            .with_impersonate(true),
        &preset(),
    );

    assert!(
        direction
            .user_prompt()
            .contains("<PlayerInput>\nAsk about the gate.\n</PlayerInput>"),
        "an Impersonate direction keeps its block: {}",
        direction.user_prompt()
    );
}

// CASE assembler-player-input-kept-for-event-continuation: an event continuation is not a history entry, so it keeps its Player Input block.
#[test]
fn test_an_event_continuation_keeps_its_block() {
    let fixture = Fixture::new();
    let history = history_ending_with_the_door_input(&fixture);
    let continuation =
        "Previous narration:\nI try the door.\n\nTrigger direction: the gate swings open";

    let event = assembled(
        &fixture,
        &fixture.context(&history, continuation),
        &preset(),
    );

    assert!(
        event
            .user_prompt()
            .contains("<PlayerInput>\nPrevious narration:"),
        "an event continuation keeps its block: {}",
        event.user_prompt()
    );
}

// CASE assembler-blank-player-input-renders-nothing: a blank user message renders no Player Input block, so the user message ends at the post-history splice, or at the guide when one is set.
#[test]
fn test_a_blank_user_message_renders_no_player_input_block() {
    let fixture = Fixture::new();
    for impersonate in [false, true] {
        let context = fixture
            .context(&fixture.history, "")
            .with_impersonate(impersonate);
        let user = assembled(&fixture, &context, &preset()).user_prompt();

        assert!(
            !user.contains("<PlayerInput>"),
            "impersonate {impersonate}: a blank user message renders no layer at all: {user}"
        );
        assert!(
            user.trim_end().ends_with("</output_format>"),
            "impersonate {impersonate}: the user message ends with the post-history splice: {user}"
        );
    }

    let guided = fixture
        .context(&fixture.history, "")
        .with_guide(Some("Add tension.".to_string()));
    let user = assembled(&fixture, &guided, &preset()).user_prompt();
    assert!(!user.contains("<PlayerInput>"), "{user}");
    assert!(
        user.trim_end().ends_with("</Guide>"),
        "the guide ends the user message: {user}"
    );
}

// CASE assembler-empty-history-placeholder: an empty history keeps its placeholder text in the first user message.
#[test]
fn test_an_empty_history_keeps_its_placeholder_in_the_first_user_message() {
    let fixture = Fixture::new();

    let prompt = assembled(&fixture, &fixture.context(&[], "I look around."), &preset());

    assert!(
        history_parts(&prompt).is_empty(),
        "no entry becomes a message: {:?}",
        prompt.parts
    );
    assert!(
        prompt.parts[1].text.contains(
            "<ConversationHistory>\n(No history yet - this is the start of the conversation)\n</ConversationHistory>"
        ),
        "the first user message holds the placeholder: {}",
        prompt.parts[1].text
    );
}

// CASE assembler-history-cut-holds-between-blocks: one more Message past the space leaves the history start where it was.
#[test]
fn test_the_history_start_holds_between_cuts() {
    let fixture = Fixture::new();
    let history = fixture.history_of(1101);

    let first = assembled_cutting(&fixture, &history[..1100], "I look around.");
    let second = assembled_cutting(&fixture, &history, "I look around.");

    let start = first_kept_entry(&first).expect("the first prompt keeps entries");
    assert!(0 < start, "the first prompt is past the space");
    assert_eq!(first_kept_entry(&second), Some(start));
}

// CASE assembler-history-cut-ignores-the-player-input: a Player Input under the reserve does not move the history start.
#[test]
fn test_the_player_input_does_not_move_the_history_start() {
    let fixture = Fixture::new();
    let history = fixture.history_of(1100);
    let long_input = "I look around the room slowly. ".repeat(100);

    let short = assembled_cutting(&fixture, &history, "Go.");
    let long = assembled_cutting(&fixture, &history, &long_input);

    assert!(first_kept_entry(&short).is_some_and(|start| 0 < start));
    assert_eq!(first_kept_entry(&short), first_kept_entry(&long));
}

// CASE assembler-history-cut-keeps-max-tokens: a cut history leaves the requested response length unchanged.
#[test]
fn test_a_cut_history_keeps_the_requested_response_length() {
    let fixture = Fixture::new();
    let history = fixture.history_of(2000);

    let prompt = assembled_under(&fixture, &history, "I look around.", window(8192, 500));

    assert!(first_kept_entry(&prompt).is_some_and(|start| 0 < start));
    assert_eq!(prompt.max_tokens, 500);
}

// CASE assembler-history-cut-marker-when-nothing-kept: Messages exist and the space keeps none, so the history holds the marker.
#[test]
fn test_a_history_with_no_kept_message_holds_the_marker() {
    let fixture = Fixture::new();
    let mut history = fixture.history_of(1);
    history[0].text = "A very long entry. ".repeat(5000);

    let prompt = assembled_cutting(&fixture, &history, "I look around.");

    assert!(
        history_parts(&prompt).is_empty(),
        "no entry fits, so none is sent"
    );
    assert!(
        prompt.parts[1].text.contains(
            "<ConversationHistory>\n(History truncated to fit context window)\n</ConversationHistory>"
        ),
        "the first user message holds the marker: {}",
        prompt.parts[1].text
    );
}

// CASE assembler-history-cut-guards-the-limit: a Player Input past the reserve drops more Messages, and the prompt fits the input limit.
#[test]
fn test_a_long_player_input_drops_more_messages_to_fit() {
    let fixture = Fixture::new();
    let history = fixture.history_of(2000);
    let long_input = "I look around the room slowly. ".repeat(600);

    let prompt = assembled_under(&fixture, &history, &long_input, window(8192, 500));

    let input_limit = 8192 - SAFETY_MARGIN_TOKENS as usize - 500;
    let sent = prompt_tokens(&prompt);
    assert!(
        sent <= input_limit,
        "the prompt fits the input limit: {sent} of {input_limit}"
    );
    assert!(
        prompt.user_prompt().contains("Entry 1999 "),
        "the newest entry stays"
    );
}

// CASE assembler-history-budget-counts-message-overhead: a long history of short entries counts the chat-template cost of each message, so the prompt with that cost fits the input limit.
#[test]
fn test_a_long_history_of_short_entries_fits_with_the_cost_of_each_message() {
    let fixture = Fixture::new();
    let history: Vec<StoryRow> = (0..4000)
        .map(|index| StoryRow {
            id: index,
            text: "Yes.".to_string(),
            ..default_story_row(if index % 2 == 0 {
                MessageType::Narration
            } else {
                MessageType::Input
            })
        })
        .collect();

    let prompt = assembled_under(&fixture, &history, "Go on.", window(8192, 500));

    let input_limit = 8192 - SAFETY_MARGIN_TOKENS as usize - 500;
    let sent = prompt_tokens(&prompt);
    assert!(
        sent <= input_limit,
        "the prompt with the cost of each message fits the input limit: {sent} of {input_limit}"
    );
    assert!(history_parts(&prompt).len() < history.len());
}

// CASE assembler-response-cap: the prompt asks for the budget's response length.
#[test]
fn test_the_prompt_asks_for_the_budgets_response_length() {
    let fixture = Fixture::new();
    let requested = TokenBudget {
        max_context_tokens: MAX_CONTEXT_TOKENS,
        max_history_tokens: DEFAULT_MAX_HISTORY_TOKENS,
        max_tokens: 500,
    };

    let prompt = assemble_under(&fixture, &fixture.look_around(), &preset(), requested)
        .expect("the prompt fits");

    assert_eq!(
        prompt.max_tokens, 500,
        "the prompt asks for the budget's response length"
    );
}

// CASE assembler-posture-macros: the assembler stamps the context's perspective and tense onto the preset's macros.
#[test]
fn test_the_assembler_stamps_the_posture() {
    let fixture = Fixture::new();

    let user = assembled(&fixture, &fixture.look_around(), &preset()).user_prompt();

    assert!(
        user.contains("second-person limited perspective"),
        "the context's perspective must win: {user}"
    );
    assert!(
        user.contains("present tense narrative prose"),
        "the context's tense must win: {user}"
    );
    assert!(
        !user.contains("{{narrative_perspective}}") && !user.contains("{{narrative_tense}}"),
        "no posture macro stays unrendered: {user}"
    );
}

// CASE assembler-guide-layer: the guide renders last, in its wrapper.
#[test]
fn test_the_guide_renders_last_in_its_wrapper() {
    let fixture = Fixture::new();
    let context = fixture
        .look_around()
        .with_guide(Some("Add tension.".to_string()));

    let user = assembled(&fixture, &context, &preset()).user_prompt();

    let guide = user.find("<Guide>").expect("the guide layer renders");
    let input = user.find("<PlayerInput>").expect("the input layer renders");
    assert!(guide > input, "the guide renders after the input: {user}");
    assert!(
        user.contains(
            "Take the following into special consideration for your next message: Add tension."
        ),
        "the guide keeps its wrapper: {user}"
    );
}

// CASE assembler-guide-absent: a plain turn, and a blank guide, render no guide layer.
#[test]
fn test_a_plain_turn_and_a_blank_guide_render_no_guide_layer() {
    let fixture = Fixture::new();
    assert!(
        !assembled(&fixture, &fixture.look_around(), &preset())
            .user_prompt()
            .contains("<Guide>"),
        "a plain turn carries no guide layer"
    );

    let blank = fixture.look_around().with_guide(Some("   ".to_string()));
    assert!(
        !assembled(&fixture, &blank, &preset())
            .user_prompt()
            .contains("<Guide>"),
        "a whitespace-only guide renders nothing"
    );
}

// CASE assembler-impersonate-layer: an impersonate prompt drops the player-character layer and renders no guide.
#[test]
fn test_an_impersonate_prompt_drops_the_player_layer_and_the_guide() {
    let fixture = Fixture::new();
    let context = fixture
        .context(&fixture.history, "Ask about the gate.")
        .with_guide(Some("Add tension.".to_string()))
        .with_impersonate(true);

    let user = assembled(&fixture, &context, &preset()).user_prompt();

    assert!(
        !user.contains("<Persona>"),
        "an impersonate turn drops the player layer: {user}"
    );
    assert!(!user.contains("<Guide>"), "and renders no guide: {user}");
}

// CASE assembler-impersonate-persona-macros: the impersonate preset's persona macros render.
#[test]
fn test_the_impersonate_preset_renders_its_persona_macros() {
    let fixture = Fixture::new();
    let context = fixture.context(&fixture.history, "").with_impersonate(true);

    let system = assembled(&fixture, &context, &impersonate_preset()).system_prompt();

    assert!(
        system.contains(&format!("Write as {}.", fixture.persona.sheet.name)),
        "{{user}} renders the Persona's name: {system}"
    );
    assert!(
        system.contains(&fixture.persona.sheet.description),
        "{{persona_description}} renders the description: {system}"
    );
    assert!(
        system.contains(&format!(
            "{} / {}",
            fixture.persona.sheet.personality, fixture.persona.sheet.scenario
        )),
        "{{persona_personality}} and {{persona_background}} render: {system}"
    );
}

// CASE assembler-global-rules: the global rules it is given render into the system prompt.
#[test]
fn test_the_global_rules_render_into_the_system_prompt() {
    let mut fixture = Fixture::new();
    fixture.global_rules = vec!["Magic is forbidden.".to_string()];

    let prompt = assembled(&fixture, &fixture.look_around(), &preset());

    assert!(
        prompt.system_prompt().contains("Magic is forbidden."),
        "the rules belong to the system prompt: {}",
        prompt.system_prompt()
    );
}

// CASE assembler-character-summary: a Character with a summary renders that summary, not the long description.
#[test]
fn test_a_character_with_a_summary_renders_that_summary() {
    let character = character_card("n1", "Alice", Some("Short summary."), "Long desc.", vec![]);
    let fixture = Fixture::new().with_characters(vec![character.clone()], vec![character]);

    let prompt = assembled(&fixture, &fixture.look_around(), &preset());
    let known = prompt_block(
        &prompt.user_prompt(),
        "<KnownCharacters>",
        "</KnownCharacters>",
    );

    assert!(
        known.contains("- Alice\n"),
        "the known entry opens with the name line: {known}"
    );
    assert!(
        known.contains("Short summary."),
        "the summary is the known entry: {known}"
    );
    assert!(
        !known.contains("Long desc."),
        "the long description stays out of the known entry: {known}"
    );
}

// CASE assembler-character-description-lines: a Character without a summary renders the first three description lines.
#[test]
fn test_a_character_without_a_summary_renders_three_description_lines() {
    let description = "line one.\nline two.\nline three.\nline four.";
    let character = character_card("n2", "Bob", None, description, vec![]);
    // Known to the World, not among the Characters present.
    let fixture = Fixture::new().with_characters(vec![character], vec![]);

    let prompt = assembled(&fixture, &fixture.look_around(), &preset());
    let known = prompt_block(
        &prompt.user_prompt(),
        "<KnownCharacters>",
        "</KnownCharacters>",
    );

    assert!(
        known.contains("- Bob\n"),
        "the known entry opens with the name line: {known}"
    );
    for line in ["line one.", "line two.", "line three."] {
        assert!(known.contains(line), "{line} must render: {known}");
    }
    assert!(
        !known.contains("line four."),
        "only the first three lines render: {known}"
    );
}

// CASE assembler-known-entry-has-no-presence-mark: a present and an absent Character both render as a bare name line.
#[test]
fn test_a_known_entry_renders_the_bare_name_whether_present_or_absent() {
    let alice = character_card("n1", "Alice", Some("Here."), "Long desc.", vec![]);
    let bob = character_card("n2", "Bob", Some("Away."), "Long desc.", vec![]);
    let fixture = Fixture::new().with_characters(vec![alice.clone(), bob], vec![alice]);

    let prompt = assembled(&fixture, &fixture.look_around(), &preset());
    let known = prompt_block(
        &prompt.user_prompt(),
        "<KnownCharacters>",
        "</KnownCharacters>",
    );

    assert!(
        known.contains("- Alice\n") && known.contains("- Bob\n"),
        "each Character renders as a bare name line: {known}"
    );
    assert!(
        !known.contains("(in room)") && !known.contains("(elsewhere)"),
        "the known entry carries no presence mark: {known}"
    );
}

fn allied_with_partner(id: &str) -> CharacterCard {
    character_card(
        id,
        "Self",
        None,
        "d",
        vec![Relationship {
            with: "p1".to_string(),
            static_text: "allies".to_string(),
            dynamic: String::new(),
        }],
    )
}

// CASE assembler-character-relationships-absent: no relationships block renders while the related Character is away.
#[test]
fn test_no_relationships_block_renders_while_the_related_character_is_away() {
    let partner = character_card("p1", "Partner", None, "d", vec![]);
    let self_card = allied_with_partner("n3");
    let fixture = Fixture::new().with_characters(vec![partner, self_card.clone()], vec![self_card]);

    let prompt = assembled(&fixture, &fixture.look_around(), &preset());

    assert!(
        !prompt.user_prompt().contains("Relationships:"),
        "an away Character has no relationships block: {}",
        prompt.user_prompt()
    );
}

// CASE assembler-character-relationships-present: the relationships block renders the present Character's name.
#[test]
fn test_the_relationships_block_renders_the_present_characters_name() {
    let partner = character_card("p1", "Partner", None, "d", vec![]);
    let self_card = allied_with_partner("n4");
    let fixture = Fixture::new().with_characters(
        vec![partner.clone(), self_card.clone()],
        vec![partner, self_card],
    );

    let user = assembled(&fixture, &fixture.look_around(), &preset()).user_prompt();

    assert!(
        user.contains("Relationships:"),
        "a present Character has a relationships block: {user}"
    );
    assert!(
        user.contains("Partner") && user.contains("allies"),
        "the block names the related Character and its text: {user}"
    );
}

// CASE assembler-cards-in-id-order: both card lists render in Character id order, whatever order the context holds them in.
#[test]
fn test_both_card_lists_render_in_character_id_order() {
    let ada = character_card("a", "Ada", None, "first", vec![]);
    let bea = character_card("b", "Bea", None, "second", vec![]);
    let cid = character_card("c", "Cid", None, "third", vec![]);
    let out_of_order = vec![cid, ada, bea];
    let fixture = Fixture::new().with_characters(out_of_order.clone(), out_of_order);

    let prompt = assembled(&fixture, &fixture.look_around(), &preset());

    for (open, close) in [
        ("<KnownCharacters>", "</KnownCharacters>"),
        ("<CharactersPresent>", "</CharactersPresent>"),
    ] {
        let rendered = prompt_block(&prompt.user_prompt(), open, close);
        let positions: Vec<usize> = ["Ada", "Bea", "Cid"]
            .iter()
            .map(|name| {
                rendered
                    .find(name)
                    .unwrap_or_else(|| panic!("{name} renders in {open}: {rendered}"))
            })
            .collect();
        assert!(
            positions.windows(2).all(|pair| pair[0] < pair[1]),
            "{open} renders its cards in id order: {rendered}"
        );
    }
}

// CASE assembler-card-scenario-label: a present card labels its scenario field `Scenario:`.
#[test]
fn test_a_present_card_labels_its_scenario_field() {
    let card = character_card("a", "Ada", None, "first", vec![]);
    let fixture = Fixture::new().with_characters(vec![card.clone()], vec![card]);

    let prompt = assembled(&fixture, &fixture.look_around(), &preset());
    let present = prompt_block(
        &prompt.user_prompt(),
        "<CharactersPresent>",
        "</CharactersPresent>",
    );

    assert!(
        present.contains("Scenario: At the inn."),
        "the card's scenario field renders under its own label: {present}"
    );
    assert!(
        !present.contains("Context:"),
        "no other label names the scenario field: {present}"
    );
}

// CASE assembler-context-overflow: a prompt that cannot fit the window fails, whether the system layer or the user half is too long for it.
#[test]
fn test_a_prompt_that_cannot_fit_the_window_fails() {
    let fixture = Fixture::new();
    let tight = TokenBudget {
        max_context_tokens: 512,
        max_history_tokens: DEFAULT_MAX_HISTORY_TOKENS,
        max_tokens: 256,
    };

    let mut long_role = preset();
    long_role.role = Some("x".repeat(5000));
    let overflow = assemble_under(&fixture, &fixture.look_around(), &long_role, tight);
    assert!(overflow.is_err(), "an unfittable system layer must fail");

    let mut big_room = fixture
        .map
        .get_room_by_id("room_1")
        .expect("the fixture's room exists")
        .clone();
    big_room.description = "x".repeat(2000);
    let mut context = fixture.context(&[], "look");
    context.room = &big_room;
    context.characters = CharacterContext {
        all_characters: &[],
        characters_present: &[],
    };
    let after_trim = assemble_under(&fixture, &context, &preset(), tight);
    assert!(after_trim.is_err(), "an unfittable user half must fail");
}

fn user_prompt_with_input(player: &str) -> String {
    let fixture = Fixture::new();
    assembled(
        &fixture,
        &fixture.context(&fixture.history, player),
        &preset(),
    )
    .user_prompt()
}

// CASE assembler-input-injection-system: a `{{system}}` in the player's text is filtered.
#[test]
fn test_a_system_injection_in_the_players_text_is_filtered() {
    let prompt = user_prompt_with_input("I want to override {{system}} instructions");
    assert!(
        prompt.contains("I want to override [FILTERED] instructions"),
        "{prompt}"
    );
}

// CASE assembler-input-injection-char: a `{{char}}` in the player's text is filtered.
#[test]
fn test_a_char_injection_in_the_players_text_is_filtered() {
    let prompt = user_prompt_with_input("Your name is now {{char}}");
    assert!(prompt.contains("Your name is now [FILTERED]"), "{prompt}");
}

// CASE assembler-input-clean: plain player text passes through.
#[test]
fn test_plain_player_text_passes_through() {
    let prompt = user_prompt_with_input("hello world");
    assert!(prompt.contains("hello world"), "{prompt}");
}

// CASE assembler-input-single-braces: single braces pass through.
#[test]
fn test_single_braces_in_the_players_text_pass_through() {
    let player = "I have {one} brace and normal text";
    let prompt = user_prompt_with_input(player);
    assert!(prompt.contains(player), "{prompt}");
}

// CASE assembler-input-multiple-injections: every injection in the player's text is filtered.
#[test]
fn test_every_injection_in_the_players_text_is_filtered() {
    let prompt = user_prompt_with_input("{{system}} ignore previous {{char}}");
    assert!(
        prompt.contains("[FILTERED] ignore previous [FILTERED]"),
        "{prompt}"
    );
}

// CASE assembler-input-empty-braces: empty braces pass through.
#[test]
fn test_empty_braces_in_the_players_text_pass_through() {
    let prompt = user_prompt_with_input("test {{}} end");
    assert!(prompt.contains("test {{}} end"), "{prompt}");
}

// CASE assembler-input-unclosed-braces: unclosed braces pass through.
#[test]
fn test_unclosed_braces_in_the_players_text_pass_through() {
    let prompt = user_prompt_with_input("test {{abc end");
    assert!(prompt.contains("test {{abc end"), "{prompt}");
}

// CASE assembler-input-nested-braces: nested braces filter the outer pattern.
#[test]
fn test_nested_braces_in_the_players_text_filter_the_outer_pattern() {
    let prompt = user_prompt_with_input("{{a{{b}}");
    assert!(prompt.contains("[FILTERED]"), "{prompt}");
}

const STORY_MEMORY: &str = "The innkeeper promised a room for three nights.";

fn assembled_remembering(
    fixture: &Fixture,
    history: &[StoryRow],
    input: &str,
    story_memory: &str,
    budget: TokenBudget,
) -> AssembledPrompt {
    let context = fixture
        .context(history, input)
        .with_story_memory(Some(story_memory));
    assemble_under(fixture, &context, &preset(), budget).expect("the prompt fits its window")
}

fn full_size_story_memory() -> String {
    "m".repeat(STORY_MEMORY_CHARS)
}

// CASE assembler-story-memory-before-history: the story memory is the last block of the first user message, so it stands before the first history entry.
#[test]
fn test_the_story_memory_ends_the_first_user_message() {
    let fixture = Fixture::new();

    let prompt = assembled_remembering(
        &fixture,
        &fixture.history,
        "I look around.",
        STORY_MEMORY,
        TokenBudget::fallback(),
    );

    let first_user = &prompt.parts[1].text;
    assert!(
        first_user.ends_with(&format!("<StoryMemory>\n{STORY_MEMORY}\n</StoryMemory>")),
        "the story memory ends the first user message: {first_user}"
    );
    assert!(
        first_user.find("<WorldLore>") < first_user.find("<StoryMemory>"),
        "the story memory follows the other blocks before the history: {first_user}"
    );
    assert_eq!(
        history_entries(&prompt).first().map(String::as_str),
        Some("You enter the inn."),
        "the history follows the story memory"
    );
}

// CASE assembler-story-memory-absent: a context with no story memory renders no story memory block.
#[test]
fn test_no_story_memory_renders_no_block() {
    let fixture = Fixture::new();
    let user = assembled(&fixture, &fixture.look_around(), &preset()).user_prompt();

    assert!(
        !user.contains("<StoryMemory>"),
        "no story memory, no block: {user}"
    );
}

// CASE assembler-plan-cut-matches-the-assembly: the planned cut keeps the entries the assembled prompt holds, with a story memory and without, and when a long Player Input drops more.
#[test]
fn test_the_planned_cut_keeps_the_entries_the_prompt_holds() {
    let fixture = Fixture::new();
    let history = fixture.history_of(2000);
    let long_input = "I look around the room slowly. ".repeat(600);
    let memory = full_size_story_memory();
    let rows: [(&str, Option<&str>, TokenBudget); 3] = [
        ("I look around.", None, history_size(CUTTING_HISTORY_TOKENS)),
        (
            "I look around.",
            Some(&memory),
            history_size(CUTTING_HISTORY_TOKENS),
        ),
        (&long_input, Some(STORY_MEMORY), window(8192, 500)),
    ];

    for (input, story_memory, budget) in rows {
        let context = fixture
            .context(&history, input)
            .with_story_memory(story_memory);

        let planned = PromptAssembler.plan_cut(&context, &preset(), &fixture.global_rules, budget);
        let prompt =
            assemble_under(&fixture, &context, &preset(), budget).expect("the prompt fits");

        assert!(planned.kept_from() > 0, "the history is cut");
        assert_eq!(
            first_kept_entry(&prompt),
            Some(planned.kept_from()),
            "the prompt holds the history from the planned start"
        );
    }
}

// CASE assembler-story-memory-size-does-not-move-the-cut: a short story memory and one at its full size leave the history start in one place.
#[test]
fn test_the_size_of_the_story_memory_does_not_move_the_history_start() {
    let fixture = Fixture::new();
    let history = fixture.history_of(2000);
    let full = full_size_story_memory();
    let input = "I look around.";

    let short = assembled_remembering(&fixture, &history, input, STORY_MEMORY, window(8192, 500));
    let long = assembled_remembering(&fixture, &history, input, &full, window(8192, 500));

    assert!(first_kept_entry(&short).is_some_and(|start| 0 < start));
    assert_eq!(first_kept_entry(&short), first_kept_entry(&long));
}

// CASE assembler-story-memory-fits-the-limit: a story memory at its full size and a long Player Input still fit the input limit.
#[test]
fn test_a_full_size_story_memory_fits_the_input_limit() {
    let fixture = Fixture::new();
    let history = fixture.history_of(2000);
    let long_input = "I look around the room slowly. ".repeat(600);
    let full = full_size_story_memory();

    let prompt = assembled_remembering(&fixture, &history, &long_input, &full, window(8192, 500));

    let input_limit = 8192 - SAFETY_MARGIN_TOKENS as usize - 500;
    let sent = prompt_tokens(&prompt);
    assert!(
        sent <= input_limit,
        "the prompt fits the input limit: {sent} of {input_limit}"
    );
    assert!(
        prompt.user_prompt().contains("Entry 1999 "),
        "the newest entry stays"
    );
}

// CASE assembler-history-cut-reserves-the-story-memory: a history inside its space and with no story memory keeps every entry. Once the context has a story memory or the history passes its space, the window keeps the room of the story memory.
#[test]
fn test_the_history_space_keeps_room_for_the_story_memory_once_it_is_needed() {
    let fixture = Fixture::new();
    let history = fixture.history_of(280);
    let budget = window(8192, 500);
    let input = "I look around.";

    let without = assembled_under(&fixture, &history, input, budget);
    let with = assembled_remembering(&fixture, &history, input, STORY_MEMORY, budget);
    let longer = fixture.history_of(2000);
    let cut_without = assembled_under(&fixture, &longer, input, budget);
    let cut_with = assembled_remembering(&fixture, &longer, input, STORY_MEMORY, budget);

    assert_eq!(
        first_kept_entry(&without),
        Some(0),
        "a history inside its space keeps every entry"
    );
    assert!(
        first_kept_entry(&with).is_some_and(|start| 0 < start),
        "with a story memory, the same history makes room for its block"
    );
    assert_eq!(
        first_kept_entry(&cut_without),
        first_kept_entry(&cut_with),
        "a history past its space keeps the room whether the story memory exists yet or not"
    );
}
