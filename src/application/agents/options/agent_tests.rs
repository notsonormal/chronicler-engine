//! Seam contract of the Options Agent, the Agent of the end-of-Job hook.

use std::collections::HashMap;
use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::application::agents::options::agent::OptionsGeneratorAgent;
use crate::application::agents::trait_def::EndOfJobAgent;
use crate::domain::model::agent::{
    AgentConfig, AgentSettings, BackendSelector, HookInput, Role, WorldView,
};
use crate::domain::model::character::CharacterCard;
use crate::domain::model::state::message_types::{MessageType, StoryRow};
use crate::test_support::fixtures::{default_story_row, TestMap, TestCharacter, TestPersona};
use crate::test_support::{make_test_recorder, make_test_recorder_with_storage, ScriptedModel};

const SET_A: &str = "<suggestion>Search the desk</suggestion>\
<suggestion>Question the guard</suggestion><suggestion>Leave the hall</suggestion>";

fn config() -> AgentConfig {
    AgentConfig {
        name: "options".to_string(),
        agent_type: "options".to_string(),
        enabled: true,
        backend: BackendSelector::UseNamed("options".to_string()),
    }
}

fn options_agent(model: Arc<ScriptedModel>) -> OptionsGeneratorAgent {
    OptionsGeneratorAgent::from_config(&config(), make_test_recorder(model))
        .expect("the options agent builds from its config")
}

struct Scene {
    map: crate::domain::model::map::MapDef,
    persona: crate::domain::model::character::PersonaCard,
    characters: HashMap<String, CharacterCard>,
    present: Vec<CharacterCard>,
    story: Vec<StoryRow>,
}

impl Scene {
    fn new() -> Self {
        let map = TestMap::single_room("start");
        let persona = TestPersona::standard();
        let present = vec![TestCharacter::named("character_1", "Test Character")];
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
            text: Some("You look around the inn."),
            story: &self.story,
            world,
        }
    }
}

// CASE options-set: a parsed answer is the offered set.
#[test]
fn test_a_parsed_answer_is_the_offered_set() {
    let scene = Scene::new();
    let model = Arc::new(ScriptedModel::new().options(SET_A));
    let world = scene.world();
    let offered = options_agent(Arc::clone(&model))
        .run(&scene.input(&world), &AgentSettings::default())
        .expect("a parsed answer is an offered set");

    assert_eq!(
        offered.0,
        vec!["Search the desk", "Question the guard", "Leave the hall"]
    );
}

// CASE options-retry: an answer with no parseable option is read again, and the second answer wins.
#[test]
fn test_an_answer_with_no_option_is_read_again() {
    let scene = Scene::new();
    let model = Arc::new(
        ScriptedModel::new()
            .options("The model rambled about the weather.")
            .options("<suggestion>Search the desk</suggestion>"),
    );
    let world = scene.world();
    let offered = options_agent(Arc::clone(&model))
        .run(&scene.input(&world), &AgentSettings::default())
        .expect("the second answer is an offered set");

    assert_eq!(offered.0, vec!["Search the desk"]);
}

// CASE options-transport-failure: both attempts fail and the call fails.
#[test]
fn test_a_transport_failure_fails_the_call() {
    let scene = Scene::new();
    let model = Arc::new(
        ScriptedModel::new()
            .failed(Role::Options)
            .failed(Role::Options),
    );
    let world = scene.world();
    let result =
        options_agent(Arc::clone(&model)).run(&scene.input(&world), &AgentSettings::default());

    assert!(result.is_err(), "a failed call has no offered set");
}

// CASE options-no-room: a hook input without a room fails.
#[test]
fn test_a_hook_input_without_a_room_fails() {
    let scene = Scene::new();
    let model = Arc::new(ScriptedModel::new());
    let world = scene.world_without_room();
    let result =
        options_agent(Arc::clone(&model)).run(&scene.input(&world), &AgentSettings::default());

    assert!(result.is_err(), "the options Agent needs the room it reads");
}

// CASE options-preset: the settings' preset text reaches the prompt.
#[test]
fn test_the_settings_preset_reaches_the_prompt() {
    let scene = Scene::new();
    let model = Arc::new(ScriptedModel::new().options(SET_A));
    let settings = AgentSettings {
        preset: Some("Offer {{option_count}} bold moves for {{user}}.".to_string()),
        max_tokens: None,
    };
    let world = scene.world();
    options_agent(Arc::clone(&model))
        .run(&scene.input(&world), &settings)
        .expect("a parsed answer is an offered set");

    let call = model.calls().into_iter().next().expect("one call");
    assert!(
        call.system_prompt.contains(&format!(
            "Offer 3 bold moves for {}.",
            scene.persona.sheet.name
        )),
        "the preset's macros must render: {}",
        call.system_prompt
    );
}

// CASE options-fallback-prompt: without a preset the fallback prompt renders its macros.
#[test]
fn test_without_a_preset_the_fallback_prompt_renders_its_macros() {
    let scene = Scene::new();
    let model = Arc::new(ScriptedModel::new().options(SET_A));
    let world = scene.world();
    options_agent(Arc::clone(&model))
        .run(&scene.input(&world), &AgentSettings::default())
        .expect("a parsed answer is an offered set");

    let call = model.calls().into_iter().next().expect("one call");
    assert!(
        call.system_prompt
            .contains("options generator for a text adventure"),
        "the fallback prompt must name what it is: {}",
        call.system_prompt
    );
    assert!(
        !call.system_prompt.contains("{{"),
        "no macro may reach the model: {}",
        call.system_prompt
    );
}

// CASE options-call-record: a good call is recorded with its agent name and prompts.
#[test]
fn test_a_good_call_is_recorded_with_its_prompts() {
    let storage = Arc::new(Storage::new_in_memory());
    let model = Arc::new(ScriptedModel::new().options(SET_A));
    let recorder = make_test_recorder_with_storage(model, Arc::clone(&storage));
    let agent =
        OptionsGeneratorAgent::from_config(&config(), recorder).expect("the options agent builds");
    let scene = Scene::new();

    let world = scene.world();
    agent
        .run(&scene.input(&world), &AgentSettings::default())
        .expect("a parsed answer is an offered set");

    let recorded = storage
        .list_latest_llm_messages(10)
        .expect("the forensics read back");
    let call = recorded.last().expect("the call is recorded");
    assert_eq!(call.agent_name, "options");
    assert!(call.error_message.is_none());
    assert!(call.user_prompt.contains("<CurrentRoom>"));
}

// CASE options-call-record-failure: a failed call is recorded with its failure text.
#[test]
fn test_a_failed_call_is_recorded_with_its_failure_text() {
    let storage = Arc::new(Storage::new_in_memory());
    let model = Arc::new(
        ScriptedModel::new()
            .failed(Role::Options)
            .failed(Role::Options),
    );
    let recorder = make_test_recorder_with_storage(model, Arc::clone(&storage));
    let agent =
        OptionsGeneratorAgent::from_config(&config(), recorder).expect("the options agent builds");
    let scene = Scene::new();

    let world = scene.world();
    let _ = agent.run(&scene.input(&world), &AgentSettings::default());

    let recorded = storage
        .list_latest_llm_messages(10)
        .expect("the forensics read back");
    let failure = recorded
        .iter()
        .find(|message| message.error_message.is_some())
        .expect("the failed attempt is recorded");
    assert_eq!(failure.agent_name, "options");
    assert_eq!(
        failure.error_message.as_deref(),
        Some("LLM Error: Narration generation failed at stage 'scripted': configured_failure")
    );
}
