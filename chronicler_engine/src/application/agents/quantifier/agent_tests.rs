use std::collections::HashMap;
use std::sync::Arc;

use crate::adapters::driven::llm::providers::MockBackend;
use crate::application::agents::Agent;
use crate::application::agents::quantifier::agent::QuantifierAgent;
use crate::domain::model::agent::{AgentContext, BackendSelector, ExecutionPhase};
use crate::domain::model::character::NpcCard;
use crate::domain::model::state::game_state::GameState;
use crate::test_support::fixtures::{TestMap, TestPersona};

#[test]
fn test_from_config_creates_agent() {
    let config = crate::domain::model::agent::AgentConfig {
        name: "quantifier".to_string(),
        agent_type: "quantifier".to_string(),
        enabled: true,
        backend: BackendSelector::UseMain,
        phase: ExecutionPhase::PostGeneration,
    };
    let agent = QuantifierAgent::from_config_with_storage(
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
fn test_with_backend() {
    let backend: Arc<dyn crate::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default());
    let agent = QuantifierAgent::with_provider("custom".to_string(), backend);
    assert_eq!(agent.name(), "custom");
}

#[test]
fn test_name() {
    let backend: Arc<dyn crate::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default());
    let agent = QuantifierAgent::with_provider("quantifier".to_string(), backend);
    assert_eq!(agent.name(), "quantifier");
}

#[test]
fn test_phase() {
    let backend: Arc<dyn crate::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default());
    let agent = QuantifierAgent::with_provider("q".to_string(), backend);
    assert_eq!(agent.phase(), ExecutionPhase::PostGeneration);
}

#[test]
fn test_backend_selector() {
    let backend: Arc<dyn crate::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default());
    let agent = QuantifierAgent::with_provider("q".to_string(), backend);
    assert_eq!(
        agent.backend_selector(),
        BackendSelector::UseNamed("quantifier".to_string())
    );
}

#[test]
fn test_execute_missing_main_response() {
    let backend: Arc<dyn crate::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default());
    let agent = QuantifierAgent::with_provider("q".to_string(), backend);

    let state = GameState::new("start");
    let map = TestMap::single_room("start");
    let persona = TestPersona::standard();
    let npcs: HashMap<String, NpcCard> = HashMap::new();

    let ctx = AgentContext {
        state: &state,
        main_response: None,
        player_input: "look",
        current_room: None,
        map: &map,
        persona: &persona,
        npcs: &npcs,
    };

    let result = agent.execute(&ctx);
    assert!(result.is_err());
}
