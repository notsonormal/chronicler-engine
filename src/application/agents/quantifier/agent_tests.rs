//! Seam contract of the Quantifier, the after-narration hook's Agent.

use std::collections::HashMap;
use std::sync::Arc;

use crate::application::agents::quantifier::agent::QuantifierAgent;
use crate::application::agents::trait_def::AfterNarrationAgent;
use crate::domain::model::agent::{AgentConfig, AgentSettings, HookInput, Role, WorldView};
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::character::CharacterCard;
use crate::domain::model::map::MapDef;
use crate::domain::model::state::message_types::{MessageType, StoryRow};
use crate::test_support::fixtures::{default_story_row, TestMap, TestCharacter, TestPersona};
use crate::test_support::{make_test_recorder, make_test_recorder_with_storage, ScriptedModel};
use crate::domain::model::generation_parameters::CallSettings;
use crate::domain::model::game::GameId;

const CHARACTER_ID: &str = "character_1";

fn config() -> AgentConfig {
    AgentConfig {
        name: "quantifier".to_string(),
        agent_type: "quantifier".to_string(),
        enabled: true,
    }
}

fn quantifier(model: Arc<ScriptedModel>) -> QuantifierAgent {
    QuantifierAgent::from_config(&config(), make_test_recorder(model))
        .expect("the quantifier builds from its config")
}

struct Turn {
    map: MapDef,
    persona: crate::domain::model::character::PersonaCard,
    characters: HashMap<String, CharacterCard>,
    present: Vec<CharacterCard>,
    story: Vec<StoryRow>,
    text: String,
}

impl Turn {
    fn new() -> Self {
        let map = TestMap::single_room("start");
        let persona = TestPersona::standard();
        let present = vec![TestCharacter::named(CHARACTER_ID, "Test Character")];
        let characters = present
            .iter()
            .map(|character| (character.id.clone(), character.clone()))
            .collect();
        Self {
            map,
            persona,
            characters,
            present,
            story: vec![StoryRow {
                message_type: MessageType::Narration,
                text: "The innkeeper glances your way.".to_string(),
                ..default_story_row(MessageType::Narration)
            }],
            text: "You look around the inn.".to_string(),
        }
    }

    fn with_no_history(mut self) -> Self {
        self.story.clear();
        self
    }

    fn with_cards(mut self, cards: Vec<CharacterCard>, present: Vec<CharacterCard>) -> Self {
        self.characters = cards
            .into_iter()
            .map(|character| (character.id.clone(), character))
            .collect();
        self.present = present;
        self
    }

    fn world(&self) -> WorldView<'_> {
        WorldView {
            room: self.map.get_room_by_id("start"),
            map: &self.map,
            persona: &self.persona,
            characters: &self.characters,
            present: &self.present,
        }
    }

    fn world_without_room(&self) -> WorldView<'_> {
        WorldView {
            room: None,
            map: &self.map,
            persona: &self.persona,
            characters: &self.characters,
            present: &self.present,
        }
    }

    fn input<'a>(&'a self, world: &'a WorldView<'a>) -> HookInput<'a> {
        HookInput {
            text: Some(&self.text),
            story: &self.story,
            world,
        }
    }

    fn input_without_text<'a>(&'a self, world: &'a WorldView<'a>) -> HookInput<'a> {
        HookInput {
            text: None,
            story: &self.story,
            world,
        }
    }
}

fn settings() -> AgentSettings {
    AgentSettings::default()
}

// CASE quantifier-result-output: the Agent returns a Quantifier result of the Characters present and the destination.
#[test]
fn test_the_agent_returns_a_quantifier_result() {
    let turn = Turn::new();
    let model = Arc::new(ScriptedModel::new().after_narration(
        r#"{"characters_present": ["character_1"], "movement": {"type": "Entering", "destination": "room2"}}"#,
    ));
    let world = turn.world();
    let result = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("a parsed report is a result");

    assert_eq!(
        result,
        QuantifierResult {
            present: vec![CHARACTER_ID.to_string()],
            destination: Some("room2".to_string()),
        }
    );
}

// CASE quantifier-character-ids: a parsed room report names the Characters present.
#[test]
fn test_parsed_report_names_the_characters_present() {
    let turn = Turn::new();
    let model = Arc::new(
        ScriptedModel::new().after_narration(r#"{"characters_present": ["character_1"]}"#),
    );
    let world = turn.world();
    let result = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("a parsed report is a result");

    assert_eq!(result.present, vec![CHARACTER_ID.to_string()]);
}

// CASE quantifier-old-field: an answer that names the Characters present in the field `npcs_in_room` still parses.
#[test]
fn test_an_answer_with_the_old_field_name_still_parses() {
    let turn = Turn::new();
    let model = Arc::new(ScriptedModel::new().after_narration(
        r#"{"npcs_in_room": ["character_1"], "movement": {"type": "Entering", "destination": "room2"}}"#,
    ));
    let world = turn.world();
    let result = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("an answer with the old field is a result");

    assert_eq!(result.present, vec![CHARACTER_ID.to_string()]);
    assert_eq!(result.destination.as_deref(), Some("room2"));
}

// CASE quantifier-empty-report: an empty report replaces the Characters present.
#[test]
fn test_an_empty_report_replaces_the_scenes_characters() {
    let turn = Turn::new();
    let model = Arc::new(ScriptedModel::new().after_narration(r#"{"characters_present": []}"#));
    let world = turn.world();
    let result = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("a parsed report is a result");

    assert!(
        result.present.is_empty(),
        "a report that names nobody beats the scene's Characters"
    );
}

// CASE quantifier-unknown-id: an id the World does not know is dropped.
#[test]
fn test_an_unknown_character_id_is_dropped() {
    let turn = Turn::new();
    let model = Arc::new(
        ScriptedModel::new()
            .after_narration(r#"{"characters_present": ["character_1", "someone_else"]}"#),
    );
    let world = turn.world();
    let result = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("a parsed report is a result");

    assert_eq!(result.present, vec![CHARACTER_ID.to_string()]);
}

// CASE quantifier-movement: a parsed report names the room the player enters.
#[test]
fn test_parsed_report_names_the_room_the_player_enters() {
    let turn = Turn::new();
    let model = Arc::new(ScriptedModel::new().after_narration(
        r#"{"characters_present": [], "movement": {"type": "Entering", "destination": "room2"}}"#,
    ));
    let world = turn.world();
    let result = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("a parsed report is a result");

    assert_eq!(result.destination.as_deref(), Some("room2"));
}

// CASE quantifier-low-confidence: an unparseable answer falls back to the Characters present.
#[test]
fn test_an_unparseable_answer_keeps_the_scenes_characters() {
    let turn = Turn::new();
    let model = Arc::new(
        ScriptedModel::new()
            .after_narration("I cannot tell who is here.")
            .after_narration("Still no room report."),
    );
    let world = turn.world();
    let result = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("the fallback is a result");

    assert_eq!(result.present, vec![CHARACTER_ID.to_string()]);
    assert_eq!(result.destination, None);
}

// CASE quantifier-retry: a low-confidence first answer is read again, and the second answer wins.
#[test]
fn test_a_low_confidence_answer_is_read_again() {
    let turn = Turn::new();
    let model = Arc::new(
        ScriptedModel::new()
            .after_narration("no report in this answer")
            .after_narration(
                r#"{"characters_present": [], "movement": {"type": "Entering", "destination": "room2"}}"#,
            ),
    );
    let world = turn.world();
    let result = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("the second answer is a result");

    assert_eq!(model.calls().len(), 2, "the first answer is read again");
    assert_eq!(
        result,
        QuantifierResult {
            present: vec![],
            destination: Some("room2".to_string()),
        },
        "the second answer wins"
    );
}

// CASE quantifier-transport-failure: both attempts fail and the fallback keeps the Characters present.
#[test]
fn test_a_transport_failure_keeps_the_scenes_characters() {
    let turn = Turn::new();
    let model = Arc::new(
        ScriptedModel::new()
            .failed(Role::Quantifier)
            .failed(Role::Quantifier),
    );
    let world = turn.world();
    let result = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("the fallback is a result");

    assert_eq!(result.present, vec![CHARACTER_ID.to_string()]);
}

// CASE quantifier-no-text: a hook input without saved text fails.
#[test]
fn test_a_hook_input_without_text_fails() {
    let turn = Turn::new();
    let model = Arc::new(ScriptedModel::new());
    let world = turn.world();
    let result = quantifier(Arc::clone(&model)).run(&turn.input_without_text(&world), &settings());

    assert!(result.is_err(), "the Quantifier needs the saved text");
}

// CASE quantifier-no-room: a hook input without a room fails.
#[test]
fn test_a_hook_input_without_a_room_fails() {
    let turn = Turn::new();
    let model = Arc::new(ScriptedModel::new());
    let world = turn.world_without_room();
    let result = quantifier(Arc::clone(&model)).run(&turn.input(&world), &settings());

    assert!(result.is_err(), "the Quantifier needs the room it reads");
}

// CASE quantifier-preset: the settings' preset text reaches the prompt.
#[test]
fn test_the_settings_preset_reaches_the_prompt() {
    let turn = Turn::new();
    let model = Arc::new(ScriptedModel::new().after_narration(r#"{"characters_present": []}"#));
    let settings = AgentSettings {
        preset: Some("Report the Characters for {{user}}.".to_string()),
        ..Default::default()
    };
    let world = turn.world();
    quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings)
        .expect("a parsed report is an output");

    let call = model.calls().into_iter().next().expect("one call");
    assert!(
        call.system_prompt.contains(&format!(
            "Report the Characters for {}.",
            turn.persona.sheet.name
        )),
        "the preset's macros must render: {}",
        call.system_prompt
    );
}

// CASE quantifier-call-record: a good call is recorded with its agent name and prompts.
#[test]
fn test_a_good_call_is_recorded_with_its_prompts() {
    let storage = Arc::new(crate::adapters::driven::storage::Storage::new_in_memory());
    let model = Arc::new(ScriptedModel::new().after_narration(r#"{"characters_present": []}"#));
    let recorder = make_test_recorder_with_storage(model, Arc::clone(&storage));
    let agent = QuantifierAgent::from_config(&config(), recorder).expect("the quantifier builds");
    let turn = Turn::new();

    let world = turn.world();
    agent
        .run(&turn.input(&world), &settings())
        .expect("a parsed report is an output");

    let recorded = storage
        .list_latest_llm_messages(10)
        .expect("the forensics read back");
    let call = recorded
        .last()
        .expect("the call is recorded, so the panel can show it");
    assert_eq!(call.agent_name, "quantifier");
    assert!(call.error_message.is_none());
    assert!(call.user_prompt.contains("<CurrentRoom>"));
    assert!(!call.system_prompt.is_empty());
}

// CASE quantifier-call-record-failure: a failed call is recorded with its failure text.
#[test]
fn test_a_failed_call_is_recorded_with_its_failure_text() {
    let storage = Arc::new(crate::adapters::driven::storage::Storage::new_in_memory());
    let model = Arc::new(
        ScriptedModel::new()
            .failed(Role::Quantifier)
            .failed(Role::Quantifier),
    );
    let recorder = make_test_recorder_with_storage(model, Arc::clone(&storage));
    let agent = QuantifierAgent::from_config(&config(), recorder).expect("the quantifier builds");
    let turn = Turn::new();

    let world = turn.world();
    agent
        .run(&turn.input(&world), &settings())
        .expect("the fallback is an output");

    let recorded = storage
        .list_latest_llm_messages(10)
        .expect("the forensics read back");
    let failure = recorded
        .iter()
        .find(|message| message.error_message.is_some())
        .expect("the failed attempt is recorded");
    assert_eq!(failure.agent_name, "quantifier");
    assert_eq!(
        failure.error_message.as_deref(),
        Some("LLM Error: Narration generation failed at stage 'scripted': configured_failure")
    );
}

fn between<'a>(prompt: &'a str, open: &str, close: &str) -> &'a str {
    let start = prompt
        .find(open)
        .unwrap_or_else(|| panic!("the {open} block renders: {prompt}"));
    let rest = &prompt[start..];
    let end = rest
        .find(close)
        .unwrap_or_else(|| panic!("the {close} tag closes the {open} block: {prompt}"));
    &rest[..end]
}

fn order_of(block: &str, names: &[&str]) -> Vec<usize> {
    names
        .iter()
        .map(|name| {
            block
                .find(name)
                .unwrap_or_else(|| panic!("{name} renders: {block}"))
        })
        .collect()
}

// CASE quantifier-cards-in-id-order: both card lists render in Character id order, whatever order the World map and the scene hold them in.
#[test]
fn test_both_card_lists_render_in_character_id_order() {
    let cards = vec![
        TestCharacter::named("c", "Cid"),
        TestCharacter::named("a", "Ada"),
        TestCharacter::named("b", "Bea"),
    ];
    let present = vec![cards[2].clone(), cards[0].clone(), cards[1].clone()];
    let turn = Turn::new().with_cards(cards, present);
    let model = Arc::new(ScriptedModel::new().after_narration(r#"{"characters_present": []}"#));
    let world = turn.world();
    quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("a parsed report is an output");

    let call = model.calls().into_iter().next().expect("one call");
    for (prompt, open, close) in [
        (
            &call.system_prompt,
            "<available_character_ids>",
            "</available_character_ids>",
        ),
        (
            &call.user_prompt,
            "<PreviousCharactersPresent>",
            "</PreviousCharactersPresent>",
        ),
    ] {
        let positions = order_of(between(prompt, open, close), &["Ada", "Bea", "Cid"]);
        assert!(
            positions.windows(2).all(|pair| pair[0] < pair[1]),
            "{open} renders its cards in id order: {prompt}"
        );
    }
}

// CASE quantifier-card-text-renders-placeholders: a card's text reaches the prompt with its placeholders rendered.
#[test]
fn test_a_cards_text_renders_its_placeholders() {
    let mut card = TestCharacter::named(CHARACTER_ID, "Test Character");
    card.sheet.description = "A friend of {{user}}.".to_string();
    let turn = Turn::new().with_cards(vec![card.clone()], vec![card]);
    let model = Arc::new(ScriptedModel::new().after_narration(r#"{"characters_present": []}"#));
    let world = turn.world();
    quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("a parsed report is an output");

    let call = model.calls().into_iter().next().expect("one call");
    let present = between(
        &call.user_prompt,
        "<PreviousCharactersPresent>",
        "</PreviousCharactersPresent>",
    );
    assert!(
        present.contains(&format!("A friend of {}.", turn.persona.sheet.name)),
        "the card text renders its placeholders: {present}"
    );
    assert!(
        !present.contains("{{user}}"),
        "no raw placeholder reaches the model: {present}"
    );
}

// CASE quantifier-latest-message-block: the newest Message renders in `<LatestMessage>` with no name prefix, and the closing sentence names that block.
#[test]
fn test_the_newest_message_renders_in_the_latest_message_block() {
    let turn = Turn::new().with_no_history();
    let model = Arc::new(ScriptedModel::new().after_narration(r#"{"characters_present": []}"#));
    let world = turn.world();
    quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("a parsed report is an output");

    let call = model.calls().into_iter().next().expect("one call");
    assert!(
        call.user_prompt
            .contains("<LatestMessage>\n  You look around the inn.\n</LatestMessage>\n\n"),
        "the block holds the saved text and no name prefix: {}",
        call.user_prompt
    );
    assert!(
        call.user_prompt.contains("happens in <LatestMessage>"),
        "the closing sentence names the block: {}",
        call.user_prompt
    );
    assert!(
        !call.user_prompt.contains("LatestNarration"),
        "no older tag name reaches the model: {}",
        call.user_prompt
    );
}

// CASE quantifier-sends-the-preset-parameters: the call settings of the Agent reach the model call as they are.
#[test]
fn test_the_call_settings_reach_the_model_call() {
    let turn = Turn::new();
    let model = Arc::new(ScriptedModel::new().after_narration(r#"{"characters_present": []}"#));
    let call = CallSettings {
        max_tokens: 300,
        temperature: Some(0.2),
        top_p: Some(0.8),
        custom: serde_json::json!({"top_k": 20})
            .as_object()
            .unwrap()
            .clone(),
        game: Some(GameId(7)),
    };
    let settings = AgentSettings {
        call: call.clone(),
        ..Default::default()
    };
    let world = turn.world();
    quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings)
        .expect("a parsed report is an output");

    let sent = model.calls().into_iter().next().expect("one call");
    assert_eq!(sent.settings, call);
}
