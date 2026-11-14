//! Seam contract of the Narrator, the box that turns a generation into one prompt and one call.
//! The prompt itself is the `PromptAssembler` contract (`prompting/assembler_tests.rs`).
//! These cases pin only what the Narrator decides.

use std::sync::Arc;

use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::narration::{NarrationValues, Narrated, Narrator};
use crate::application::prompting::token_budget::{TokenBudget, MAX_CONTEXT_TOKENS, STORY_MEMORY_CHARS};
use crate::application::prompting::{CharacterContext, PromptAssembler};
use crate::domain::model::agent::Role;
use crate::domain::model::character::{CharacterCard, PersonaCard};
use crate::domain::model::game::GameId;
use crate::domain::model::generation::Generation;
use crate::domain::model::generation_parameters::{CallSettings, GenerationParameters};
use crate::domain::model::map::MapDef;
use crate::domain::model::message::{Maker, Message, Swipe};
use crate::domain::model::prompt_part::{PromptPart, PromptRole};
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::{NarrativePerspective, NarrativeTense, DEFAULT_MAX_HISTORY_TOKENS};
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

    fn look_around(&self) -> NarrationCase<'_> {
        self.narration(&self.history, "I look around.")
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
                story_memory: None,
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

impl<'a> NarrationCase<'a> {
    fn with_story_memory(mut self, story_memory: &'a str) -> Self {
        self.values.story_memory = Some(story_memory);
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
    narrator.narrate(&case.generation, &case.values, preset, &call, budget)
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

/// The number of the first history entry the call sends, from the
/// `Entry <n> ...` text of `Fixture::history_of`.
fn first_kept_entry(call: &ModelCall) -> Option<usize> {
    let first = call
        .parts
        .iter()
        .skip(2)
        .map(|part| part.text.as_str())
        .find(|text| text.contains("Entry "))?;
    let digits: String = first[first.find("Entry ")? + "Entry ".len()..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

// CASE narrator-posture-macros: the narrator hands the resolved perspective and tense of its values to the prompt.
#[test]
fn test_the_narrator_hands_over_the_resolved_posture() {
    let fixture = Fixture::new();
    let mut case = fixture.look_around();
    case.values.narrative_perspective = NarrativePerspective::Third;
    case.values.narrative_tense = NarrativeTense::Past;

    let user = sent_prompt(&case, &preset()).user_prompt();

    assert!(
        user.contains("third-person limited perspective, past tense narrative prose."),
        "the resolved posture of the values reaches the prompt: {user}"
    );
}

// CASE narrator-global-rules: the narrator hands the World's global rules to the prompt.
#[test]
fn test_the_narrator_hands_over_the_world_global_rules() {
    let mut fixture = Fixture::new();
    fixture.world.global_rules = vec!["Magic is forbidden.".to_string()];

    let call = sent_prompt(&fixture.look_around(), &preset());

    assert!(
        call.system_prompt().contains("Magic is forbidden."),
        "the rules belong to the system prompt: {}",
        call.system_prompt()
    );
}

// CASE narrator-hands-over-the-scene: the room, the Characters and the Persona of the values reach the prompt.
#[test]
fn test_the_narrator_hands_over_the_room_the_characters_and_the_persona() {
    let mut fixture = Fixture::new();
    fixture.persona = TestPersona::named("Quill");
    fixture.characters = vec![TestCharacter::named("character_2", "Mira")];
    fixture.present = fixture.characters.clone();
    let mut room = fixture
        .map
        .get_room_by_id("room_1")
        .expect("the fixture's room exists")
        .clone();
    room.name = "The Lantern Cellar".to_string();
    let mut case = fixture.look_around();
    case.values.room = &room;

    let user = sent_prompt(&case, &preset()).user_prompt();

    for given in ["The Lantern Cellar", "Mira", "Quill"] {
        assert!(
            user.contains(given),
            "`{given}` of the values reaches the prompt: {user}"
        );
    }
}

// CASE narrator-context-overflow: a prompt that cannot fit the window fails the narration, and no call is sent.
#[test]
fn test_a_prompt_that_cannot_fit_the_window_fails_the_narration() {
    let fixture = Fixture::new();
    let model = Arc::new(ScriptedModel::new());
    let tight = TokenBudget {
        max_context_tokens: 512,
        max_history_tokens: DEFAULT_MAX_HISTORY_TOKENS,
        max_tokens: 256,
    };
    let mut long_role = preset();
    long_role.role = Some("x".repeat(5000));

    let overflow = narrate_with(&model, &fixture.look_around(), &long_role, tight);

    assert!(
        overflow.is_err(),
        "an unfittable prompt fails the narration"
    );
    assert!(model.calls().is_empty(), "no call is sent");
}

fn cleaned(answer: &str) -> String {
    let fixture = Fixture::new();
    let case = fixture.look_around();
    let model = Arc::new(ScriptedModel::new().narration(answer));
    narrate_with(&model, &case, &preset(), TokenBudget::fallback())
        .expect("a narration is a result")
        .text
}

// CASE narrator-cleanup-lead-channel: a leading channel close is stripped from the answer.
#[test]
fn test_a_leading_channel_close_is_stripped() {
    assert_eq!(cleaned("<channel|>The gates..."), "The gates...");
}

// CASE narrator-cleanup-thought: a thought block is stripped from the answer.
#[test]
fn test_a_thought_block_is_stripped() {
    assert_eq!(
        cleaned("<thought>The user wants to continue...</thought>The gates creaked."),
        "The gates creaked."
    );
}

// CASE narrator-cleanup-channel-thought: a channel-thought block is stripped from the answer.
#[test]
fn test_a_channel_thought_block_is_stripped() {
    assert_eq!(
        cleaned("<|channel>thought\nSome reasoning here\n<channel|>Narrative text."),
        "Narrative text."
    );
}

// CASE narrator-cleanup-turn-markers: orphan turn markers are stripped from the answer.
#[test]
fn test_orphan_turn_markers_are_stripped() {
    assert_eq!(
        cleaned("<|turn>modelStart of text<turn|>more text<|turn>end."),
        "Start of textmore textend."
    );
}

// CASE narrator-cleanup-combined: every artifact in one answer is stripped.
#[test]
fn test_every_artifact_in_one_answer_is_stripped() {
    assert_eq!(
        cleaned("<channel|><thought>reasoning</thought><|turn>modelThe real content."),
        "The real content."
    );
}

// CASE narrator-cleanup-clean: a clean answer passes through.
#[test]
fn test_a_clean_answer_passes_through() {
    let answer = "The heavy iron gates offered no resistance.";
    assert_eq!(cleaned(answer), answer);
}

// CASE narrator-cleanup-multiple-blocks: two thought blocks are both stripped.
#[test]
fn test_two_thought_blocks_are_both_stripped() {
    assert_eq!(
        cleaned("<thought>first</thought>A<thought>second</thought>B"),
        "AB"
    );
}

// CASE narrator-cleanup-lines: leading whitespace is trimmed on every line.
#[test]
fn test_leading_whitespace_is_trimmed_on_every_line() {
    assert_eq!(
        cleaned("  First paragraph.\n\n        Second paragraph.\n\n        Third paragraph."),
        "First paragraph.\n\nSecond paragraph.\n\nThird paragraph."
    );
}

// CASE narrator-empty-answer: an empty or whitespace-only answer fails, so no empty Narration is saved.
#[test]
fn test_an_empty_or_whitespace_only_answer_fails() {
    let fixture = Fixture::new();
    let case = fixture.look_around();

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
    let case = fixture.look_around();
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
    let case = fixture.look_around();

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

// CASE narrator-user-message-after-narration-empty: a Continue after a Narration has an empty user message, so no player-input layer renders.
#[test]
fn test_a_continue_after_a_narration_has_an_empty_user_message() {
    let fixture = Fixture::new();
    let after_narration = fixture.narration(&fixture.history, "");
    let user = sent_prompt(&after_narration, &preset()).user_prompt();

    assert!(
        !user.contains("<PlayerInput>"),
        "a Continue after a Narration has no user message: {user}"
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
        "a Guided Generation with no Input has no user message: {user}"
    );
    assert!(user.contains("Add tension."), "the guide renders: {user}");
}

// CASE narrator-impersonate-no-direction-empty-message: an Impersonate with no direction has an empty user message, so no player-input layer renders.
#[test]
fn test_an_impersonate_with_no_direction_has_an_empty_user_message() {
    let fixture = Fixture::new();
    let case = fixture.narration(&fixture.history, "").impersonating(None);
    let user = sent_prompt(&case, &preset()).user_prompt();

    assert!(
        !user.contains("<PlayerInput>"),
        "a blank direction renders no player-input layer: {user}"
    );
}

// CASE narrator-call-record-failure: a call that fails after the send is recorded with its failure text and the messages it sent.
#[test]
fn test_a_failed_call_is_recorded_with_its_failure_text() {
    let storage = Arc::new(crate::adapters::driven::storage::Storage::new_in_memory());
    let model = Arc::new(ScriptedModel::new().failed(Role::Narrator));
    let narrator = recording_narrator(Arc::clone(&model), &storage);
    let fixture = Fixture::new();
    let case = fixture.look_around();

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
        max_history_tokens: DEFAULT_MAX_HISTORY_TOKENS,
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

const STORY_MEMORY: &str = "The innkeeper promised a room for three nights.";

fn sent_with_memory(case: NarrationCase, preset: &PromptPreset, budget: TokenBudget) -> ModelCall {
    let model = Arc::new(match case.generation {
        Generation::Event(_) => ScriptedModel::new().event_narration("Narration."),
        _ => ScriptedModel::new().narration("Narration."),
    });
    narrate_with(&model, &case, preset, budget).expect("a narration is a result");
    prompt_of(&model)
}

// CASE narrator-story-memory-every-kind: a Narration, an Impersonate, the Retry of a typed Input and an event Narration all show the story memory.
#[test]
fn test_every_generation_shows_the_story_memory() {
    let fixture = Fixture::new();
    for generation in [
        Generation::Narration { guide: None },
        Generation::Impersonate { direction: None },
        Generation::InputRetry,
        Generation::Event(event_record()),
    ] {
        let case = fixture
            .case_for(generation, &fixture.history, Some(TYPED_INPUT))
            .with_story_memory(STORY_MEMORY);
        let kind = case.generation.kind();

        let call = sent_with_memory(case, &preset(), TokenBudget::fallback());

        assert!(
            call.parts[1].text.contains(STORY_MEMORY),
            "{kind:?} shows the story memory: {}",
            call.parts[1].text
        );
    }
}

// CASE narrator-plan-cut-matches-the-call: the planned cut keeps the entries the call sends, with the user message and the story memory of the call.
#[test]
fn test_the_planned_cut_keeps_the_entries_the_call_sends() {
    let fixture = Fixture::new();
    let history = fixture.history_of(2000);
    let long_input = "I look around the room slowly. ".repeat(600);
    let memory = "m".repeat(STORY_MEMORY_CHARS);
    let budget = TokenBudget {
        max_context_tokens: 8192,
        max_history_tokens: DEFAULT_MAX_HISTORY_TOKENS,
        max_tokens: 500,
    };
    let case = fixture
        .narration(&history, &long_input)
        .with_story_memory(&memory);
    let model = Arc::new(ScriptedModel::new().narration("Narration."));
    let narrator = narrator(Arc::clone(&model));

    let planned = narrator.plan_cut(&case.generation, &case.values, &preset(), budget);
    narrate_on(&narrator, &case, &preset(), budget).expect("a narration is a result");

    assert!(planned.kept_from() > 0, "the history is cut");
    assert_eq!(
        first_kept_entry(&prompt_of(&model)),
        Some(planned.kept_from()),
        "the call sends the history from the planned start"
    );
}
