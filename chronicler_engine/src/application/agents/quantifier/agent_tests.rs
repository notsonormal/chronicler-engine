use std::sync::Arc;

use crate::domain::model::agent::{AgentContext, BackendSelector, ExecutionPhase};
use crate::domain::model::state::game_state::GameState;
use crate::application::agents::Agent;
use crate::application::agents::quantifier::agent::QuantifierAgent;
use crate::adapters::driven::llm::providers::MockBackend;
use crate::test_support::fixtures::{TestMap, TestPlayer, TestWorld};

#[test]
fn test_from_config_creates_agent() {
    let config = crate::domain::model::agent::AgentConfig {
        name: "quantifier".to_string(),
        agent_type: "quantifier".to_string(),
        enabled: true,
        backend: BackendSelector::UseMain,
        phase: ExecutionPhase::PostGeneration,
    };
    let agent = QuantifierAgent::from_config(&config);
    assert!(agent.is_ok());
}

#[test]
fn test_with_backend() {
    let backend: Arc<dyn crate::application::ports::llm_provider::LlmBackend> =
        Arc::new(MockBackend::default());
    let agent = QuantifierAgent::with_backend("custom".to_string(), backend);
    assert_eq!(agent.name(), "custom");
}

#[test]
fn test_name() {
    let backend: Arc<dyn crate::application::ports::llm_provider::LlmBackend> =
        Arc::new(MockBackend::default());
    let agent = QuantifierAgent::with_backend("quantifier".to_string(), backend);
    assert_eq!(agent.name(), "quantifier");
}

#[test]
fn test_phase() {
    let backend: Arc<dyn crate::application::ports::llm_provider::LlmBackend> =
        Arc::new(MockBackend::default());
    let agent = QuantifierAgent::with_backend("q".to_string(), backend);
    assert_eq!(agent.phase(), ExecutionPhase::PostGeneration);
}

#[test]
fn test_backend_selector() {
    let backend: Arc<dyn crate::application::ports::llm_provider::LlmBackend> =
        Arc::new(MockBackend::default());
    let agent = QuantifierAgent::with_backend("q".to_string(), backend);
    assert_eq!(
        agent.backend_selector(),
        BackendSelector::UseNamed("quantifier".to_string())
    );
}

#[test]
fn test_execute_missing_main_response() {
    let backend: Arc<dyn crate::application::ports::llm_provider::LlmBackend> =
        Arc::new(MockBackend::default());
    let agent = QuantifierAgent::with_backend("q".to_string(), backend);

    let state = GameState::new(
        Arc::new(TestWorld::minimal()),
        Arc::new(TestMap::single_room("start")),
        Arc::new(TestPlayer::standard()),
        vec![],
        "start".to_string(),
    );

    let ctx = AgentContext {
        state: &state,
        main_response: None,
        player_input: "look",
        current_room: None,
    };

    let result = agent.execute(&ctx);
    assert!(result.is_err());
}
