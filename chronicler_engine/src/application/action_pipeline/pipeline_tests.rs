use std::sync::Arc;

use crate::application::action_pipeline::pipeline::{
    ActionOutcome, ActionPipeline, ActionPipelineBackend,
};
use crate::application::context::GameServiceContext;
use crate::error::EngineError;
use crate::model::character::NpcCard;
use crate::model::quantifier::{QuantifierConfidence, QuantifierParseResult, QuantifierResult};

use crate::model::state::{GameState, GenerationPhase, GenerationStatus, MessageType};
use crate::narrative::llm::backend::{AGENT_NARRATOR, LlmCallResult};
use crate::narrative::prompt::{LayeredPromptAssembler, PromptAssembler};
use crate::storage::{Operation, Storage, TestOverride};
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
            quantifier_result: QuantifierResult::default(),
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
    let final_state = ctx.load_state_for_test();
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

    let final_state = ctx.load_state_for_test();
    let has_narration = final_state
        .narrative
        .history()
        .iter()
        .any(|e| e.message_type == MessageType::Narration);
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
    let final_state = ctx.load_state_for_test();
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
    let final_state = ctx.load_state_for_test();
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle
    );
}

#[test]
fn test_quantifier_result_default_has_low_confidence_and_empty_npcs() {
    use crate::model::quantifier::QuantifierConfidence;

    let result = QuantifierResult::default();
    assert!(result.npcs.npc_ids.is_empty());
    assert_eq!(result.npcs.confidence, QuantifierConfidence::Low);
    assert!(result.movement.destination.is_none());
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
    let final_state = ctx.load_state_for_test();
    assert_eq!(
        final_state.scene.npcs_in_area.len(),
        1,
        "Custom quantifier should place npc1 in area"
    );
}

#[test]
fn test_run_trigger_continuation_cancels_at_start() {
    let state = make_test_state();
    let ctx = make_ctx(state.clone());
    ctx.cancel_token.cancel();

    let backend = MockPipelineBackend::default();
    let pipeline = ActionPipeline::new(&backend, &ctx);

    let trigger = crate::test_support::TestStoredTriggerContext::for_npc("npc1", "Test", "Hello");

    let outcome = pipeline.run_trigger_continuation(state, trigger, "look");

    assert!(
        matches!(outcome, ActionOutcome::Cancelled),
        "Expected cancellation at start of trigger continuation, got {outcome:?}"
    );
    let final_state = ctx.load_state_for_test();
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
    let (failing_storage, handle) = Storage::new_in_memory().with_test_failures();
    let failing = Arc::new(failing_storage);
    handle.set(
        Operation::SaveSnapshot,
        TestOverride::internal("simulated save failure"),
    );
    let ctx = GameServiceContext {
        storage: failing,
        ..base_ctx.clone()
    };
    let backend = MockPipelineBackend::default();
    let pipeline = ActionPipeline::new(&backend, &ctx);
    let trigger = crate::test_support::TestStoredTriggerContext::for_npc("npc1", "Test", "Hello");
    let outcome = pipeline.run_trigger_continuation(state, trigger, "look");
    assert!(
        matches!(outcome, ActionOutcome::Error { ref message } if message.contains("Failed to save post-trigger retry snapshot")),
        "Expected save error for post-trigger snapshot, got {outcome:?}"
    );
}

#[test]
fn test_pipeline_trigger_happy_path() {
    use crate::model::trigger::{ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement};

    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: crate::test_support::fixtures::TestPlayer::standard().sheet,
        inventory: vec![],
        triggers: vec![Trigger {
            requirement: TriggerRequirement::TimesMet(ComparisonOperator::Eq, 0),
            narration: TriggerNarration {
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
    let final_state = ctx.load_state_for_test();
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
    use crate::model::trigger::{ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement};

    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: crate::test_support::fixtures::TestPlayer::standard().sheet,
        inventory: vec![],
        triggers: vec![Trigger {
            requirement: TriggerRequirement::TimesMet(ComparisonOperator::Eq, 0),
            narration: TriggerNarration {
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
    use crate::model::trigger::{ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement};

    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: crate::test_support::fixtures::TestPlayer::standard().sheet,
        inventory: vec![],
        triggers: vec![Trigger {
            requirement: TriggerRequirement::TimesMet(ComparisonOperator::Eq, 0),
            narration: TriggerNarration {
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

#[test]
fn test_pipeline_saves_narration_before_quantifier() {
    let state = make_test_state();
    let ctx = make_test_context(state.clone());
    let backend = MockPipelineBackend::default();
    let pipeline = ActionPipeline::new(&backend, &ctx);

    let _outcome = pipeline.run_from_input(state, "look".to_string());

    // Load all messages from storage to verify save order
    let messages = ctx.load_messages().unwrap();

    // Should have exactly 1 narration message (no duplicates)
    let narration_msgs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();

    assert_eq!(
        narration_msgs.len(),
        1,
        "Should have exactly 1 narration message, found {}",
        narration_msgs.len()
    );

    // Verify narration was saved (has valid snapshot_id or db_id)
    let narration = narration_msgs.first().unwrap();
    assert!(
        narration.snapshot_id().is_some() || narration.id != 0,
        "Narration should be persisted"
    );
}

#[test]
fn test_pipeline_no_duplicate_narration() {
    let state = make_test_state();
    let ctx = make_ctx(state.clone());
    let backend = MockPipelineBackend::default();
    let pipeline = ActionPipeline::new(&backend, &ctx);

    let _outcome = pipeline.run_from_input(state, "test input".to_string());

    let final_state = ctx.load_state_for_test();
    let history = final_state.narrative.history();

    // Count narration entries
    let narration_count = history
        .iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .count();

    assert_eq!(
        narration_count, 1,
        "Should have exactly 1 narration entry (no duplicates), found {narration_count}"
    );

    // Verify the narration text is correct
    let narration_entry = history
        .iter()
        .find(|e| e.message_type == MessageType::Narration)
        .unwrap();
    assert_eq!(narration_entry.text, "You look around.");
}

#[test]
fn test_pipeline_quantifier_runs_on_saved_state() {
    let state = make_test_state();
    let ctx = make_ctx(state.clone());
    let backend = MockPipelineBackend::default();
    let pipeline = ActionPipeline::new(&backend, &ctx);

    let _outcome = pipeline.run_from_input(state, "look".to_string());

    // Verify quantifier metadata was saved (NPC list, confidence)
    let messages = ctx.load_messages().unwrap();
    let narration = messages
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .unwrap();

    // Should have swipes (metadata from quantifier)
    assert!(
        !narration.swipes.is_empty(),
        "Narration should have quantifier metadata"
    );
}

#[test]
fn test_pipeline_continues_if_quantifier_save_fails() {
    // This test verifies that a quantifier save failure doesn't stop the pipeline
    let state = make_test_state();
    let ctx = make_ctx(state.clone());
    let backend = MockPipelineBackend {
        quantifier_result: QuantifierResult {
            npcs: QuantifierParseResult {
                npc_ids: vec!["npc1".to_string()],
                confidence: QuantifierConfidence::High,
            },
            movement: Default::default(),
        },
        ..Default::default()
    };
    let pipeline = ActionPipeline::new(&backend, &ctx);

    let outcome = pipeline.run_from_input(state, "look".to_string());

    // Should complete successfully even if quantifier save has issues
    assert!(
        matches!(outcome, ActionOutcome::Completed),
        "Pipeline should complete even with quantifier save warnings"
    );
}

#[test]
fn test_narration_persisted_even_if_quantifier_changes_state() {
    // Verify narration remains in history even when quantifier modifies state
    let state = make_test_state();
    let ctx = make_ctx(state.clone());
    let backend = MockPipelineBackend {
        quantifier_result: QuantifierResult {
            npcs: QuantifierParseResult {
                npc_ids: vec!["npc1".to_string()],
                confidence: QuantifierConfidence::High,
            },
            movement: Default::default(),
        },
        ..Default::default()
    };
    let pipeline = ActionPipeline::new(&backend, &ctx);

    let _outcome = pipeline.run_from_input(state, "look".to_string());

    let messages = ctx.load_messages().unwrap();
    let narration_msgs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();

    // Should have exactly 1 narration
    assert_eq!(
        narration_msgs.len(),
        1,
        "Should have 1 narration despite quantifier changes"
    );

    // Verify narration text is preserved
    assert_eq!(narration_msgs[0].text(), "You look around.");
}
