use std::sync::Arc;

use crate::application::action_pipeline::pipeline::{
    ActionOutcome, ActionPipeline, ActionPipelineBackend, default_quantifier_result,
};
use crate::application::context::GameServiceContext;
use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::quantifier::{QuantifierConfidence, QuantifierParseResult, QuantifierResult};
use crate::model::state::StoredTriggerContext;
use crate::model::state::{GameState, GenerationPhase, GenerationStatus, LogType};
use crate::narrative::llm::backend::{AGENT_NARRATOR, LlmCallResult};
use crate::narrative::prompt::{LayeredPromptAssembler, PromptAssembler};
use crate::storage::snapshot_storage::SnapshotStorage;
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

struct FailingSaveStorage {
    inner: Arc<dyn SnapshotStorage>,
}

impl SnapshotStorage for FailingSaveStorage {
    fn set_game_id(&self, game_id: u64) {
        self.inner.set_game_id(game_id);
    }
    fn current_game_id(&self) -> u64 {
        self.inner.current_game_id()
    }
    fn save(
        &self,
        _snapshot: &crate::model::state_snapshot::GameStateSnapshot,
    ) -> Result<u64, crate::error::EngineError> {
        Err(crate::error::EngineError::Internal(
            crate::error::internal_error("simulated save failure"),
        ))
    }
    fn load_latest(
        &self,
    ) -> Result<Option<crate::model::state_snapshot::GameStateSnapshot>, crate::error::EngineError>
    {
        self.inner.load_latest()
    }
    fn load_by_id(
        &self,
        id: u64,
    ) -> Result<Option<crate::model::state_snapshot::GameStateSnapshot>, crate::error::EngineError>
    {
        self.inner.load_by_id(id)
    }
    fn list_games(&self) -> Result<Vec<crate::model::game::Game>, crate::error::EngineError> {
        self.inner.list_games()
    }
    fn create_game(&self, world_name: &str, name: &str) -> Result<u64, crate::error::EngineError> {
        self.inner.create_game(world_name, name)
    }
    fn delete_game(&self, id: u64) -> Result<(), crate::error::EngineError> {
        self.inner.delete_game(id)
    }
    fn get_game(
        &self,
        id: u64,
    ) -> Result<Option<crate::model::game::Game>, crate::error::EngineError> {
        self.inner.get_game(id)
    }
}

#[test]
fn test_run_trigger_continuation_cancels_at_start() {
    let state = make_test_state();
    let ctx = make_ctx(state.clone());
    ctx.cancel_token.cancel();

    let backend = MockPipelineBackend::default();
    let pipeline = ActionPipeline::new(&backend, &ctx);

    let trigger = StoredTriggerContext {
        npc_id: "npc1".to_string(),
        trigger_idx: 0,
        trigger_name: "Test".to_string(),
        trigger_repeat: false,
        trigger_narration_prompt: "Hello".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: None,
    };

    let outcome = pipeline.run_trigger_continuation(state, trigger, "look");

    assert!(
        matches!(outcome, ActionOutcome::Cancelled),
        "Expected cancellation at start of trigger continuation, got {outcome:?}"
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
}

#[test]
fn test_trigger_continuation_save_post_trigger_error() {
    let state = make_test_state();
    let base_ctx = make_ctx(state.clone());

    let failing = Arc::new(FailingSaveStorage {
        inner: Arc::clone(&base_ctx.snapshot_storage),
    });

    let ctx = GameServiceContext {
        snapshot_storage: failing,
        ..base_ctx.clone()
    };

    let backend = MockPipelineBackend::default();
    let pipeline = ActionPipeline::new(&backend, &ctx);

    let trigger = StoredTriggerContext {
        npc_id: "npc1".to_string(),
        trigger_idx: 0,
        trigger_name: "Test".to_string(),
        trigger_repeat: false,
        trigger_narration_prompt: "Hello".to_string(),
        system_prompt: "sys".to_string(),
        user_prompt: "user".to_string(),
        max_tokens: None,
    };

    let outcome = pipeline.run_trigger_continuation(state, trigger, "look");

    assert!(
        matches!(outcome, ActionOutcome::Error { ref message } if message.contains("Failed to save post-trigger retry snapshot")),
        "Expected save error for post-trigger snapshot, got {outcome:?}"
    );
}

#[test]
fn test_pipeline_trigger_happy_path() {
    use crate::model::trigger::{ComparisonOperator, Trigger, TriggerCondition, TriggerEffect};

    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: crate::test_support::fixtures::TestPlayer::standard().sheet,
        inventory: vec![],
        triggers: vec![Trigger {
            condition: TriggerCondition::TimesMet(ComparisonOperator::Eq, 0),
            effect: TriggerEffect {
                name: "Greeting".to_string(),
                narration_prompt: "The NPC greets you warmly.".to_string(),
            },
            repeat: false,
            room_id: None,
        }],
        relationships: vec![],
    };

    let world = Arc::new(crate::test_support::fixtures::TestWorld::minimal());
    let map = Arc::new(crate::test_support::fixtures::TestMap::single_room("start"));
    let player = Arc::new(crate::test_support::fixtures::TestPlayer::standard());
    let state = GameState::new(world, map, player, vec![npc], "start".to_string());

    let ctx = make_ctx(state.clone());
    let _backend = MockPipelineBackend::default();
    // Quantifier must return npc1 so trigger condition (times_met == 0) is evaluated
    let custom_quantifier = QuantifierResult {
        npcs: crate::model::quantifier::QuantifierParseResult {
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

    assert!(
        matches!(outcome, ActionOutcome::Completed),
        "Expected Completed, got {outcome:?}"
    );
    let final_state = ctx.load_state();
    assert!(
        final_state
            .narrative
            .history()
            .iter()
            .any(|e| e.text.contains("glows brighter") || e.text.contains("greets")),
        "Trigger continuation text should appear in history"
    );
    assert!(
        final_state.narrative.last_trigger.is_some(),
        "last_trigger should be set"
    );
}

#[test]
fn test_pipeline_trigger_empty_continuation() {
    use crate::model::trigger::{ComparisonOperator, Trigger, TriggerCondition, TriggerEffect};

    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: crate::test_support::fixtures::TestPlayer::standard().sheet,
        inventory: vec![],
        triggers: vec![Trigger {
            condition: TriggerCondition::TimesMet(ComparisonOperator::Eq, 0),
            effect: TriggerEffect {
                name: "Greeting".to_string(),
                narration_prompt: "The NPC greets you.".to_string(),
            },
            repeat: false,
            room_id: None,
        }],
        relationships: vec![],
    };

    let world = Arc::new(crate::test_support::fixtures::TestWorld::minimal());
    let map = Arc::new(crate::test_support::fixtures::TestMap::single_room("start"));
    let player = Arc::new(crate::test_support::fixtures::TestPlayer::standard());
    let state = GameState::new(world, map, player, vec![npc], "start".to_string());

    let ctx = make_ctx(state.clone());
    let custom_quantifier = QuantifierResult {
        npcs: crate::model::quantifier::QuantifierParseResult {
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
        complete_result: Ok("".to_string()),
        ..Default::default()
    };
    let pipeline = ActionPipeline::new(&backend, &ctx);

    let outcome = pipeline.run_from_input(state, "look".to_string());

    assert!(
        matches!(outcome, ActionOutcome::Error { ref message } if message.contains("empty response")),
        "Expected empty response error for trigger, got {outcome:?}"
    );
}

#[test]
fn test_pipeline_trigger_complete_failure() {
    use crate::model::trigger::{ComparisonOperator, Trigger, TriggerCondition, TriggerEffect};

    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: crate::test_support::fixtures::TestPlayer::standard().sheet,
        inventory: vec![],
        triggers: vec![Trigger {
            condition: TriggerCondition::TimesMet(ComparisonOperator::Eq, 0),
            effect: TriggerEffect {
                name: "Greeting".to_string(),
                narration_prompt: "The NPC greets you.".to_string(),
            },
            repeat: false,
            room_id: None,
        }],
        relationships: vec![],
    };

    let world = Arc::new(crate::test_support::fixtures::TestWorld::minimal());
    let map = Arc::new(crate::test_support::fixtures::TestMap::single_room("start"));
    let player = Arc::new(crate::test_support::fixtures::TestPlayer::standard());
    let state = GameState::new(world, map, player, vec![npc], "start".to_string());

    let ctx = make_ctx(state.clone());
    let custom_quantifier = QuantifierResult {
        npcs: crate::model::quantifier::QuantifierParseResult {
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
        complete_result: Err(EngineError::Llm(crate::error::LlmFailure::EmptyResponse)),
        ..Default::default()
    };
    let pipeline = ActionPipeline::new(&backend, &ctx);

    let outcome = pipeline.run_from_input(state, "look".to_string());

    assert!(
        matches!(outcome, ActionOutcome::Error { ref message } if message.contains("Trigger narration failed")),
        "Expected trigger narration failure, got {outcome:?}"
    );
}
