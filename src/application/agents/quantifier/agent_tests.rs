//! Seam contract of the Quantifier, the analysis hook's Agent.
//!
//! CASE quantifier-npc-ids: a parsed room report names the Characters present.
//! CASE quantifier-unknown-id: an id the World does not know is dropped.
//! CASE quantifier-movement: a parsed report names the room the player enters.
//! CASE quantifier-low-confidence: an unparseable answer falls back to the scene's Characters, with Low confidence.
//!
//! CASE quantifier-retry: a low-confidence first answer is read again, and the second answer wins.
//! CASE quantifier-transport-failure: both attempts fail and the fallback keeps the scene's Characters.
//! CASE quantifier-no-text: a hook input without saved text fails.
//! CASE quantifier-no-room: a hook input without a room fails.
//!
//! CASE quantifier-preset: the settings' preset text reaches the prompt.
//! CASE quantifier-call-record: a good call is recorded with its agent name and prompts.
//! CASE quantifier-call-record-failure: a failed call is recorded with its failure text.

use std::collections::HashMap;
use std::sync::Arc;

use crate::application::agents::quantifier::agent::QuantifierAgent;
use crate::application::agents::trait_def::AnalysisAgent;
use crate::domain::model::agent::{
    AgentConfig, AgentSettings, BackendSelector, Confidence, Hook, HookInput, Kind, StoryEntry,
    WorldView,
};
use crate::domain::model::character::NpcCard;
use crate::domain::model::map::MapDef;
use crate::domain::model::state::message_types::MessageType;
use crate::test_support::fixtures::{TestMap, TestNpc, TestPersona};
use crate::test_support::{
    make_test_recorder, make_test_recorder_with_storage, ModelRole, ScriptedModel,
};

const NPC_ID: &str = "npc_1";

fn config() -> AgentConfig {
    AgentConfig {
        name: "quantifier".to_string(),
        agent_type: "quantifier".to_string(),
        enabled: true,
        backend: BackendSelector::UseNamed("quantifier".to_string()),
        hook: Hook::Analysis,
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
    story: Vec<StoryEntry>,
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
            story: vec![StoryEntry {
                message_type: MessageType::Narration,
                text: "The innkeeper glances your way.".to_string(),
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
            kind: Kind::Narration,
            text: Some(&self.text),
            player_text: "look",
            story: &self.story,
            world,
            event: None,
        }
    }

    fn input_without_text<'a>(&'a self, world: &'a WorldView<'a>) -> HookInput<'a> {
        HookInput {
            kind: Kind::Narration,
            text: None,
            player_text: "look",
            story: &self.story,
            world,
            event: None,
        }
    }
}

fn settings() -> AgentSettings {
    AgentSettings::default()
}

// CASE: quantifier-npc-ids
#[test]
fn test_parsed_report_names_the_characters_present() {
    let turn = Turn::new();
    let model = Arc::new(ScriptedModel::new().analysis(r#"{"npcs_in_room": ["npc_1"]}"#));
    let world = turn.world();
    let output = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("a parsed report is an output");

    assert_eq!(output.0.npc_ids, vec![NPC_ID.to_string()]);
    assert_eq!(output.0.confidence, Confidence::High);
}

// CASE: quantifier-unknown-id
#[test]
fn test_an_unknown_character_id_is_dropped() {
    let turn = Turn::new();
    let model =
        Arc::new(ScriptedModel::new().analysis(r#"{"npcs_in_room": ["npc_1", "someone_else"]}"#));
    let world = turn.world();
    let output = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("a parsed report is an output");

    assert_eq!(output.0.npc_ids, vec![NPC_ID.to_string()]);
}

// CASE: quantifier-movement
#[test]
fn test_parsed_report_names_the_room_the_player_enters() {
    let turn = Turn::new();
    let model = Arc::new(ScriptedModel::new().analysis(
        r#"{"npcs_in_room": [], "movement": {"type": "Entering", "destination": "room2"}}"#,
    ));
    let world = turn.world();
    let output = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("a parsed report is an output");

    assert_eq!(output.0.movement_destination.as_deref(), Some("room2"));
}

// CASE: quantifier-low-confidence
#[test]
fn test_an_unparseable_answer_keeps_the_scenes_characters() {
    let turn = Turn::new();
    let model = Arc::new(
        ScriptedModel::new()
            .analysis("I cannot tell who is here.")
            .analysis("Still no room report."),
    );
    let world = turn.world();
    let output = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("the fallback is an output");

    assert_eq!(output.0.npc_ids, vec![NPC_ID.to_string()]);
    assert_eq!(output.0.confidence, Confidence::Low);
}

// CASE: quantifier-retry
#[test]
fn test_a_low_confidence_answer_is_read_again() {
    let turn = Turn::new();
    let model = Arc::new(
        ScriptedModel::new()
            .analysis("no report in this answer")
            .analysis(r#"{"npcs_in_room": ["npc_1"]}"#),
    );
    let world = turn.world();
    let output = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("the second answer is an output");

    assert_eq!(output.0.confidence, Confidence::High);
}

// CASE: quantifier-transport-failure
#[test]
fn test_a_transport_failure_keeps_the_scenes_characters() {
    let turn = Turn::new();
    let model = Arc::new(
        ScriptedModel::new()
            .failed(ModelRole::Analysis)
            .failed(ModelRole::Analysis),
    );
    let world = turn.world();
    let output = quantifier(Arc::clone(&model))
        .run(&turn.input(&world), &settings())
        .expect("the fallback is an output");

    assert_eq!(output.0.npc_ids, vec![NPC_ID.to_string()]);
    assert_eq!(output.0.confidence, Confidence::Low);
}

// CASE: quantifier-no-text
#[test]
fn test_a_hook_input_without_text_fails() {
    let turn = Turn::new();
    let model = Arc::new(ScriptedModel::new());
    let world = turn.world();
    let result = quantifier(Arc::clone(&model)).run(&turn.input_without_text(&world), &settings());

    assert!(result.is_err(), "the analysis needs the saved text");
}

// CASE: quantifier-no-room
#[test]
fn test_a_hook_input_without_a_room_fails() {
    let turn = Turn::new();
    let model = Arc::new(ScriptedModel::new());
    let world = turn.world_without_room();
    let result = quantifier(Arc::clone(&model)).run(&turn.input(&world), &settings());

    assert!(result.is_err(), "the analysis needs the room it reads");
}

// CASE: quantifier-preset
#[test]
fn test_the_settings_preset_reaches_the_prompt() {
    let turn = Turn::new();
    let model = Arc::new(ScriptedModel::new().analysis(r#"{"npcs_in_room": []}"#));
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

// CASE: quantifier-call-record
#[test]
fn test_a_good_call_is_recorded_with_its_prompts() {
    let storage = Arc::new(crate::adapters::driven::storage::Storage::new_in_memory());
    let model = Arc::new(ScriptedModel::new().analysis(r#"{"npcs_in_room": []}"#));
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

// CASE: quantifier-call-record-failure
#[test]
fn test_a_failed_call_is_recorded_with_its_failure_text() {
    let storage = Arc::new(crate::adapters::driven::storage::Storage::new_in_memory());
    let model = Arc::new(
        ScriptedModel::new()
            .failed(ModelRole::Analysis)
            .failed(ModelRole::Analysis),
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
