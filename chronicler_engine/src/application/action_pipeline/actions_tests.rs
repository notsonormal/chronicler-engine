use std::sync::Arc;

use crate::application::action_pipeline::execute_action_impl;
use crate::application::action_pipeline::pipeline::{
    ActionPipelineBackend, default_quantifier_result,
};
use crate::application::context::GameServiceContext;
use crate::error::EngineError;
use crate::model::quantifier::QuantifierResult;
use crate::model::state::{GameState, GenerationPhase, GenerationStatus, MessageType};
use crate::narrative::llm::backend::{AGENT_NARRATOR, LlmCallResult};
use crate::narrative::prompt::{LayeredPromptAssembler, PromptAssembler};
use crate::test_support::fixtures::{TestMap, TestNpc, TestPlayer, TestWorld};
use crate::test_support::make_test_context;

struct MockBackend {
    narrate_result: Result<String, EngineError>,
    complete_result: Result<String, EngineError>,
    quantifier_result: QuantifierResult,
}

impl Default for MockBackend {
    fn default() -> Self {
        Self {
            narrate_result: Ok("You look around the room.".to_string()),
            complete_result: Ok("The orb glows brighter.".to_string()),
            quantifier_result: default_quantifier_result(&[]),
        }
    }
}

impl ActionPipelineBackend for MockBackend {
    fn assembler(&self) -> &dyn PromptAssembler {
        static ASSEMBLER: std::sync::OnceLock<LayeredPromptAssembler> = std::sync::OnceLock::new();
        ASSEMBLER.get_or_init(|| {
            LayeredPromptAssembler::new(crate::narrative::prompt::budget::MAX_CONTEXT_TOKENS)
        })
    }

    fn complete(
        &self,
        agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        let result = if agent_name == AGENT_NARRATOR {
            &self.narrate_result
        } else {
            &self.complete_result
        };
        match result {
            Ok(text) => Ok(LlmCallResult {
                text: text.clone(),
                system_prompt: String::new(),
                user_prompt: String::new(),
                raw_request_json: String::new(),
                raw_response_json: String::new(),
                backend_name: "mock".to_string(),
                model_name: "mock".to_string(),
                agent_name: agent_name.to_string(),
            }),
            Err(_) => Err(EngineError::Llm(crate::error::LlmFailure::EmptyResponse)),
        }
    }

    fn run_post_generation_agents(
        &self,
        _state: &GameState,
        _player_input: &str,
        _main_response: &str,
        result: &mut QuantifierResult,
    ) {
        *result = self.quantifier_result.clone();
    }
}

fn make_test_state() -> GameState {
    let world = Arc::new(TestWorld::minimal());
    let map = Arc::new(TestMap::single_room("start"));
    let player = Arc::new(TestPlayer::standard());
    let npcs = vec![TestNpc::named("npc1", "Test NPC")];
    GameState::new(world, map, player, npcs, "start".to_string())
}

fn make_ctx(state: GameState) -> GameServiceContext {
    make_test_context(state)
}

#[test]
fn test_execute_action_impl_completes_and_persists_state() {
    let state = make_test_state();
    let ctx = make_ctx(state.clone());
    let backend = MockBackend::default();

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look".to_string(),
        "Player".to_string(),
    );

    let final_state = ctx.load_state();
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle
    );
    assert_eq!(
        final_state.narrative.input_buffer.phase,
        GenerationPhase::default()
    );
    let has_narration = final_state
        .narrative
        .history()
        .iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(
        has_narration,
        "execute_action_impl should persist narration"
    );
}

#[test]
fn test_execute_action_impl_clears_last_trigger() {
    let mut state = make_test_state();
    state.narrative.last_trigger = Some(crate::test_support::TestStoredTriggerContext::named(
        "Old Trigger",
        "npc1",
    ));
    let ctx = make_ctx(state);
    let backend = MockBackend::default();

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look".to_string(),
        "Player".to_string(),
    );

    let final_state = ctx.load_state();
    assert!(
        final_state.narrative.last_trigger.is_none(),
        "last_trigger should be cleared before pipeline runs"
    );
}

#[test]
fn test_execute_action_impl_handles_narration_error() {
    let state = make_test_state();
    let ctx = make_ctx(state.clone());
    let backend = MockBackend {
        narrate_result: Err(EngineError::Llm(crate::error::LlmFailure::EmptyResponse)),
        ..Default::default()
    };

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look".to_string(),
        "Player".to_string(),
    );

    let final_state = ctx.load_state();
    assert!(
        matches!(
            final_state.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "State should reflect error status after failed narration"
    );
}

#[test]
fn test_execute_action_impl_handles_cancellation() {
    let state = make_test_state();
    let ctx = make_ctx(state.clone());
    ctx.cancel_token.cancel();
    let backend = MockBackend::default();

    execute_action_impl(
        &backend,
        ctx.clone(),
        "look".to_string(),
        "Player".to_string(),
    );

    let final_state = ctx.load_state();
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "Cancellation should reset status to Idle"
    );
}

#[test]
fn test_execute_action_impl_preserves_existing_input_log() {
    let mut state = make_test_state();
    state.add_message(
        "examine room".to_string(),
        Some("Player".to_string()),
        MessageType::Input,
    );
    let ctx = make_ctx(state);
    let backend = MockBackend::default();

    execute_action_impl(
        &backend,
        ctx.clone(),
        "examine room".to_string(),
        "Player".to_string(),
    );

    let final_state = ctx.load_state();
    let entries: Vec<_> = final_state.narrative.history().into_iter().collect();
    let input_idx = entries
        .iter()
        .position(|e| e.message_type == MessageType::Input);
    let narration_idx = entries
        .iter()
        .position(|e| e.message_type == MessageType::Narration);
    assert!(input_idx.is_some(), "Existing input should be preserved");
    assert!(narration_idx.is_some(), "Narration should be added");
    assert!(
        input_idx.unwrap() < narration_idx.unwrap(),
        "Input should appear before narration"
    );
}
