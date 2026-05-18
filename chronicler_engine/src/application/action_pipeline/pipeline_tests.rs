use std::sync::Arc;

use crate::application::action_pipeline::pipeline::{
    ActionOutcome, ActionPipeline, ActionPipelineBackend, default_quantifier_result,
};
use crate::application::context::GameServiceContext;
use crate::error::EngineError;
use crate::model::quantifier::{QuantifierConfidence, QuantifierParseResult, QuantifierResult};
use crate::model::state::{GameState, GenerationPhase, GenerationStatus, LogType};
use crate::narrative::llm::backend::LlmCallResult;
use crate::narrative::prompt::PromptContext;
use crate::test_support::fixtures::{TestMap, TestNpc, TestPlayer, TestWorld};
use crate::test_support::make_test_context;

struct MockPipelineBackend {
    narrate_result: Result<String, EngineError>,
    complete_result: Result<String, EngineError>,
    quantifier_result: QuantifierResult,
}

impl Default for MockPipelineBackend {
    fn default() -> Self {
        Self {
            narrate_result: Ok("You look around.".to_string()),
            complete_result: Ok("The orb glows brighter.".to_string()),
            quantifier_result: default_quantifier_result(&[]),
        }
    }
}

impl ActionPipelineBackend for MockPipelineBackend {
    fn narrate_action(&self, _ctx: &PromptContext) -> Result<LlmCallResult, EngineError> {
        match &self.narrate_result {
            Ok(text) => Ok(LlmCallResult {
                text: text.clone(),
                system_prompt: String::new(),
                user_prompt: String::new(),
                raw_request_json: String::new(),
                raw_response_json: String::new(),
                backend_name: "mock".to_string(),
                model_name: "mock".to_string(),
                agent_name: "narrator".to_string(),
            }),
            Err(_e) => Err(EngineError::Llm(crate::error::LlmFailure::EmptyResponse)),
        }
    }

    fn complete(
        &self,
        _agent_name: &str,
        _system_prompt: &str,
        _user_prompt: &str,
        _max_tokens: Option<u32>,
    ) -> Result<LlmCallResult, EngineError> {
        match &self.complete_result {
            Ok(text) => Ok(LlmCallResult {
                text: text.clone(),
                system_prompt: String::new(),
                user_prompt: String::new(),
                raw_request_json: String::new(),
                raw_response_json: String::new(),
                backend_name: "mock".to_string(),
                model_name: "mock".to_string(),
                agent_name: "trigger".to_string(),
            }),
            Err(_e) => Err(EngineError::Llm(crate::error::LlmFailure::EmptyResponse)),
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
fn test_pipeline_runs_to_completion() {
    let state = make_test_state();
    let ctx = make_ctx(state.clone());
    let backend = MockPipelineBackend::default();
    let pipeline = ActionPipeline::new(&backend, &ctx);

    let outcome = pipeline.run_from_input(state, "look".to_string());

    assert!(matches!(outcome, ActionOutcome::Completed));
    let final_state = ctx.load_state();
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle
    );
    assert_eq!(
        final_state.narrative.input_buffer.phase,
        GenerationPhase::default()
    );
}

#[test]
fn test_pipeline_saves_narration_to_history() {
    let state = make_test_state();
    let ctx = make_ctx(state.clone());
    let backend = MockPipelineBackend::default();
    let pipeline = ActionPipeline::new(&backend, &ctx);

    let _outcome = pipeline.run_from_input(state, "look".to_string());

    let final_state = ctx.load_state();
    let has_narration = final_state
        .narrative
        .history()
        .iter()
        .any(|e| e.log_type == LogType::Narration);
    assert!(has_narration);
}

#[test]
fn test_pipeline_returns_error_on_narration_failure() {
    let state = make_test_state();
    let ctx = make_ctx(state.clone());
    let backend = MockPipelineBackend {
        narrate_result: Err(EngineError::Llm(crate::error::LlmFailure::EmptyResponse)),
        ..Default::default()
    };
    let pipeline = ActionPipeline::new(&backend, &ctx);

    let outcome = pipeline.run_from_input(state, "look".to_string());

    assert!(
        matches!(outcome, ActionOutcome::Error { ref message } if message.contains("empty response")),
        "Expected error for empty narration response, got {outcome:?}"
    );
    let final_state = ctx.load_state();
    assert!(
        matches!(
            final_state.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "State should reflect error status"
    );
}

#[test]
fn test_pipeline_returns_error_on_empty_narration_text() {
    let state = make_test_state();
    let ctx = make_ctx(state.clone());
    let backend = MockPipelineBackend {
        narrate_result: Ok("".to_string()),
        ..Default::default()
    };
    let pipeline = ActionPipeline::new(&backend, &ctx);

    let outcome = pipeline.run_from_input(state, "look".to_string());

    assert!(
        matches!(outcome, ActionOutcome::Error { ref message } if message.contains("empty response")),
        "Expected error for empty narration text, got {outcome:?}"
    );
}

#[test]
fn test_pipeline_cancels_mid_run() {
    let state = make_test_state();
    let ctx = make_ctx(state.clone());
    ctx.cancel_token.cancel();
    let backend = MockPipelineBackend::default();
    let pipeline = ActionPipeline::new(&backend, &ctx);

    let outcome = pipeline.run_from_input(state, "look".to_string());

    assert!(
        matches!(outcome, ActionOutcome::Cancelled),
        "Expected cancellation when token is cancelled, got {outcome:?}"
    );
    let final_state = ctx.load_state();
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle
    );
}

#[test]
fn test_default_quantifier_result_uses_fallback_ids() {
    let fallback = vec!["npc1".to_string(), "npc2".to_string()];
    let result = default_quantifier_result(&fallback);
    assert_eq!(result.npcs.npc_ids, fallback);
    assert_eq!(result.npcs.confidence, QuantifierConfidence::Low);
}

#[test]
fn test_pipeline_backend_trait_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MockPipelineBackend>();
}

#[test]
fn test_pipeline_with_custom_quantifier_result() {
    let state = make_test_state();
    let ctx = make_ctx(state.clone());
    let custom_quantifier = QuantifierResult {
        npcs: QuantifierParseResult {
            npc_ids: vec!["npc1".to_string()],
            confidence: QuantifierConfidence::High,
        },
        movement: crate::model::quantifier::MovementParseResult {
            destination: None,
            movement_type: None,
            confidence: crate::model::quantifier::QuantifierConfidence::Low,
        },
    };
    let backend = MockPipelineBackend {
        quantifier_result: custom_quantifier,
        ..Default::default()
    };
    let pipeline = ActionPipeline::new(&backend, &ctx);

    let outcome = pipeline.run_from_input(state, "look".to_string());

    assert!(matches!(outcome, ActionOutcome::Completed));
    let final_state = ctx.load_state();
    assert_eq!(
        final_state.scene.npcs_in_area.len(),
        1,
        "Custom quantifier should place npc1 in area"
    );
}
