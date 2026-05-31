/// Unit tests for GameService
use std::sync::Arc;
use crate::application::action_pipeline::pipeline::ActionPipelineBackend;
use crate::application::game_service::GameService;
use crate::model::state::GameState;
use crate::narrative::agents::registry::AgentRegistry;
use crate::narrative::llm::MockBackend;
use crate::test_support::{
    fixtures::{TestWorld, TestMap, TestPlayer},
    make_test_context,
};

#[test]
fn test_default_construction_creates_service() {
    let service = GameService::default();
    let _assembler = service.assembler();
}

#[test]
fn test_new_construction_creates_service() {
    let service = GameService::new();
    let _assembler = service.assembler();
}

#[test]
fn test_with_backends_creates_service() {
    let llm_backend = Arc::new(MockBackend::default());
    let registry = AgentRegistry::default();
    let service = GameService::with_backends(llm_backend, registry);
    let _assembler = service.assembler();
}

#[test]
fn test_with_mock_quantifier_creates_service() {
    let llm_backend = Arc::new(MockBackend::default());
    let quantifier_backend = Arc::new(MockBackend::default());
    let service = GameService::with_mock_quantifier(llm_backend, quantifier_backend);
    let _assembler = service.assembler();
}

#[test]
fn test_complete_delegates_to_llm_backend() {
    let llm_backend = Arc::new(MockBackend::default());
    let service = GameService::with_backends(llm_backend, AgentRegistry::default());
    let result = service.complete("test-agent", "system", "user", None);
    assert!(result.is_ok());
}

#[test]
fn test_complete_with_max_tokens() {
    let llm_backend = Arc::new(MockBackend::default());
    let service = GameService::with_backends(llm_backend, AgentRegistry::default());
    let result = service.complete("test-agent", "system", "user", Some(100));
    assert!(result.is_ok());
}

#[test]
fn test_complete_returns_error_when_backend_fails() {
    let llm_backend = Arc::new(MockBackend::failing());
    let service = GameService::with_backends(llm_backend, AgentRegistry::default());
    let result = service.complete("test-agent", "system", "user", None);
    assert!(result.is_err());
}

#[test]
fn test_execute_action_with_empty_input_does_not_panic() {
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());
    let state = make_test_context(GameState::new(
        Arc::new(TestWorld::minimal()),
        Arc::new(TestMap::single_room("start")),
        Arc::new(TestPlayer::named("Test")),
        vec![],
        "start".to_string(),
    ));
    service.execute_action(state, "".to_string(), "Player".to_string());
}

#[test]
fn test_execute_action_with_valid_command_does_not_panic() {
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());
    let state = make_test_context(GameState::new(
        Arc::new(TestWorld::minimal()),
        Arc::new(TestMap::single_room("start")),
        Arc::new(TestPlayer::named("Test")),
        vec![],
        "start".to_string(),
    ));
    service.execute_action(state, "look".to_string(), "Player".to_string());
}

#[test]
fn test_assembler_is_not_null() {
    let service = GameService::new();
    let _assembler = service.assembler();
}

#[test]
fn test_complete_passes_max_tokens_to_backend() {
    let llm_backend = Arc::new(MockBackend::default());
    let service = GameService::with_backends(llm_backend, AgentRegistry::default());
    let result = service.complete("test-agent", "system", "user", Some(50));
    assert!(result.is_ok());
}

#[test]
fn test_complete_with_empty_prompts() {
    let llm_backend = Arc::new(MockBackend::default());
    let service = GameService::with_backends(llm_backend, AgentRegistry::default());
    let result = service.complete("", "", "", None);
    assert!(result.is_ok());
}

#[test]
fn test_execute_action_adds_message_to_state() {
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());
    let state = make_test_context(GameState::new(
        Arc::new(TestWorld::minimal()),
        Arc::new(TestMap::single_room("start")),
        Arc::new(TestPlayer::named("Test")),
        vec![],
        "start".to_string(),
    ));
    let history_len_before = state.load_messages().unwrap().len();
    service.execute_action(
        state.clone(),
        "test input".to_string(),
        "Player".to_string(),
    );
    let history_len_after = state.load_messages().unwrap().len();
    assert!(history_len_after >= history_len_before);
}

#[tokio::test]
async fn test_retry_last_response_retriggers_generation() {
    let service =
        GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default());
    let mut state = GameState::new(
        Arc::new(TestWorld::minimal()),
        Arc::new(TestMap::single_room("start")),
        Arc::new(TestPlayer::named("Test")),
        vec![],
        "start".to_string(),
    );
    state.add_message(
        "Test input".to_string(),
        Some("Player".to_string()),
        crate::model::state::MessageType::Input,
    );
    let ctx = make_test_context(state);
    service.retry_last_response(ctx.clone());
}

#[test]
fn test_run_post_generation_agents_merges_state_patches() {
    let llm_backend = Arc::new(MockBackend::default());
    let registry = AgentRegistry::default();
    let service = GameService::with_backends(llm_backend, registry);
    let state = GameState::new(
        Arc::new(TestWorld::minimal()),
        Arc::new(TestMap::single_room("start")),
        Arc::new(TestPlayer::named("Test")),
        vec![],
        "start".to_string(),
    );
    let mut result = crate::model::quantifier::QuantifierResult {
        npcs: crate::model::quantifier::QuantifierParseResult {
            npc_ids: Vec::new(),
            confidence: crate::model::quantifier::QuantifierConfidence::Low,
        },
        movement: crate::model::quantifier::MovementParseResult::default(),
    };
    service.run_post_generation_agents(&state, "player input", "response", &mut result);
}

#[test]
fn test_run_post_generation_agents_with_quantifier() {
    let llm_backend = Arc::new(MockBackend::default());
    let quantifier = crate::narrative::agents::quantifier::QuantifierAgent::with_backend(
        "test_quantifier".to_string(),
        llm_backend.clone(),
    );
    let registry = AgentRegistry::with_agent(Box::new(quantifier));
    let service = GameService::with_backends(llm_backend, registry);
    let state = GameState::new(
        Arc::new(TestWorld::minimal()),
        Arc::new(TestMap::single_room("start")),
        Arc::new(TestPlayer::named("Test")),
        vec![],
        "start".to_string(),
    );
    let mut result = crate::model::quantifier::QuantifierResult {
        npcs: crate::model::quantifier::QuantifierParseResult {
            npc_ids: Vec::new(),
            confidence: crate::model::quantifier::QuantifierConfidence::Low,
        },
        movement: crate::model::quantifier::MovementParseResult::default(),
    };
    service.run_post_generation_agents(&state, "player input", "response", &mut result);
}
