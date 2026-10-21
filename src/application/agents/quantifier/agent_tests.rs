//! Seam contract of the Quantifier, the after-narration hook's Agent.

use std::collections::HashMap;
use std::sync::Arc;

use crate::application::agents::quantifier::agent::QuantifierAgent;
use crate::application::agents::trait_def::AfterNarrationAgent;
use crate::domain::model::agent::{
    AgentConfig, AgentSettings, BackendSelector, HookInput, Role, WorldView,
};
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::character::NpcCard;
use crate::domain::model::map::MapDef;
use crate::domain::model::state::message_types::{MessageType, StoryRow};
use crate::test_support::fixtures::{default_story_row, TestMap, TestNpc, TestPersona};
use crate::test_support::{make_test_recorder, make_test_recorder_with_storage, ScriptedModel};

const NPC_ID: &str = "npc_1";

fn config() -> AgentConfig {
    AgentConfig {
        name: "quantifier".to_string(),
        agent_type: "quantifier".to_string(),
        enabled: true,
        backend: BackendSelector::UseNamed("quantifier".to_string()),
    }
}

fn quantifier(model: Arc<ScriptedModel>) -> QuantifierAgent {
    QuantifierAgent::from_config(&config(), make_test_recorder(model))
        .expect("the quantifier builds from its config")
}

struct Turn {
    map: MapDef,
    persona: crate::domain::model::character::PersonaCard,
    characters: HashMap<String, NpcCard>,
    present: Vec<NpcCard>,
    story: Vec<StoryRow>,
    text: String,
}

impl Turn {
    fn new() -> Self {
        let map = TestMap::single_room("start");
        let persona = TestPersona::standard();
        let npcs = vec![TestNpc::named(NPC_ID, "Test NPC")];
        let characters = npcs
            .iter()
            .map(|npc| (npc.id.clone(), npc.clone()))
            .collect();
        Self {
            map,
            persona,
            characters,
            present: npcs.clone(),
            story: vec![StoryRow {
                message_type: MessageType::Narration,
                text: "The innkeeper glances your way.".to_string(),
                ..default_story_row(MessageType::Narration)
            }],
            text: "You look around the inn.".to_string(),
        }
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
            player_text: "look",
            story: &self.story,
            world,
        }
    }

    fn input_without_text<'a>(&'a self, world: &'a WorldView<'a>) -> HookInput<'a> {
        HookInput {
            text: None,
            player_text: "look",
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
        r#"{"npcs_in_room": ["npc_1"], "movement": {"type": "Entering", "destination": "room2"}}"#,
    ));
    let world = turn.world();
    let result = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("a parsed report is a result");

    assert_eq!(
        result,
        QuantifierResult {
            present: vec![NPC_ID.to_string()],
            destination: Some("room2".to_string()),
        }
    );
}

// CASE quantifier-npc-ids: a parsed room report names the Characters present.
#[test]
fn test_parsed_report_names_the_characters_present() {
    let turn = Turn::new();
    let model = Arc::new(ScriptedModel::new().after_narration(r#"{"npcs_in_room": ["npc_1"]}"#));
    let world = turn.world();
    let result = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("a parsed report is a result");

    assert_eq!(result.present, vec![NPC_ID.to_string()]);
}

// CASE quantifier-empty-report: an empty report replaces the scene's Characters.
#[test]
fn test_an_empty_report_replaces_the_scenes_characters() {
    let turn = Turn::new();
    let model = Arc::new(ScriptedModel::new().after_narration(r#"{"npcs_in_room": []}"#));
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
        ScriptedModel::new().after_narration(r#"{"npcs_in_room": ["npc_1", "someone_else"]}"#),
    );
    let world = turn.world();
    let result = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("a parsed report is a result");

    assert_eq!(result.present, vec![NPC_ID.to_string()]);
}

// CASE quantifier-movement: a parsed report names the room the player enters.
#[test]
fn test_parsed_report_names_the_room_the_player_enters() {
    let turn = Turn::new();
    let model = Arc::new(ScriptedModel::new().after_narration(
        r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#,
    ));
    let world = turn.world();
    let result = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("a parsed report is a result");

    assert_eq!(result.destination.as_deref(), Some("room2"));
}

// CASE quantifier-low-confidence: an unparseable answer falls back to the scene's Characters.
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

    assert_eq!(result.present, vec![NPC_ID.to_string()]);
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
                r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#,
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

// CASE quantifier-transport-failure: both attempts fail and the fallback keeps the scene's Characters.
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

    assert_eq!(result.present, vec![NPC_ID.to_string()]);
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
    let model = Arc::new(ScriptedModel::new().after_narration(r#"{"npcs_in_room": []}"#));
    let settings = AgentSettings {
        preset: Some("Report the Characters for {{user}}.".to_string()),
        max_tokens: None,
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
    let model = Arc::new(ScriptedModel::new().after_narration(r#"{"npcs_in_room": []}"#));
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
}
