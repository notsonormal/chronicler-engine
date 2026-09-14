//! Tests for the OptionsAgent trait surface and execute behavior.

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};

use crate::adapters::driven::llm::providers::MockBackend;
use crate::adapters::driven::storage::Storage;
use crate::application::agents::options::agent::OptionsAgent;
use crate::application::agents::Agent;
use crate::domain::model::agent::{AgentConfig, AgentContext, BackendSelector, ExecutionPhase};
use crate::domain::model::character::NpcCard;
use crate::domain::model::prompt_preset::{PromptPreset, PresetType};
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::game_state::GameState;
use crate::test_support::fixtures::{TestMap, TestPersona};
use crate::test_support::make_test_recorder_with_storage;

fn make_agent(
    provider: Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
) -> OptionsAgent {
    OptionsAgent::with_provider("options".to_string(), provider)
}

fn make_ctx<'a>(
    state: &'a GameState,
    map: &'a crate::domain::model::map::MapDef,
    persona: &'a crate::domain::model::character::PersonaCard,
    npcs: &'a HashMap<String, NpcCard>,
) -> AgentContext<'a> {
    AgentContext {
        state,
        main_response: None,
        player_input: "",
        current_room: map.get_room_by_id("start"),
        map,
        persona,
        npcs,
    }
}

#[test]
fn test_from_config_creates_agent() {
    let config = crate::domain::model::agent::AgentConfig {
        name: "options".to_string(),
        agent_type: "options".to_string(),
        enabled: true,
        backend: BackendSelector::UseNamed("options".to_string()),
        phase: ExecutionPhase::OptionsGeneration,
    };
    let agent = OptionsAgent::from_config_with_storage(
        &config,
        crate::test_support::make_test_recorder(Arc::new(MockBackend::default())),
        None,
        Arc::new(std::sync::RwLock::new(
            crate::domain::model::settings::AppSettings::default(),
        )),
    );
    assert!(agent.is_ok());
}

#[test]
fn test_trait_surface() {
    let agent = make_agent(Arc::new(MockBackend::default()));
    assert_eq!(agent.name(), "options");
    assert_eq!(agent.phase(), ExecutionPhase::OptionsGeneration);
    assert_eq!(
        agent.backend_selector(),
        BackendSelector::UseNamed("options".to_string())
    );
}

#[test]
fn test_execute_parses_tagged_response() {
    let provider: Arc<dyn crate::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default().with_prompt_responses(vec![
            "<suggestion>Search the desk</suggestion><suggestion>Question the guard</suggestion>"
                .to_string(),
        ]));
    let agent = make_agent(provider);

    let state = GameState::new("start");
    let map = TestMap::single_room("start");
    let persona = TestPersona::standard();
    let npcs: HashMap<String, NpcCard> = HashMap::new();
    let ctx = make_ctx(&state, &map, &persona, &npcs);

    match agent.execute(&ctx) {
        Ok(crate::domain::model::agent::AgentResult::Options(options)) => {
            assert_eq!(options, vec!["Search the desk", "Question the guard"]);
        }
        other => panic!("expected Options result, got {other:?}"),
    }
}

#[test]
fn test_execute_unparseable_response_errors_after_attempts() {
    // A cycling unparseable response: "[Continuation: ...]" (the default
    // mock branch) never increments call_index, so the retry pin needs the
    // per-call response list. The agent must surface an error, not an empty
    // set, after exactly the contract's two attempts.
    let backend = Arc::new(
        MockBackend::new()
            .with_prompt_responses(vec!["The model rambled about the weather.".to_string()]),
    );
    let agent = make_agent(Arc::clone(&backend) as Arc<_>);

    let state = GameState::new("start");
    let map = TestMap::single_room("start");
    let persona = TestPersona::standard();
    let npcs: HashMap<String, NpcCard> = HashMap::new();
    let ctx = make_ctx(&state, &map, &persona, &npcs);

    assert!(agent.execute(&ctx).is_err());
    assert_eq!(
        backend.call_index.load(Ordering::SeqCst),
        2,
        "unparseable output must be retried once before erroring"
    );
}

#[test]
fn test_execute_recovers_when_second_attempt_parses() {
    // Attempt 1 unparseable, attempt 2 parseable — the retry loop must
    // recover instead of failing the turn.
    let backend = MockBackend::new().with_prompt_responses(vec![
        "The model rambled about the weather.".to_string(),
        "<suggestion>Search the desk</suggestion>".to_string(),
    ]);
    let agent = make_agent(Arc::new(backend));

    let state = GameState::new("start");
    let map = TestMap::single_room("start");
    let persona = TestPersona::standard();
    let npcs: HashMap<String, NpcCard> = HashMap::new();
    let ctx = make_ctx(&state, &map, &persona, &npcs);

    match agent.execute(&ctx) {
        Ok(crate::domain::model::agent::AgentResult::Options(options)) => {
            assert_eq!(options, vec!["Search the desk"]);
        }
        other => panic!("expected the retry to recover, got {other:?}"),
    }
}

#[test]
fn test_execute_transport_failure_errors() {
    let agent = make_agent(Arc::new(MockBackend::default().with_fail()));

    let state = GameState::new("start");
    let map = TestMap::single_room("start");
    let persona = TestPersona::standard();
    let npcs: HashMap<String, NpcCard> = HashMap::new();
    let ctx = make_ctx(&state, &map, &persona, &npcs);

    assert!(agent.execute(&ctx).is_err());
}

#[test]
fn test_execute_missing_current_room_errors() {
    let agent = make_agent(Arc::new(MockBackend::default()));

    let state = GameState::new("nowhere");
    let map = TestMap::single_room("start");
    let persona = TestPersona::standard();
    let npcs: HashMap<String, NpcCard> = HashMap::new();
    let ctx = AgentContext {
        state: &state,
        main_response: None,
        player_input: "",
        current_room: None,
        map: &map,
        persona: &persona,
        npcs: &npcs,
    };

    assert!(agent.execute(&ctx).is_err());
}

#[test]
fn test_execute_resolves_active_options_preset_and_records_prompts() {
    // No game row exists, so the agent's preset lookup falls back to the
    // settings id — the same fallback Storage::active_options_preset_id pins.
    let storage = Arc::new(Storage::new_in_memory());
    storage
        .save_preset(&PromptPreset {
            id: "options_test_preset".to_string(),
            name: "Options Test".to_string(),
            instructions: Some("Offer {{option_count}} bold moves for {{user}}.".to_string()),
            preset_type: PresetType::Options,
            ..PromptPreset::default()
        })
        .expect("save_preset should succeed");

    let settings = AppSettings {
        active_options_prompt_preset_id: "options_test_preset".to_string(),
        ..AppSettings::default()
    };
    let config = AgentConfig {
        name: "options".to_string(),
        agent_type: "options".to_string(),
        enabled: true,
        backend: BackendSelector::UseNamed("options".to_string()),
        phase: ExecutionPhase::OptionsGeneration,
    };
    let provider = Arc::new(
        MockBackend::new()
            .with_prompt_responses(vec!["<suggestion>Search the desk</suggestion>".to_string()]),
    );
    let agent = OptionsAgent::from_config_with_storage(
        &config,
        make_test_recorder_with_storage(provider, Arc::clone(&storage)),
        Some(Arc::clone(&storage)),
        Arc::new(RwLock::new(settings)),
    )
    .expect("agent construction should succeed");

    let state = GameState::new("start");
    let map = TestMap::single_room("start");
    let persona = TestPersona::standard();
    let npcs: HashMap<String, NpcCard> = HashMap::new();
    let ctx = make_ctx(&state, &map, &persona, &npcs);

    match agent.execute(&ctx) {
        Ok(crate::domain::model::agent::AgentResult::Options(options)) => {
            assert_eq!(options, vec!["Search the desk"]);
        }
        other => panic!("expected Options result, got {other:?}"),
    }

    // The recorded forensics must show the preset override rendered through
    // the template vars — not the fallback prompt and not raw macros.
    let recorded = storage
        .list_latest_llm_messages(10)
        .expect("llm message listing should succeed");
    let options_call = recorded
        .iter()
        .find(|message| message.agent_name == "options")
        .expect("options LLM call must be recorded");
    let expected_system = format!("Offer 3 bold moves for {}.", persona.sheet.name);
    assert!(
        options_call.system_prompt.contains(&expected_system),
        "the preset override must be rendered with template vars: {}",
        options_call.system_prompt
    );
    assert!(!options_call.system_prompt.contains("{{"));
    assert!(!options_call.system_prompt.contains("{{"));
    assert!(options_call.user_prompt.contains("<CurrentRoom>"));
}
