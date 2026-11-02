//! Seam contract of the Narrator, the box that turns a generation into one prompt and one call.

use std::sync::Arc;

use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::narration::{NarrationValues, Narrated, Narrator};
use crate::application::prompting::token_budget::{
    estimate_tokens, TokenBudget, MAX_CONTEXT_TOKENS, MAX_HISTORY_TOKENS, MESSAGE_OVERHEAD_TOKENS,
    SAFETY_MARGIN_TOKENS,
};
use crate::application::prompting::{CharacterContext, PromptAssembler};
use crate::domain::model::agent::Role;
use crate::domain::model::character::{CharacterSheet, CharacterCard, PersonaCard, Relationship};
use crate::domain::model::game::GameId;
use crate::domain::model::generation::Generation;
use crate::domain::model::generation_parameters::{CallSettings, GenerationParameters};
use crate::domain::model::map::MapDef;
use crate::domain::model::message::{Maker, Message, Swipe};
use crate::domain::model::prompt_part::{PromptPart, PromptRole};
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::{NarrativePerspective, NarrativeTense, DEFAULT_MAX_TOKENS};
use crate::domain::model::state::message_types::{MessageKind, MessageType, StoryRow};
use crate::domain::model::state::swipe_state::SwipeState;
use crate::domain::model::state::event_record::{EventRecord, TriggerRef};
use crate::domain::model::tail::Tail;
use crate::domain::model::world::WorldCard;
use crate::test_support::fixtures::{
    default_story_row, prompt_block, TestMap, TestCharacter, TestPersona, TestWorld,
};
use crate::test_support::{
    make_test_recorder, make_test_recorder_with_storage, sent_system_text, sent_user_text,
    ModelCall, ScriptedModel,
};
use crate::domain::model::state::generation_status::GenerationFailure;

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
    /// The story ends with the player's typed Input, which the Retry of a typed Input repeats.
    tail: Tail,
}

impl Fixture {
    fn new() -> Self {
        let characters = vec![TestCharacter::named("character_1", "Test Character")];
        let mut world = TestWorld::minimal();
        world.global_rules = vec!["No fast travel.".to_string()];
        Self {
            world,
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
            tail: Tail {
                last: Message::new(
                    MessageType::Input,
                    MessageKind::TypedInput,
                    Swipe::new(
                        TYPED_INPUT,
                        Maker::Player,
                        SwipeState::starting(None, &Default::default(), "room_1"),
                    ),
                ),
                state_before_last: None,
                count: 1,
            },
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

    fn without_global_rules(mut self) -> Self {
        self.world.global_rules = Vec::new();
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

    fn narration<'a>(
        &'a self,
        history: &'a [StoryRow],
        user_message: &'a str,
    ) -> NarrationCase<'a> {
        self.case_for(
            Generation::Narration { guide: None },
            history,
            Some(user_message),
        )
    }

    fn case_for<'a>(
        &'a self,
        generation: Generation,
        history: &'a [StoryRow],
        input: Option<&'a str>,
    ) -> NarrationCase<'a> {
        let room = self
            .map
            .get_room_by_id("room_1")
            .expect("the fixture's room exists");
        NarrationCase {
            generation,
            values: NarrationValues {
                world: &self.world,
                narrative_perspective: NarrativePerspective::Second,
                narrative_tense: NarrativeTense::Present,
                room,
                characters: CharacterContext {
                    all_characters: &self.characters,
                    characters_present: &self.present,
                },
                persona: &self.persona,
                history,
                input,
                tail: &self.tail,
            },
        }
    }
}

const TYPED_INPUT: &str = "I try the door.";

struct NarrationCase<'a> {
    generation: Generation,
    values: NarrationValues<'a>,
}

impl NarrationCase<'_> {
    fn with_guide(mut self, guide: Option<String>) -> Self {
        self.generation = Generation::Narration { guide };
        self
    }

    fn impersonating(mut self, direction: Option<String>) -> Self {
        self.generation = Generation::Impersonate { direction };
        self
    }
}

fn narrator(model: Arc<ScriptedModel>) -> Narrator {
    let recorder = make_test_recorder(model);
    Narrator::new(PromptAssembler, Arc::clone(&recorder), recorder)
}

fn narrate_with(
    model: &Arc<ScriptedModel>,
    case: &NarrationCase,
    preset: &PromptPreset,
    budget: TokenBudget,
) -> Result<Narrated, GenerationFailure> {
    narrate_on(&narrator(Arc::clone(model)), case, preset, budget)
}

fn narrate_on(
    narrator: &Narrator,
    case: &NarrationCase,
    preset: &PromptPreset,
    budget: TokenBudget,
) -> Result<Narrated, GenerationFailure> {
    let call = CallSettings::of(&preset.parameters, budget.max_tokens, GameId(7));
    narrator.narrate(
        &case.generation,
        &case.values,
        preset,
        &call,
        budget.max_context_tokens,
    )
}

fn sent_prompt(case: &NarrationCase, preset: &PromptPreset) -> ModelCall {
    let model = Arc::new(match case.generation {
        Generation::Event(_) => ScriptedModel::new().event_narration("You look around."),
        _ => ScriptedModel::new().narration("You look around."),
    });
    narrate_with(&model, case, preset, TokenBudget::fallback()).expect("a narration is a result");
    prompt_of(&model)
}

fn prompt_of(model: &ScriptedModel) -> ModelCall {
    model
        .calls()
        .into_iter()
        .next()
        .expect("the narrator makes one call")
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
fn history_parts(call: &ModelCall) -> Vec<&PromptPart> {
    call.parts
        .iter()
        .skip(2)
        .filter(|part| part.text.contains("<LastMessage>") || !part.text.contains("<GameState>"))
        .collect()
}

fn history_entries(call: &ModelCall) -> Vec<String> {
    history_parts(call).into_iter().map(entry_text).collect()
}

fn first_kept_entry(call: &ModelCall) -> Option<usize> {
    let first = history_entries(call).into_iter().next()?;
    let digits: String = first
        .strip_prefix("Entry ")?
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

fn sent_tokens(call: &ModelCall) -> usize {
    call.parts
        .iter()
        .map(|part| estimate_tokens(&part.text) + MESSAGE_OVERHEAD_TOKENS)
        .sum()
}

fn window(max_context_tokens: u32, max_tokens: u32) -> TokenBudget {
    TokenBudget {
        max_context_tokens,
        max_tokens,
    }
}

fn sent_under(
    fixture: &Fixture,
    history: &[StoryRow],
    input: &str,
    budget: TokenBudget,
) -> ModelCall {
    let case = fixture.narration(history, input);
    let model = Arc::new(ScriptedModel::new().narration("Narration."));
    narrate_with(&model, &case, &preset(), budget).expect("a narration is a result");
    prompt_of(&model)
}

// CASE narrator-prompt-layers: every prompt layer renders.
#[test]
fn test_every_prompt_layer_renders() {
    let fixture = Fixture::new();
    let call = sent_prompt(
        &fixture.narration(&fixture.history, "I look around."),
        &preset(),
    );

    assert!(
        call.system_prompt().contains("<role>")
            && call.system_prompt().contains("You are a test narrator."),
        "the preset's role must open the system prompt: {}",
        call.system_prompt()
    );
    assert!(
        call.system_prompt().contains("<instructions>")
            && call.system_prompt().contains("Follow the world's rules."),
        "the preset's instructions must render: {}",
        call.system_prompt()
    );
    assert!(
        call.system_prompt().contains("<global_rules>"),
        "the World's rules belong to the system prompt: {}",
        call.system_prompt()
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
            call.user_prompt().contains(layer),
            "the {layer} layer must render: {}",
            call.user_prompt()
        );
    }
    assert!(
        call.user_prompt().contains(&fixture.persona.sheet.name),
        "the Persona renders: {}",
        call.user_prompt()
    );
}

// CASE narrator-layer-order: the changing blocks follow the history, and the style, the format and the Input stay after them.
#[test]
fn test_the_changing_blocks_follow_the_history() {
    let fixture = Fixture::new();

    let user = sent_prompt(
        &fixture.narration(&fixture.history, "I look around."),
        &preset(),
    )
    .user_prompt();

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

// CASE narrator-turn-untrimmed: a turn that fits the window passes through untrimmed.
#[test]
fn test_a_turn_that_fits_passes_through_untrimmed() {
    let fixture = Fixture::new();
    let call = sent_prompt(
        &fixture.narration(&fixture.history, "I look around."),
        &preset(),
    );

    assert!(
        call.user_prompt().contains("You enter the inn."),
        "the history renders verbatim when the turn fits: {}",
        call.user_prompt()
    );
    assert!(
        call.user_prompt().contains("I look around."),
        "the player's own text renders: {}",
        call.user_prompt()
    );
}

// CASE narrator-preset-fields-absent: a preset with no fields renders no section tags.
#[test]
fn test_a_preset_with_no_fields_renders_no_sections() {
    let fixture = Fixture::new().without_global_rules();
    let preset = empty_preset();
    let call = sent_prompt(
        &fixture.narration(&fixture.history, "I look around."),
        &preset,
    );

    assert!(
        call.system_prompt().is_empty(),
        "no field means no system prompt: {}",
        call.system_prompt()
    );
    assert!(
        call.user_prompt().contains("<GameState>"),
        "the data layers stay"
    );
    assert!(call.user_prompt().contains("<PlayerInput>"));
    for absent in [
        "<role>",
        "<instructions>",
        "<global_rules>",
        "<writing_style>",
    ] {
        assert!(
            !call.user_prompt().contains(absent),
            "{absent} must not render"
        );
    }
}

// CASE narrator-preset-fields-present: only the preset's present fields render, and the output format holds the preset's own words.
#[test]
fn test_only_the_presets_present_fields_render() {
    let fixture = Fixture::new();
    let mut preset = preset();
    preset.instructions = None;
    preset.writing_style = None;
    let call = sent_prompt(
        &fixture.narration(&fixture.history, "I look around."),
        &preset,
    );

    assert!(
        call.system_prompt().contains("<role>"),
        "a present field renders"
    );
    assert!(call.system_prompt().contains("<global_rules>"));
    assert!(
        !call.system_prompt().contains("<instructions>"),
        "an absent field renders nothing: {}",
        call.system_prompt()
    );
    assert!(!call.user_prompt().contains("<writing_style>"));
    let format = prompt_block(&call.user_prompt(), "<output_format>", "</output_format>");
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

// CASE narrator-output-format-is-the-preset-text: the output format layer holds exactly the preset's text, for a Narration and for an Impersonate.
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
        let case = fixture.narration(&fixture.history, "I look around.");
        let case = match generation_preset.preset_type {
            PresetType::Impersonate => case.impersonating(None),
            _ => case,
        };

        let call = sent_prompt(&case, &generation_preset);

        assert_eq!(
            layer_text(&call.user_prompt(), "output_format"),
            marker,
            "{:?}: the engine adds nothing to the preset's output format",
            case.generation.kind()
        );
    }
}

// CASE narrator-history-small-budget: a small context window drops the oldest history entries, whole.
#[test]
fn test_a_small_budget_drops_the_oldest_history() {
    let fixture = Fixture::new();
    let history = fixture.history_of(200);
    let case = fixture.narration(&history, "I look around.");
    let model = Arc::new(ScriptedModel::new().narration("Narration."));

    let narrated = narrate_with(&model, &case, &preset(), window(6144, DEFAULT_MAX_TOKENS))
        .expect("a narration is a result");
    let fitted = prompt_of(&model);

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
    assert!(!narrated.text.is_empty(), "a trimmed turn still narrates");
}

// CASE narrator-history-large-budget: the default window keeps the whole story window.
#[test]
fn test_the_default_window_carries_the_whole_history() {
    let fixture = Fixture::new();
    let history = fixture.history_of(200);
    let call = sent_prompt(&fixture.narration(&history, "I look around."), &preset());

    assert!(
        call.user_prompt().contains("Entry 0 "),
        "the default window keeps the oldest entry: {}",
        call.user_prompt()
    );
    assert!(
        call.user_prompt().contains("Entry 199 "),
        "and the newest: {}",
        call.user_prompt()
    );
}

// CASE narrator-history-cap: a history past the history cap keeps its newest entries within the cap.
#[test]
fn test_a_history_past_the_cap_keeps_its_newest_entries() {
    let fixture = Fixture::new();
    let history = fixture.history_of(2000);
    let call = sent_prompt(&fixture.narration(&history, "I look around."), &preset());

    let kept = history_entries(&call);
    let kept_tokens: usize = kept.iter().map(|entry| estimate_tokens(entry)).sum();
    assert!(
        kept_tokens <= MAX_HISTORY_TOKENS as usize,
        "the history stays within the cap: {kept_tokens} tokens"
    );
    assert!(
        kept.last()
            .is_some_and(|entry| entry.starts_with("Entry 1999 ")),
        "the newest entry must stay"
    );
    assert_eq!(first_kept_entry(&call).map(|start| 0 < start), Some(true));
}

// CASE narrator-history-cut-drops-whole-messages: a cut history starts at an entry, and every kept entry is one whole message with no sender label.
#[test]
fn test_a_cut_history_keeps_whole_entries() {
    let fixture = Fixture::new();
    let history = fixture.multiline_history_of(2000);
    let call = sent_under(
        &fixture,
        &history,
        "I look around.",
        TokenBudget::fallback(),
    );

    let kept = history_entries(&call);
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
        !call.user_prompt().contains("Narrator:"),
        "no entry carries a sender label"
    );
}

// CASE narrator-history-roles: each history entry is one message whose role follows its type, with no sender label: a Narration is assistant, an Input is user, and a System entry is system.
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

    let call = sent_prompt(&fixture.narration(&history, "I look around."), &preset());

    let parts = history_parts(&call);
    let seen: Vec<(PromptRole, String)> = parts
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
        call.parts
    );
}

// CASE narrator-history-wrap: the history tag opens the first entry and closes the second-to-last, and the last entry has its own tag. A history of one entry has the last-message tag only.
#[test]
fn test_the_history_tags_wrap_the_entries_as_marinara_engine_does() {
    let fixture = Fixture::new();
    let history = fixture.history_of(3);

    let call = sent_prompt(&fixture.narration(&history, "I look around."), &preset());

    let parts = history_parts(&call);
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
    let flat = call.user_prompt();
    assert_eq!(flat.matches("<ConversationHistory>").count(), 1);
    assert_eq!(flat.matches("</ConversationHistory>").count(), 1);

    let one = sent_prompt(
        &fixture.narration(&history[..1], "I look around."),
        &preset(),
    );
    let flat = one.user_prompt();
    assert!(flat.contains("<LastMessage>\nEntry 0 "), "{flat}");
    assert!(
        !flat.contains("ConversationHistory>\nEntry"),
        "one entry has no history tag: {flat}"
    );
}

// CASE narrator-before-blocks-in-first-user-message: the blocks before the history make the first user message, and the system message holds the preset text only.
#[test]
fn test_the_blocks_before_the_history_make_the_first_user_message() {
    let fixture = Fixture::new();
    let history = fixture.history_of(3);

    let call = sent_prompt(&fixture.narration(&history, "I look around."), &preset());

    assert_eq!(call.parts[0].role, PromptRole::System);
    assert_eq!(call.parts[1].role, PromptRole::User);
    for block in ["<KnownCharacters>", "<Persona>", "<WorldLore>"] {
        assert!(
            call.parts[1].text.contains(block),
            "the first user message holds {block}: {}",
            call.parts[1].text
        );
        assert!(
            !call.parts[0].text.contains(block),
            "the system message holds no {block}"
        );
    }
    assert_eq!(
        call.parts[2].role,
        PromptRole::Assistant,
        "the first history entry follows, so the roles alternate"
    );
}

// CASE narrator-after-blocks-join-the-last-user-message: the blocks after the history join the last history message when it is a user message, so no two user messages follow each other.
#[test]
fn test_the_blocks_after_the_history_join_a_last_user_message() {
    let fixture = Fixture::new();
    let mut history = fixture.history_of(2);
    history.push(StoryRow {
        text: "I try the door.".to_string(),
        ..default_story_row(MessageType::Input)
    });

    let call = sent_prompt(&fixture.narration(&history, "I try the door."), &preset());

    let last = call.parts.last().expect("a call has parts");
    assert_eq!(last.role, PromptRole::User);
    assert!(
        last.text
            .starts_with("<LastMessage>\nI try the door.\n</LastMessage>\n\n<GameState>"),
        "the blocks follow the last entry in one message: {}",
        last.text
    );
    assert!(
        call.parts
            .windows(2)
            .all(|pair| !(pair[0].role == PromptRole::User && pair[1].role == PromptRole::User)),
        "no two user messages follow each other: {:?}",
        call.parts.iter().map(|part| part.role).collect::<Vec<_>>()
    );
}

// CASE narrator-after-blocks-join-the-first-user-message: with no history entry, the blocks after the history join the first user message.
#[test]
fn test_the_blocks_after_the_history_join_the_first_user_message_with_no_entry() {
    let fixture = Fixture::new();

    let call = sent_prompt(&fixture.narration(&[], "I look around."), &preset());

    let roles: Vec<PromptRole> = call.parts.iter().map(|part| part.role).collect();
    assert_eq!(roles, vec![PromptRole::System, PromptRole::User]);
    assert!(
        call.parts[1]
            .text
            .contains("</ConversationHistory>\n\n<GameState>")
            && call.parts[1]
                .text
                .contains("<PlayerInput>\nI look around.\n</PlayerInput>"),
        "the blocks after the history follow the placeholder in one message: {}",
        call.parts[1].text
    );
}

// CASE narrator-after-blocks-new-user-message: the blocks after the history form a new final user message when the last history message is not a user message.
#[test]
fn test_the_blocks_after_the_history_make_a_new_message_after_a_narration() {
    let fixture = Fixture::new();
    let history = fixture.history_of(2);

    let call = sent_prompt(
        &fixture.narration(&history, "").impersonating(None),
        &preset(),
    );

    let roles: Vec<PromptRole> = call.parts.iter().map(|part| part.role).collect();
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
    let last = call.parts.last().expect("a call has parts");
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

// CASE narrator-player-input-dropped-when-in-history: the Player Input block is left out when its text is the last history entry, so the Input appears once.
#[test]
fn test_the_player_input_block_is_dropped_when_the_history_ends_with_it() {
    let fixture = Fixture::new();
    let history = history_ending_with_the_door_input(&fixture);

    let answered = sent_prompt(&fixture.narration(&history, DOOR_INPUT), &preset());

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

// CASE narrator-player-input-kept-when-not-in-history: an Input that is not the last history entry keeps its Player Input block.
#[test]
fn test_an_input_that_is_not_the_last_entry_keeps_its_block() {
    let fixture = Fixture::new();
    let history = history_ending_with_the_door_input(&fixture);

    let other = sent_prompt(&fixture.narration(&history, "I knock."), &preset());

    assert!(
        other
            .user_prompt()
            .contains("<PlayerInput>\nI knock.\n</PlayerInput>"),
        "an Input that is not the last entry keeps its block: {}",
        other.user_prompt()
    );
}

// CASE narrator-player-input-kept-for-impersonate-direction: an Impersonate direction is not a history entry, so it keeps its Player Input block.
#[test]
fn test_an_impersonate_direction_keeps_its_block() {
    let fixture = Fixture::new();
    let history = history_ending_with_the_door_input(&fixture);

    let direction = sent_prompt(
        &fixture
            .narration(&history, "")
            .impersonating(Some("Ask about the gate.".to_string())),
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

// CASE narrator-player-input-kept-for-event-continuation: an event continuation is not a history entry, so it keeps its Player Input block.
#[test]
fn test_an_event_continuation_keeps_its_block() {
    let fixture = Fixture::new();
    let history = history_ending_with_the_door_input(&fixture);

    let event = sent_prompt(
        &fixture.case_for(Generation::Event(event_record()), &history, None),
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

// CASE narrator-empty-history-placeholder: an empty history keeps its placeholder text in the first user message.
#[test]
fn test_an_empty_history_keeps_its_placeholder_in_the_first_user_message() {
    let fixture = Fixture::new();

    let call = sent_prompt(&fixture.narration(&[], "I look around."), &preset());

    assert!(
        history_parts(&call).is_empty(),
        "no entry becomes a message: {:?}",
        call.parts
    );
    assert!(
        call.parts[1].text.contains(
            "<ConversationHistory>\n(No history yet - this is the start of the conversation)\n</ConversationHistory>"
        ),
        "the first user message holds the placeholder: {}",
        call.parts[1].text
    );
}

// CASE narrator-history-cut-holds-between-blocks: one more Message past the space leaves the history start where it was.
#[test]
fn test_the_history_start_holds_between_cuts() {
    let fixture = Fixture::new();
    let history = fixture.history_of(1101);

    let first = sent_under(
        &fixture,
        &history[..1100],
        "I look around.",
        TokenBudget::fallback(),
    );
    let second = sent_under(
        &fixture,
        &history,
        "I look around.",
        TokenBudget::fallback(),
    );

    let start = first_kept_entry(&first).expect("the first call keeps entries");
    assert!(0 < start, "the first call is past the space");
    assert_eq!(first_kept_entry(&second), Some(start));
}

// CASE narrator-history-cut-ignores-the-player-input: a Player Input under the reserve does not move the history start.
#[test]
fn test_the_player_input_does_not_move_the_history_start() {
    let fixture = Fixture::new();
    let history = fixture.history_of(1100);
    let long_input = "I look around the room slowly. ".repeat(100);

    let short = sent_under(&fixture, &history, "Go.", TokenBudget::fallback());
    let long = sent_under(&fixture, &history, &long_input, TokenBudget::fallback());

    assert!(first_kept_entry(&short).is_some_and(|start| 0 < start));
    assert_eq!(first_kept_entry(&short), first_kept_entry(&long));
}

// CASE narrator-history-cut-keeps-max-tokens: a cut history leaves the requested response length unchanged.
#[test]
fn test_a_cut_history_keeps_the_requested_response_length() {
    let fixture = Fixture::new();
    let history = fixture.history_of(2000);

    let call = sent_under(&fixture, &history, "I look around.", window(8192, 500));

    assert!(first_kept_entry(&call).is_some_and(|start| 0 < start));
    assert_eq!(call.settings.max_tokens, 500);
}

// CASE narrator-history-cut-marker-when-nothing-kept: Messages exist and the space keeps none, so the history holds the marker.
#[test]
fn test_a_history_with_no_kept_message_holds_the_marker() {
    let fixture = Fixture::new();
    let mut history = fixture.history_of(1);
    history[0].text = "A very long entry. ".repeat(5000);

    let call = sent_under(
        &fixture,
        &history,
        "I look around.",
        TokenBudget::fallback(),
    );

    assert!(
        history_parts(&call).is_empty(),
        "no entry fits, so none is sent"
    );
    assert!(
        call.parts[1].text.contains(
            "<ConversationHistory>\n(History truncated to fit context window)\n</ConversationHistory>"
        ),
        "the first user message holds the marker: {}",
        call.parts[1].text
    );
}

// CASE narrator-history-cut-guards-the-limit: a Player Input past the reserve drops more Messages, and the prompt fits the input limit.
#[test]
fn test_a_long_player_input_drops_more_messages_to_fit() {
    let fixture = Fixture::new();
    let history = fixture.history_of(2000);
    let long_input = "I look around the room slowly. ".repeat(600);

    let call = sent_under(&fixture, &history, &long_input, window(8192, 500));

    let input_limit = 8192 - SAFETY_MARGIN_TOKENS as usize - 500;
    let sent = sent_tokens(&call);
    assert!(
        sent <= input_limit,
        "the prompt fits the input limit: {sent} of {input_limit}"
    );
    assert!(
        call.user_prompt().contains("Entry 1999 "),
        "the newest entry stays"
    );
}

// CASE narrator-history-budget-counts-message-overhead: a long history of short entries counts the chat-template cost of each message, so the prompt with that cost fits the input limit.
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

    let call = sent_under(&fixture, &history, "Go on.", window(8192, 500));

    let input_limit = 8192 - SAFETY_MARGIN_TOKENS as usize - 500;
    let sent = sent_tokens(&call);
    assert!(
        sent <= input_limit,
        "the prompt with the cost of each message fits the input limit: {sent} of {input_limit}"
    );
    assert!(history_parts(&call).len() < history.len());
}

// CASE narrator-response-cap: the call asks for the budget's response length.
#[test]
fn test_the_call_asks_for_the_budgets_response_length() {
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "I look around.");
    let requested = TokenBudget {
        max_context_tokens: MAX_CONTEXT_TOKENS,
        max_tokens: 500,
    };
    let model = Arc::new(ScriptedModel::new().narration("Narration."));

    narrate_with(&model, &case, &preset(), requested).expect("a narration is a result");

    assert_eq!(
        prompt_of(&model).settings.max_tokens,
        500,
        "the call asks for the budget's response length"
    );
}

// CASE narrator-posture-macros: the narrator stamps the resolved perspective and tense onto the preset's macros.
#[test]
fn test_the_narrator_stamps_the_resolved_posture() {
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "I look around.");

    let call = sent_prompt(&case, &preset());

    assert!(
        call.user_prompt()
            .contains("second-person limited perspective"),
        "the resolved perspective must win: {}",
        call.user_prompt()
    );
    assert!(
        call.user_prompt().contains("present tense narrative prose"),
        "the resolved tense must win: {}",
        call.user_prompt()
    );
    assert!(
        !call.user_prompt().contains("{{narrative_perspective}}")
            && !call.user_prompt().contains("{{narrative_tense}}"),
        "no posture macro stays unrendered: {}",
        call.user_prompt()
    );
}

// CASE narrator-guide-layer: the guide renders last, in its wrapper.
#[test]
fn test_the_guide_renders_last_in_its_wrapper() {
    let fixture = Fixture::new();
    let case = fixture
        .narration(&fixture.history, "I look around.")
        .with_guide(Some("Add tension.".to_string()));

    let user = sent_prompt(&case, &preset()).user_prompt();

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

// CASE narrator-guide-absent: a plain turn, and a blank guide, render no guide layer.
#[test]
fn test_a_plain_turn_and_a_blank_guide_render_no_guide_layer() {
    let fixture = Fixture::new();
    let plain = fixture.narration(&fixture.history, "I look around.");
    assert!(
        !sent_prompt(&plain, &preset())
            .user_prompt()
            .contains("<Guide>"),
        "a plain turn carries no guide layer"
    );

    let blank = plain.with_guide(Some("   ".to_string()));
    assert!(
        !sent_prompt(&blank, &preset())
            .user_prompt()
            .contains("<Guide>"),
        "a whitespace-only guide renders nothing"
    );
}

// CASE narrator-impersonate-layer: an impersonate prompt drops the player-character layer and renders no guide.
#[test]
fn test_an_impersonate_prompt_drops_the_player_layer_and_the_guide() {
    let fixture = Fixture::new();
    let case = fixture
        .narration(&fixture.history, "")
        .impersonating(Some("Ask about the gate.".to_string()));

    let user = sent_prompt(&case, &preset()).user_prompt();

    assert!(
        !user.contains("<Persona>"),
        "an impersonate turn drops the player layer: {user}"
    );
    assert!(!user.contains("<Guide>"), "and renders no guide: {user}");
}

// CASE narrator-impersonate-persona-macros: the impersonate preset's persona macros render.
#[test]
fn test_the_impersonate_preset_renders_its_persona_macros() {
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "").impersonating(None);

    let system = sent_prompt(&case, &impersonate_preset()).system_prompt();

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

// CASE narrator-global-rules: the World's global rules render into the system prompt.
#[test]
fn test_the_world_global_rules_render_into_the_system_prompt() {
    let mut fixture = Fixture::new();
    fixture.world.global_rules = vec!["Magic is forbidden.".to_string()];

    let call = sent_prompt(
        &fixture.narration(&fixture.history, "I look around."),
        &preset(),
    );

    assert!(
        call.system_prompt().contains("Magic is forbidden."),
        "the rules belong to the system prompt: {}",
        call.system_prompt()
    );
}

// CASE narrator-character-summary: a Character with a summary renders that summary, not the long description.
#[test]
fn test_a_character_with_a_summary_renders_that_summary() {
    let character = character_card("n1", "Alice", Some("Short summary."), "Long desc.", vec![]);
    let fixture = Fixture::new().with_characters(vec![character.clone()], vec![character]);

    let call = sent_prompt(
        &fixture.narration(&fixture.history, "I look around."),
        &preset(),
    );
    let known = prompt_block(
        &call.user_prompt(),
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

// CASE narrator-character-description-lines: a Character without a summary renders the first three description lines.
#[test]
fn test_a_character_without_a_summary_renders_three_description_lines() {
    let description = "line one.\nline two.\nline three.\nline four.";
    let character = character_card("n2", "Bob", None, description, vec![]);
    // Known to the World, not among the Characters present.
    let fixture = Fixture::new().with_characters(vec![character], vec![]);

    let call = sent_prompt(
        &fixture.narration(&fixture.history, "I look around."),
        &preset(),
    );
    let known = prompt_block(
        &call.user_prompt(),
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

// CASE narrator-known-entry-has-no-presence-mark: a present and an absent Character both render as a bare name line.
#[test]
fn test_a_known_entry_renders_the_bare_name_whether_present_or_absent() {
    let alice = character_card("n1", "Alice", Some("Here."), "Long desc.", vec![]);
    let bob = character_card("n2", "Bob", Some("Away."), "Long desc.", vec![]);
    let fixture = Fixture::new().with_characters(vec![alice.clone(), bob], vec![alice]);

    let call = sent_prompt(
        &fixture.narration(&fixture.history, "I look around."),
        &preset(),
    );
    let known = prompt_block(
        &call.user_prompt(),
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

// CASE narrator-character-relationships-absent: no relationships block renders while the related Character is away.
#[test]
fn test_no_relationships_block_renders_while_the_related_character_is_away() {
    let partner = character_card("p1", "Partner", None, "d", vec![]);
    let self_card = character_card(
        "n3",
        "Self",
        None,
        "d",
        vec![Relationship {
            with: "p1".to_string(),
            static_text: "allies".to_string(),
            dynamic: String::new(),
        }],
    );
    let fixture = Fixture::new().with_characters(vec![partner, self_card.clone()], vec![self_card]);

    let call = sent_prompt(
        &fixture.narration(&fixture.history, "I look around."),
        &preset(),
    );

    assert!(
        !call.user_prompt().contains("Relationships:"),
        "an away Character has no relationships block: {}",
        call.user_prompt()
    );
}

// CASE narrator-character-relationships-present: the relationships block renders the present Character's name.
#[test]
fn test_the_relationships_block_renders_the_present_characters_name() {
    let partner = character_card("p1", "Partner", None, "d", vec![]);
    let self_card = character_card(
        "n4",
        "Self",
        None,
        "d",
        vec![Relationship {
            with: "p1".to_string(),
            static_text: "allies".to_string(),
            dynamic: String::new(),
        }],
    );
    let fixture = Fixture::new().with_characters(
        vec![partner.clone(), self_card.clone()],
        vec![partner, self_card],
    );

    let call = sent_prompt(
        &fixture.narration(&fixture.history, "I look around."),
        &preset(),
    );

    assert!(
        call.user_prompt().contains("Relationships:"),
        "a present Character has a relationships block: {}",
        call.user_prompt()
    );
    assert!(
        call.user_prompt().contains("Partner") && call.user_prompt().contains("allies"),
        "the block names the related Character and its text: {}",
        call.user_prompt()
    );
}

// CASE narrator-cards-in-id-order: both card lists render in Character id order, whatever order the context holds them in.
#[test]
fn test_both_card_lists_render_in_character_id_order() {
    let ada = character_card("a", "Ada", None, "first", vec![]);
    let bea = character_card("b", "Bea", None, "second", vec![]);
    let cid = character_card("c", "Cid", None, "third", vec![]);
    let out_of_order = vec![cid.clone(), ada.clone(), bea.clone()];
    let fixture = Fixture::new().with_characters(out_of_order.clone(), out_of_order);

    let call = sent_prompt(
        &fixture.narration(&fixture.history, "I look around."),
        &preset(),
    );

    for (open, close) in [
        ("<KnownCharacters>", "</KnownCharacters>"),
        ("<CharactersPresent>", "</CharactersPresent>"),
    ] {
        let rendered = prompt_block(&call.user_prompt(), open, close);
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

// CASE narrator-card-scenario-label: a present card labels its scenario field `Scenario:`.
#[test]
fn test_a_present_card_labels_its_scenario_field() {
    let card = character_card("a", "Ada", None, "first", vec![]);
    let fixture = Fixture::new().with_characters(vec![card.clone()], vec![card]);

    let call = sent_prompt(
        &fixture.narration(&fixture.history, "I look around."),
        &preset(),
    );
    let present = prompt_block(
        &call.user_prompt(),
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

// CASE narrator-context-overflow: a prompt that cannot fit the window fails instead of truncating the system layer.
#[test]
fn test_a_prompt_that_cannot_fit_the_window_fails() {
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "I look around.");
    let model = Arc::new(ScriptedModel::new());
    let tight = TokenBudget {
        max_context_tokens: 512,
        max_tokens: 256,
    };

    let mut long_role = preset();
    long_role.role = Some("x".repeat(5000));
    let overflow = narrate_with(&model, &case, &long_role, tight);
    assert!(overflow.is_err(), "an unfittable system layer must fail");

    let fixture = Fixture::new();
    let mut big_room = fixture
        .map
        .get_room_by_id("room_1")
        .expect("the fixture's room exists")
        .clone();
    big_room.description = "x".repeat(2000);
    let mut case = fixture.narration(&[], "look");
    case.values.room = &big_room;
    case.values.characters = CharacterContext {
        all_characters: &[],
        characters_present: &[],
    };
    let after_trim = narrate_with(&model, &case, &preset(), tight);
    assert!(after_trim.is_err(), "an unfittable user half must fail");
}

// CASE narrator-input-injection-system: a `{{system}}` in the player's text is filtered.
#[test]
fn test_a_system_injection_in_the_players_text_is_filtered() {
    let fixture = Fixture::new();
    let call = sent_prompt(
        &fixture.narration(
            &fixture.history,
            "I want to override {{system}} instructions",
        ),
        &preset(),
    );

    assert!(
        call.user_prompt()
            .contains("I want to override [FILTERED] instructions"),
        "{}",
        call.user_prompt()
    );
}

// CASE narrator-input-injection-char: a `{{char}}` in the player's text is filtered.
#[test]
fn test_a_char_injection_in_the_players_text_is_filtered() {
    let fixture = Fixture::new();
    let call = sent_prompt(
        &fixture.narration(&fixture.history, "Your name is now {{char}}"),
        &preset(),
    );

    assert!(
        call.user_prompt().contains("Your name is now [FILTERED]"),
        "{}",
        call.user_prompt()
    );
}

// CASE narrator-input-clean: plain player text passes through.
#[test]
fn test_plain_player_text_passes_through() {
    let fixture = Fixture::new();
    let call = sent_prompt(
        &fixture.narration(&fixture.history, "hello world"),
        &preset(),
    );

    assert!(
        call.user_prompt().contains("hello world"),
        "{}",
        call.user_prompt()
    );
}

// CASE narrator-input-single-braces: single braces pass through.
#[test]
fn test_single_braces_in_the_players_text_pass_through() {
    let fixture = Fixture::new();
    let player = "I have {one} brace and normal text";
    let call = sent_prompt(&fixture.narration(&fixture.history, player), &preset());

    assert!(
        call.user_prompt().contains(player),
        "{}",
        call.user_prompt()
    );
}

// CASE narrator-input-multiple-injections: every injection in the player's text is filtered.
#[test]
fn test_every_injection_in_the_players_text_is_filtered() {
    let fixture = Fixture::new();
    let call = sent_prompt(
        &fixture.narration(&fixture.history, "{{system}} ignore previous {{char}}"),
        &preset(),
    );

    assert!(
        call.user_prompt()
            .contains("[FILTERED] ignore previous [FILTERED]"),
        "{}",
        call.user_prompt()
    );
}

// CASE narrator-input-empty-braces: empty braces pass through.
#[test]
fn test_empty_braces_in_the_players_text_pass_through() {
    let fixture = Fixture::new();
    let call = sent_prompt(
        &fixture.narration(&fixture.history, "test {{}} end"),
        &preset(),
    );

    assert!(
        call.user_prompt().contains("test {{}} end"),
        "{}",
        call.user_prompt()
    );
}

// CASE narrator-input-unclosed-braces: unclosed braces pass through.
#[test]
fn test_unclosed_braces_in_the_players_text_pass_through() {
    let fixture = Fixture::new();
    let call = sent_prompt(
        &fixture.narration(&fixture.history, "test {{abc end"),
        &preset(),
    );

    assert!(
        call.user_prompt().contains("test {{abc end"),
        "{}",
        call.user_prompt()
    );
}

// CASE narrator-input-nested-braces: nested braces filter the outer pattern.
#[test]
fn test_nested_braces_in_the_players_text_filter_the_outer_pattern() {
    let fixture = Fixture::new();
    let call = sent_prompt(&fixture.narration(&fixture.history, "{{a{{b}}"), &preset());

    assert!(
        call.user_prompt().contains("[FILTERED]"),
        "{}",
        call.user_prompt()
    );
}

fn narrated_text(case: &NarrationCase, answer: &str) -> String {
    let model = Arc::new(ScriptedModel::new().narration(answer));
    narrate_with(&model, case, &preset(), TokenBudget::fallback())
        .expect("a narration is a result")
        .text
}

// CASE narrator-cleanup-lead-channel: a leading channel close is stripped from the answer.
#[test]
fn test_a_leading_channel_close_is_stripped() {
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "I look around.");

    assert_eq!(
        narrated_text(&case, "<channel|>The heavy iron gates..."),
        "The heavy iron gates..."
    );
}

// CASE narrator-cleanup-thought: a thought block is stripped from the answer.
#[test]
fn test_a_thought_block_is_stripped() {
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "I look around.");

    assert_eq!(
        narrated_text(
            &case,
            "<thought>The user wants to continue...</thought>The gates creaked."
        ),
        "The gates creaked."
    );
}

// CASE narrator-cleanup-channel-thought: a channel-thought block is stripped from the answer.
#[test]
fn test_a_channel_thought_block_is_stripped() {
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "I look around.");

    assert_eq!(
        narrated_text(
            &case,
            "<|channel>thought\nSome reasoning here\n<channel|>Narrative text."
        ),
        "Narrative text."
    );
}

// CASE narrator-cleanup-turn-markers: orphan turn markers are stripped from the answer.
#[test]
fn test_orphan_turn_markers_are_stripped() {
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "I look around.");

    assert_eq!(
        narrated_text(
            &case,
            "<|turn>modelStart of text<turn|>more text<|turn>end."
        ),
        "Start of textmore textend."
    );
}

// CASE narrator-cleanup-combined: every artifact in one answer is stripped.
#[test]
fn test_every_artifact_in_one_answer_is_stripped() {
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "I look around.");

    assert_eq!(
        narrated_text(
            &case,
            "<channel|><thought>reasoning</thought><|turn>modelThe real content."
        ),
        "The real content."
    );
}

// CASE narrator-cleanup-clean: a clean answer passes through.
#[test]
fn test_a_clean_answer_passes_through() {
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "I look around.");
    let answer = "The heavy iron gates offered no resistance.";

    assert_eq!(narrated_text(&case, answer), answer);
}

// CASE narrator-cleanup-multiple-blocks: two thought blocks are both stripped.
#[test]
fn test_two_thought_blocks_are_both_stripped() {
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "I look around.");

    assert_eq!(
        narrated_text(&case, "<thought>first</thought>A<thought>second</thought>B"),
        "AB"
    );
}

// CASE narrator-cleanup-lines: leading whitespace is trimmed on every line.
#[test]
fn test_leading_whitespace_is_trimmed_on_every_line() {
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "I look around.");

    assert_eq!(
        narrated_text(
            &case,
            "  First paragraph.\n\n        Second paragraph.\n\n        Third paragraph."
        ),
        "First paragraph.\n\nSecond paragraph.\n\nThird paragraph."
    );
}

// CASE narrator-empty-answer: an empty or whitespace-only answer fails, so no empty Narration is saved.
#[test]
fn test_an_empty_or_whitespace_only_answer_fails() {
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "I look around.");

    for answer in ["", "   "] {
        let model = Arc::new(ScriptedModel::new().narration(answer));
        let result = narrate_with(&model, &case, &preset(), TokenBudget::fallback());
        match result {
            Err(failure) => assert_eq!(
                failure.kind,
                crate::domain::model::state::generation_status::GenerationFailureKind::EmptyText,
                "an empty answer is its own kind, so its own line shows"
            ),
            Ok(narrated) => panic!("an answer of {answer:?} is no narration: {narrated:?}"),
        }
    }
}

// CASE narrator-transport-failure: a failed call fails the narration.
#[test]
fn test_a_failed_call_fails_the_narration() {
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "I look around.");
    let model = Arc::new(ScriptedModel::new().failed(Role::Narrator));

    let result = narrate_with(&model, &case, &preset(), TokenBudget::fallback());

    assert!(result.is_err(), "a failed call is a failed narration");
}

fn recording_narrator(
    model: Arc<ScriptedModel>,
    storage: &Arc<crate::adapters::driven::storage::Storage>,
) -> Narrator {
    let recorder: Arc<LlmCallRecorder> =
        make_test_recorder_with_storage(model, Arc::clone(storage));
    Narrator::new(PromptAssembler, Arc::clone(&recorder), recorder)
}

// CASE narrator-call-record: a good call is recorded with its agent name and prompts.
#[test]
fn test_a_good_call_is_recorded_with_its_prompts() {
    let storage = Arc::new(crate::adapters::driven::storage::Storage::new_in_memory());
    let model = Arc::new(ScriptedModel::new().narration("You look around."));
    let narrator = recording_narrator(model, &storage);
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "I look around.");

    narrate_on(&narrator, &case, &preset(), TokenBudget::fallback())
        .expect("a narration is a result");

    let recorded = storage
        .list_latest_llm_messages(10)
        .expect("the forensics read back");
    let call = recorded.last().expect("the call is recorded");
    assert_eq!(call.agent_name, "narrator");
    assert!(call.error_message.is_none());
    assert!(!sent_system_text(call).is_empty());
    assert!(!sent_user_text(call).is_empty());
}

// CASE narrator-impersonate-uses-its-own-recorder: an Impersonate calls the recorder of its own slot, and every other kind calls the narration recorder.
#[test]
fn test_an_impersonate_calls_its_own_recorder() {
    let narration_model = Arc::new(ScriptedModel::new().narration("You look around."));
    let impersonate_model = Arc::new(ScriptedModel::new().narration("I look around."));
    let narrator = Narrator::new(
        PromptAssembler,
        make_test_recorder(narration_model.clone()),
        make_test_recorder(impersonate_model.clone()),
    );
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, TYPED_INPUT);
    let impersonation = fixture
        .narration(&fixture.history, TYPED_INPUT)
        .impersonating(None);

    narrate_on(
        &narrator,
        &impersonation,
        &impersonate_preset(),
        TokenBudget::fallback(),
    )
    .expect("an impersonation is a result");
    assert_eq!(impersonate_model.calls().len(), 1);
    assert!(
        narration_model.calls().is_empty(),
        "an Impersonate never calls the narration recorder"
    );

    narrate_on(&narrator, &case, &preset(), TokenBudget::fallback())
        .expect("a narration is a result");
    assert_eq!(narration_model.calls().len(), 1);
    assert_eq!(
        impersonate_model.calls().len(),
        1,
        "a Narration never calls the Impersonate recorder"
    );
}

fn event_record() -> EventRecord {
    EventRecord {
        trigger: TriggerRef {
            character_id: "character_1".to_string(),
            trigger_idx: 0,
            trigger_name: "The gate opens".to_string(),
            trigger_repeat: false,
        },
        header_name: "The gate opens".to_string(),
        direction: "the gate swings open".to_string(),
    }
}

// CASE narrator-kind-role: the call carries the role of its Generation, so an event
// Narration feeds the Trigger's health and every other kind feeds the narrator's.
#[test]
fn test_a_call_carries_the_role_of_its_generation() {
    let rows = [
        (Generation::Narration { guide: None }, false, "narrator"),
        (
            Generation::Impersonate { direction: None },
            false,
            "narrator",
        ),
        (Generation::InputRetry, false, "narrator"),
        (Generation::Event(event_record()), true, "trigger"),
    ];

    for (generation, is_event, agent_name) in rows {
        let storage = Arc::new(crate::adapters::driven::storage::Storage::new_in_memory());
        let model = Arc::new(if is_event {
            ScriptedModel::new().event_narration("The gate swings open.")
        } else {
            ScriptedModel::new().narration("You look around.")
        });
        let narrator = recording_narrator(model, &storage);
        let fixture = Fixture::new();
        let mut case = fixture.narration(&fixture.history, TYPED_INPUT);
        case.generation = generation;

        narrate_on(&narrator, &case, &preset(), TokenBudget::fallback())
            .expect("the narration is a result");

        let recorded = storage
            .list_latest_llm_messages(10)
            .expect("the forensics read back");
        let call = recorded.last().expect("the call is recorded");
        assert_eq!(call.agent_name, agent_name, "{:?}", case.generation.kind());
    }
}

// CASE narrator-user-message-by-kind: each Generation makes its own user message and flags from the values it is given, with no storage.
#[test]
fn test_each_generation_makes_its_user_message_and_flags() {
    let fixture = Fixture::new();
    let history = fixture.history.clone();
    let rows = [
        // A Narration answers the request's own Input, and a guide renders after it.
        (
            Generation::Narration {
                guide: Some("Add tension.".to_string()),
            },
            Some("I open the door."),
            "I open the door.",
            Some("Add tension."),
            true,
        ),
        // An Impersonate takes the Player's direction, and drops the player layer.
        (
            Generation::Impersonate {
                direction: Some("Ask about the gate.".to_string()),
            },
            None,
            "Ask about the gate.",
            None,
            false,
        ),
        // The Retry of a typed Input repeats the Input the tail ends with.
        (Generation::InputRetry, None, TYPED_INPUT, None, true),
        // An event continues the narration that fired it.
        (
            Generation::Event(event_record()),
            None,
            "Previous narration:\nYou enter the inn.\n\nTrigger direction: the gate swings open\n\nContinue the scene naturally",
            None,
            true,
        ),
    ];

    for (generation, input, message, guide, player_layer) in rows {
        let label = format!("{:?}", generation.kind());
        let case = fixture.case_for(generation, &history, input);

        let user = sent_prompt(&case, &preset()).user_prompt();

        let rendered = prompt_block(&user, "<PlayerInput>", "</PlayerInput>");
        assert!(
            rendered.contains(message),
            "{label}: the user message: {rendered}"
        );
        assert_eq!(
            user.contains("<Persona>"),
            player_layer,
            "{label}: the player layer: {user}"
        );
        match guide {
            Some(guide) => assert!(user.contains(guide), "{label}: the guide: {user}"),
            None => assert!(!user.contains("<Guide>"), "{label}: no guide: {user}"),
        }
    }
}

// CASE narrator-user-message-previous-input: a Continue answers the Input that ends the window.
#[test]
fn test_a_continue_answers_the_input_that_ends_the_window() {
    let fixture = Fixture::new();
    let mut ends_in_input = fixture.history.clone();
    ends_in_input.push(StoryRow {
        id: 2,
        text: "I wait.".to_string(),
        message_type: MessageType::Input,
        ..default_story_row(MessageType::Input)
    });

    let after_input = fixture.narration(&ends_in_input, "");
    let call = sent_prompt(&after_input, &preset());
    let last = call.parts.last().expect("a call has parts");
    assert!(
        last.role == PromptRole::User
            && last
                .text
                .starts_with("<LastMessage>\nI wait.\n</LastMessage>"),
        "a Continue answers the last Input, which is the last message: {}",
        last.text
    );
    assert!(
        !call.user_prompt().contains("<PlayerInput>"),
        "the Input is not sent twice: {}",
        call.user_prompt()
    );
}

// CASE narrator-user-message-after-narration-empty: a Continue after a Narration renders no player-input layer.
#[test]
fn test_a_continue_after_a_narration_renders_no_player_input_layer() {
    let fixture = Fixture::new();
    let after_narration = fixture.narration(&fixture.history, "");
    let user = sent_prompt(&after_narration, &preset()).user_prompt();

    assert!(
        !user.contains("<PlayerInput>"),
        "a blank player input renders no layer at all: {user}"
    );
    assert!(
        user.trim_end().ends_with("</output_format>"),
        "the user message ends with the post-history splice: {user}"
    );
}

// CASE narrator-guide-after-narration: a Guided Generation with no Input after a Narration renders the guide and no player-input layer.
#[test]
fn test_a_guide_after_a_narration_renders_the_guide_and_no_player_input_layer() {
    let fixture = Fixture::new();
    let case = fixture
        .narration(&fixture.history, "")
        .with_guide(Some("Add tension.".to_string()));
    let user = sent_prompt(&case, &preset()).user_prompt();

    assert!(
        !user.contains("<PlayerInput>"),
        "a blank player input renders no layer at all: {user}"
    );
    assert!(
        user.trim_end().ends_with("</Guide>"),
        "the guide ends the user message: {user}"
    );
}

// CASE narrator-blank-player-input-renders-nothing: an Impersonate with no direction ends the user message at the post-history splice.
#[test]
fn test_an_impersonate_with_no_direction_ends_at_the_post_history_splice() {
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "").impersonating(None);
    let user = sent_prompt(&case, &preset()).user_prompt();

    assert!(
        !user.contains("<PlayerInput>"),
        "a blank direction renders no player-input layer: {user}"
    );
    assert!(
        user.trim_end().ends_with("</output_format>"),
        "the user message ends with the post-history splice: {user}"
    );
}

// CASE narrator-call-record-failure: a call that fails after the send is recorded with its failure text and the messages it sent.
#[test]
fn test_a_failed_call_is_recorded_with_its_failure_text() {
    let storage = Arc::new(crate::adapters::driven::storage::Storage::new_in_memory());
    let model = Arc::new(ScriptedModel::new().failed(Role::Narrator));
    let narrator = recording_narrator(Arc::clone(&model), &storage);
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "I look around.");

    let _ = narrate_on(&narrator, &case, &preset(), TokenBudget::fallback());

    let recorded = storage
        .list_latest_llm_messages(10)
        .expect("the forensics read back");
    let failure = recorded
        .iter()
        .find(|message| message.error_message.is_some())
        .expect("the failed attempt is recorded");
    assert_eq!(failure.agent_name, "narrator");
    assert_eq!(
        failure.error_message.as_deref(),
        Some("LLM Error: Narration generation failed at stage 'scripted': configured_failure")
    );
    let payload: serde_json::Value = serde_json::from_str(&failure.raw_request_json)
        .expect("a failure after the send stores the payload");
    let sent: Vec<serde_json::Value> = prompt_of(&model)
        .parts
        .iter()
        .map(PromptPart::to_json)
        .collect();
    assert_eq!(payload["messages"], serde_json::json!(sent));
}

// CASE narrator-sends-the-preset-parameters: the samplers of the preset and the Game reach the model call, for a Narration and for an Impersonate, and the response length comes from the budget.
#[test]
fn test_the_preset_samplers_and_the_game_reach_the_model_call() {
    let tuned = PromptPreset {
        parameters: GenerationParameters {
            max_tokens: Some(9999),
            temperature: Some(0.45),
            top_p: Some(0.85),
            custom: serde_json::json!({"top_k": 30})
                .as_object()
                .unwrap()
                .clone(),
        },
        ..preset()
    };
    let fixture = Fixture::new();
    let budget = TokenBudget {
        max_context_tokens: MAX_CONTEXT_TOKENS,
        max_tokens: 700,
    };
    for case in [
        fixture.narration(&fixture.history, TYPED_INPUT),
        fixture
            .narration(&fixture.history, TYPED_INPUT)
            .impersonating(None),
    ] {
        let model = Arc::new(ScriptedModel::new().narration("Narration."));

        narrate_on(&narrator(Arc::clone(&model)), &case, &tuned, budget)
            .expect("a narration is a result");

        let sent = prompt_of(&model).settings;
        assert_eq!(sent.temperature, Some(0.45));
        assert_eq!(sent.top_p, Some(0.85));
        assert_eq!(sent.custom["top_k"], serde_json::json!(30));
        assert_eq!(sent.game, Some(GameId(7)));
        assert_eq!(
            sent.max_tokens, 700,
            "the response length is the budget's: the Job resolved the preset value and the ceiling"
        );
    }
}

// CASE narrator-sends-no-sampler-when-the-preset-sets-none: a preset with no parameters sends no sampler and no custom key.
#[test]
fn test_a_preset_without_parameters_sends_no_sampler() {
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, TYPED_INPUT);
    let model = Arc::new(ScriptedModel::new().narration("Narration."));

    narrate_with(&model, &case, &preset(), TokenBudget::fallback())
        .expect("a narration is a result");

    let sent = prompt_of(&model).settings;
    assert_eq!(sent.temperature, None);
    assert_eq!(sent.top_p, None);
    assert!(sent.custom.is_empty());
}
