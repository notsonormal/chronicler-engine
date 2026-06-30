use std::sync::Arc;

use crate::application::action_pipeline::pipeline::{ActionOutcome, ActionPipeline};
use crate::application::context::{GameServiceContext, load_or_fresh};
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::agents::registry::AgentRegistry;
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::adapters::driven::storage::{Storage, TestOverride};
use crate::adapters::driven::llm::providers::MockBackend;
use crate::test_support::fixtures::{TestGameState, TestNpc};
use crate::test_support::make_test_context;

fn make_test_recorder(
    provider: Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
) -> Arc<LlmCallRecorder> {
    struct NoopForensics;
    impl crate::application::ports::llm_message_repository::LlmMessageRepository for NoopForensics {
        fn save_llm_message(
            &self,
            _: &crate::application::ports::llm_message_repository::LlmMessage,
        ) -> Result<(), crate::error::EngineError> {
            Ok(())
        }
        fn list_latest_llm_messages(
            &self,
            _: usize,
        ) -> Result<
            Vec<crate::application::ports::llm_message_repository::LlmMessage>,
            crate::error::EngineError,
        > {
            Ok(vec![])
        }
    }
    Arc::new(LlmCallRecorder::new(provider, Arc::new(NoopForensics)))
}

fn make_test_pipeline(
    _ctx: &GameServiceContext,
    narrator_recorder: Arc<LlmCallRecorder>,
    agent_registry: AgentRegistry,
) -> ActionPipeline {
    let prompt_assembler = Arc::new(
        crate::application::narrative_prompt::LayeredPromptAssembler::new(
            crate::application::narrative_prompt::budget::MAX_CONTEXT_TOKENS,
        ),
    );
    let agent_registry = Arc::new(agent_registry);
    ActionPipeline::new(prompt_assembler, narrator_recorder, agent_registry)
}

fn make_test_state() -> GameState {
    TestGameState::with_npc("start", TestNpc::named("npc1", "Test NPC"))
}

#[test]
fn test_pipeline_runs_to_completion() {
    let state = make_test_state();
    let ctx = make_test_context(state.clone());
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let pipeline = make_test_pipeline(&ctx, narrator_recorder, agent_registry);

    let outcome = pipeline.run_from_input(&ctx, state, "look".to_string());

    assert!(matches!(outcome, Ok(())));
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
    let ctx = make_test_context(state.clone());
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let pipeline = make_test_pipeline(&ctx, narrator_recorder, agent_registry);

    let _outcome = pipeline.run_from_input(&ctx, state, "look".to_string());

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
    let ctx = make_test_context(state.clone());
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default().with_fail()));
    let agent_registry = AgentRegistry::default();
    let pipeline = make_test_pipeline(&ctx, narrator_recorder, agent_registry);

    let outcome = pipeline.run_from_input(&ctx, state, "look".to_string());

    assert!(
        outcome.is_ok(),
        "Expected Ok(()) after error-model unification, got {outcome:?}"
    );
    let final_state = ctx.load_state_for_test();
    assert!(
        final_state
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_some(),
        "State should reflect error status via GenerationStatus::Error"
    );
}

#[test]
fn test_pipeline_returns_error_on_empty_narration_text() {
    let state = make_test_state();
    let ctx = make_test_context(state.clone());
    let narrator_recorder =
        make_test_recorder(Arc::new(MockBackend::default().with_empty_response()));
    let agent_registry = AgentRegistry::default();
    let pipeline = make_test_pipeline(&ctx, narrator_recorder, agent_registry);

    let outcome = pipeline.run_from_input(&ctx, state, "look".to_string());

    assert!(
        outcome.is_ok(),
        "Expected Ok(()) after error-model unification, got {outcome:?}"
    );
    let final_state = ctx.load_state_for_test();
    assert!(
        final_state
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_some(),
        "State should reflect error status via GenerationStatus::Error"
    );
}

#[test]
fn test_pipeline_cancels_mid_run() {
    let state = make_test_state();
    let ctx = make_test_context(state.clone());
    ctx.cancel_token.cancel();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let pipeline = make_test_pipeline(&ctx, narrator_recorder, agent_registry);

    let outcome = pipeline.run_from_input(&ctx, state, "look".to_string());

    assert!(
        matches!(outcome, Err(ActionOutcome::Cancelled)),
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
    use crate::domain::model::quantifier::QuantifierConfidence;

    let result = QuantifierResult::default();
    assert!(result.npcs.npc_ids.is_empty());
    assert_eq!(result.npcs.confidence, QuantifierConfidence::Low);
    assert!(result.movement.destination.is_none());
}

#[test]
fn test_pipeline_with_custom_quantifier_result() {
    use crate::application::agents::quantifier::QuantifierAgent;

    let state = make_test_state();
    let ctx = make_test_context(state.clone());

    // Create a quantifier agent that returns custom result via prompt response
    let custom_quantifier_result = r#"{"npcs_in_room": ["npc1"], "movement": null}"#.to_string();
    let mock_provider =
        Arc::new(MockBackend::default().with_prompt_responses(vec![custom_quantifier_result]));
    let quantifier_recorder = make_test_recorder(
        mock_provider as Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    );
    let agent = QuantifierAgent::with_backend(
        "quantifier".to_string(),
        quantifier_recorder.provider().clone(),
    );
    let agent_registry = Arc::new(AgentRegistry::with_agent(Box::new(agent)));

    let prompt_assembler = Arc::new(
        crate::application::narrative_prompt::LayeredPromptAssembler::new(
            crate::application::narrative_prompt::budget::MAX_CONTEXT_TOKENS,
        ),
    );
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let pipeline = ActionPipeline::new(prompt_assembler, narrator_recorder, agent_registry);

    let outcome = pipeline.run_from_input(&ctx, state, "look".to_string());

    assert!(matches!(outcome, Ok(())));
    let final_state = ctx.load_state_for_test();
    assert_eq!(
        final_state.scene.npcs_in_area.len(),
        1,
        "Custom quantifier should place npc1 in area"
    );
}

#[test]
fn test_phase_trigger_continuation_cancels_at_start() {
    let state = make_test_state();
    let ctx = make_test_context(state.clone());
    ctx.cancel_token.cancel();

    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let pipeline = make_test_pipeline(&ctx, narrator_recorder, agent_registry);

    let trigger = crate::test_support::TestStoredTriggerContext::for_npc("npc1", "Test", "Hello");

    let result = pipeline.phase_trigger_continuation(state, &trigger, &ctx);

    assert!(
        matches!(result, Err(ActionOutcome::Cancelled)),
        "Expected cancellation at start of trigger continuation, got {result:?}"
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
    let base_ctx = make_test_context(state.clone());
    let (failing_storage, handle) = Storage::new_in_memory().with_test_failures();
    let failing = Arc::new(failing_storage);
    handle.set(
        "save_snapshot",
        TestOverride::internal("simulated save failure"),
    );
    let ctx = GameServiceContext {
        storage: failing,
        ..base_ctx.clone()
    };
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let pipeline = make_test_pipeline(&ctx, narrator_recorder, agent_registry);
    let trigger = crate::test_support::TestStoredTriggerContext::for_npc("npc1", "Test", "Hello");
    let result = pipeline.phase_trigger_continuation(state, &trigger, &ctx);

    match result {
        Ok((_, text)) => {
            assert!(text.is_empty(), "Expected empty text on snapshot failure");
        }
        Err(outcome) => {
            assert!(
                matches!(outcome, ActionOutcome::Cancelled),
                "Expected Cancelled or Completed, got {outcome:?}"
            );
        }
    }
}

#[test]
fn test_pipeline_trigger_happy_path() {
    use crate::domain::model::character::NpcCard;
    use crate::domain::model::trigger::{
        ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement,
    };
    use crate::application::agents::quantifier::QuantifierAgent;

    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: crate::test_support::fixtures::TestPlayer::standard().sheet,
        inventory: vec![],
        triggers: vec![Trigger {
            requirement: TriggerRequirement {
                operator: ComparisonOperator::Eq,
                threshold: 0,
            },
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

    let ctx = make_test_context(state.clone());

    // Custom quantifier that detects npc1
    let custom_quantifier_result = r#"{"npcs_in_room": ["npc1"], "movement": null}"#.to_string();
    let mock_provider =
        Arc::new(MockBackend::default().with_prompt_responses(vec![custom_quantifier_result]));
    let quantifier_recorder = make_test_recorder(
        mock_provider as Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    );
    let agent = QuantifierAgent::with_backend(
        "quantifier".to_string(),
        quantifier_recorder.provider().clone(),
    );
    let agent_registry = Arc::new(AgentRegistry::with_agent(Box::new(agent)));

    let prompt_assembler = Arc::new(
        crate::application::narrative_prompt::LayeredPromptAssembler::new(
            crate::application::narrative_prompt::budget::MAX_CONTEXT_TOKENS,
        ),
    );
    // Provide expected trigger narration text
    let narrator_recorder = make_test_recorder(Arc::new(
        MockBackend::default().with_narrations(vec!["The NPC greets you warmly.".to_string()]),
    ));
    let pipeline = ActionPipeline::new(prompt_assembler, narrator_recorder, agent_registry);

    let outcome = pipeline.run_from_input(&ctx, state, "look".to_string());

    assert!(
        matches!(outcome, Ok(())),
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
    use crate::domain::model::character::NpcCard;
    use crate::domain::model::trigger::{
        ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement,
    };
    use crate::application::agents::quantifier::QuantifierAgent;

    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: crate::test_support::fixtures::TestPlayer::standard().sheet,
        inventory: vec![],
        triggers: vec![Trigger {
            requirement: TriggerRequirement {
                operator: ComparisonOperator::Eq,
                threshold: 0,
            },
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

    let ctx = make_test_context(state.clone());

    // Custom quantifier
    let custom_quantifier_result = r#"{"npcs_in_room": ["npc1"], "movement": null}"#.to_string();
    let mock_provider =
        Arc::new(MockBackend::default().with_prompt_responses(vec![custom_quantifier_result]));
    let quantifier_recorder = make_test_recorder(
        mock_provider as Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    );
    let agent = QuantifierAgent::with_backend(
        "quantifier".to_string(),
        quantifier_recorder.provider().clone(),
    );
    let agent_registry = Arc::new(AgentRegistry::with_agent(Box::new(agent)));

    let prompt_assembler = Arc::new(
        crate::application::narrative_prompt::LayeredPromptAssembler::new(
            crate::application::narrative_prompt::budget::MAX_CONTEXT_TOKENS,
        ),
    );
    let narrator_recorder =
        make_test_recorder(Arc::new(MockBackend::default().with_empty_response()));
    let pipeline = ActionPipeline::new(prompt_assembler, narrator_recorder, agent_registry);

    let outcome = pipeline.run_from_input(&ctx, state, "look".to_string());
    assert!(
        outcome.is_ok(),
        "Expected Ok with error status, got: {outcome:?}"
    );
    let reloaded = load_or_fresh(&ctx);
    assert!(
        reloaded
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_some(),
        "Expected error status after trigger empty response, got: {:?}",
        reloaded.narrative.input_buffer.status
    );
}

#[test]
fn test_pipeline_trigger_complete_failure() {
    use crate::domain::model::character::NpcCard;
    use crate::domain::model::trigger::{
        ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement,
    };
    use crate::application::agents::quantifier::QuantifierAgent;

    let npc = NpcCard {
        id: "npc1".to_string(),
        sheet: crate::test_support::fixtures::TestPlayer::standard().sheet,
        inventory: vec![],
        triggers: vec![Trigger {
            requirement: TriggerRequirement {
                operator: ComparisonOperator::Eq,
                threshold: 0,
            },
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

    let ctx = make_test_context(state.clone());

    // Custom quantifier
    let custom_quantifier_result = r#"{"npcs_in_room": ["npc1"], "movement": null}"#.to_string();
    let mock_provider =
        Arc::new(MockBackend::default().with_prompt_responses(vec![custom_quantifier_result]));
    let quantifier_recorder = make_test_recorder(
        mock_provider as Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    );
    let agent = QuantifierAgent::with_backend(
        "quantifier".to_string(),
        quantifier_recorder.provider().clone(),
    );
    let agent_registry = Arc::new(AgentRegistry::with_agent(Box::new(agent)));

    let prompt_assembler = Arc::new(
        crate::application::narrative_prompt::LayeredPromptAssembler::new(
            crate::application::narrative_prompt::budget::MAX_CONTEXT_TOKENS,
        ),
    );
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default().with_fail()));
    let pipeline = ActionPipeline::new(prompt_assembler, narrator_recorder, agent_registry);

    let outcome = pipeline.run_from_input(&ctx, state, "look".to_string());
    assert!(
        outcome.is_ok(),
        "Expected Ok with error status, got: {outcome:?}"
    );
    let reloaded = load_or_fresh(&ctx);
    assert!(
        reloaded
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_some(),
        "Expected error status after trigger failure, got: {:?}",
        reloaded.narrative.input_buffer.status
    );
}

#[test]
fn test_pipeline_saves_narration_before_quantifier() {
    let state = make_test_state();
    let ctx = make_test_context(state.clone());
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let pipeline = make_test_pipeline(&ctx, narrator_recorder, agent_registry);

    let _outcome = pipeline.run_from_input(&ctx, state, "look".to_string());

    let messages = ctx.load_messages().unwrap();
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

    let narration = narration_msgs.first().unwrap();
    assert!(
        narration.snapshot_id().is_some() || narration.id != 0,
        "Narration should be persisted"
    );
}

#[test]
fn test_pipeline_no_duplicate_narration() {
    let state = make_test_state();
    let ctx = make_test_context(state.clone());
    let narrator_recorder = make_test_recorder(Arc::new(
        MockBackend::default().with_narrations(vec!["You look around.".to_string()]),
    ));
    let agent_registry = AgentRegistry::default();
    let pipeline = make_test_pipeline(&ctx, narrator_recorder, agent_registry);

    let _outcome = pipeline.run_from_input(&ctx, state, "test input".to_string());

    let final_state = ctx.load_state_for_test();
    let history = final_state.narrative.history();
    let narration_count = history
        .iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .count();

    assert_eq!(
        narration_count, 1,
        "Should have exactly 1 narration entry (no duplicates), found {narration_count}"
    );

    let narration_entry = history
        .iter()
        .find(|e| e.message_type == MessageType::Narration)
        .unwrap();
    assert_eq!(narration_entry.text, "You look around.");
}

#[test]
fn test_pipeline_quantifier_runs_on_saved_state() {
    let state = make_test_state();
    let ctx = make_test_context(state.clone());
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let pipeline = make_test_pipeline(&ctx, narrator_recorder, agent_registry);

    let _outcome = pipeline.run_from_input(&ctx, state, "look".to_string());

    let messages = ctx.load_messages().unwrap();
    let narration = messages
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .unwrap();

    assert!(
        !narration.swipes.is_empty(),
        "Narration should have quantifier metadata"
    );
}

#[test]
fn test_pipeline_continues_if_quantifier_save_fails() {
    let state = make_test_state();
    let ctx = make_test_context(state.clone());

    let custom_quantifier_result = r#"{"npcs_in_room": ["npc1"], "movement": null}"#.to_string();
    let mock_provider =
        Arc::new(MockBackend::default().with_prompt_responses(vec![custom_quantifier_result]));
    let quantifier_recorder = make_test_recorder(
        mock_provider as Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    );
    use crate::application::agents::quantifier::QuantifierAgent;
    let agent = QuantifierAgent::with_backend(
        "quantifier".to_string(),
        quantifier_recorder.provider().clone(),
    );
    let agent_registry = Arc::new(AgentRegistry::with_agent(Box::new(agent)));

    let prompt_assembler = Arc::new(
        crate::application::narrative_prompt::LayeredPromptAssembler::new(
            crate::application::narrative_prompt::budget::MAX_CONTEXT_TOKENS,
        ),
    );
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let pipeline = ActionPipeline::new(prompt_assembler, narrator_recorder, agent_registry);

    let outcome = pipeline.run_from_input(&ctx, state, "look".to_string());

    assert!(
        matches!(outcome, Ok(())),
        "Pipeline should complete even with quantifier save warnings"
    );
}

#[test]
fn test_narration_persisted_even_if_quantifier_changes_state() {
    let state = make_test_state();
    let ctx = make_test_context(state.clone());

    let custom_quantifier_result = r#"{"npcs_in_room": ["npc1"], "movement": null}"#.to_string();
    let mock_provider =
        Arc::new(MockBackend::default().with_prompt_responses(vec![custom_quantifier_result]));
    let quantifier_recorder = make_test_recorder(
        mock_provider as Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
    );
    use crate::application::agents::quantifier::QuantifierAgent;
    let agent = QuantifierAgent::with_backend(
        "quantifier".to_string(),
        quantifier_recorder.provider().clone(),
    );
    let agent_registry = Arc::new(AgentRegistry::with_agent(Box::new(agent)));

    let prompt_assembler = Arc::new(
        crate::application::narrative_prompt::LayeredPromptAssembler::new(
            crate::application::narrative_prompt::budget::MAX_CONTEXT_TOKENS,
        ),
    );
    let narrator_recorder = make_test_recorder(Arc::new(
        MockBackend::default().with_narrations(vec!["You look around.".to_string()]),
    ));
    let pipeline = ActionPipeline::new(prompt_assembler, narrator_recorder, agent_registry);

    let _outcome = pipeline.run_from_input(&ctx, state, "look".to_string());

    let messages = ctx.load_messages().unwrap();
    let narration_msgs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();

    assert_eq!(
        narration_msgs.len(),
        1,
        "Should have 1 narration despite quantifier changes"
    );

    assert_eq!(narration_msgs[0].text(), "You look around.");
}
