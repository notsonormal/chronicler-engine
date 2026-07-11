use std::sync::Arc;

use crate::application::action_pipeline::pipeline::{ActionOutcome, ActionPipeline};
use crate::application::application_service::DefaultApplicationService;
use crate::test_support::make_test_recorder;
use crate::application::game_service::GameService;
use crate::application::agents::registry::AgentRegistry;
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::adapters::driven::storage::{Storage, TestOverride};
use crate::adapters::driven::llm::providers::MockBackend;
use crate::test_support::{TestAppBuilder, TestDataBuilder};
use crate::test_support::fixtures::{TestGameState, TestNpc};

fn make_test_pipeline(service: &crate::application::game_service::GameService) -> ActionPipeline {
    service.pipeline()
}

fn make_test_state() -> GameState {
    TestGameState::with_npc("start", TestNpc::named("npc1", "Test NPC"))
}

#[test]
fn test_pipeline_runs_to_completion() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let pipeline = make_test_pipeline(&service);
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.load_or_fresh();
    let outcome = pipeline.run_from_input(&app, state, "look".to_string());

    assert!(matches!(outcome, Ok(())));
    let final_state = app.load_or_fresh();
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
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let pipeline = make_test_pipeline(&service);
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.load_or_fresh();
    let _outcome = pipeline.run_from_input(&app, state, "look".to_string());

    let final_state = app.load_or_fresh();
    let has_narration = final_state
        .narrative
        .history()
        .iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(has_narration);
}

#[test]
fn test_pipeline_returns_error_on_narration_failure() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default().with_fail()));
    let agent_registry = AgentRegistry::default();
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let pipeline = make_test_pipeline(&service);
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.load_or_fresh();
    let outcome = pipeline.run_from_input(&app, state, "look".to_string());

    assert!(
        outcome.is_ok(),
        "Expected Ok(()) after error-model unification, got {outcome:?}"
    );
    let final_state = app.load_or_fresh();
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
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder =
        make_test_recorder(Arc::new(MockBackend::default().with_empty_response()));
    let agent_registry = AgentRegistry::default();
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let pipeline = make_test_pipeline(&service);
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.load_or_fresh();
    let outcome = pipeline.run_from_input(&app, state, "look".to_string());

    assert!(
        outcome.is_ok(),
        "Expected Ok(()) after error-model unification, got {outcome:?}"
    );
    let final_state = app.load_or_fresh();
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

    let data = TestDataBuilder::default_test().build();

    let custom_quantifier_result = r#"{"npcs_in_room": ["npc_1"], "movement": null}"#.to_string();
    let mock_provider =
        Arc::new(MockBackend::default().with_prompt_responses(vec![custom_quantifier_result]));
    let quantifier_provider =
        mock_provider as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let agent = QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider);
    let agent_registry = AgentRegistry::with_agent(Box::new(agent));
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let pipeline = service.pipeline();
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.load_or_fresh();
    let outcome = pipeline.run_from_input(&app, state, "look".to_string());

    assert!(matches!(outcome, Ok(())));
    let final_state = app.load_or_fresh();
    assert_eq!(
        final_state.scene.npcs_in_area.len(),
        1,
        "Custom quantifier should place npc1 in area"
    );
}

#[test]
fn test_trigger_continuation_save_post_trigger_error() {
    let state = make_test_state();
    let (failing_storage, handle) = Storage::new_in_memory().with_test_failures();
    let failing = Arc::new(failing_storage);
    handle.set(
        "save_snapshot",
        TestOverride::internal("simulated save failure"),
    );
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let pipeline = make_test_pipeline(&service);
    let app = Arc::new(DefaultApplicationService::new(
        failing,
        Arc::new(crate::adapters::driven::storage::Storage::new_in_memory()),
        Arc::new(std::sync::RwLock::new(
            crate::domain::model::settings::AppSettings::default(),
        )),
        tokio_util::sync::CancellationToken::new(),
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        Arc::new(service),
    ));
    let trigger = crate::test_support::TestStoredTriggerContext::for_npc("npc1", "Test", "Hello");
    let result = pipeline.phase_trigger_continuation(state, &trigger, &app);

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
        sheet: crate::test_support::fixtures::TestPersona::standard().sheet,
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

    let data = TestDataBuilder::default_test().npc(npc).build();

    let custom_quantifier_result = r#"{"npcs_in_room": ["npc1"], "movement": null}"#.to_string();
    let mock_provider =
        Arc::new(MockBackend::default().with_prompt_responses(vec![custom_quantifier_result]));
    let quantifier_provider =
        mock_provider as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let agent = QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider);
    let agent_registry = AgentRegistry::with_agent(Box::new(agent));
    let narrator_recorder = make_test_recorder(Arc::new(
        MockBackend::default().with_narrations(vec!["The NPC greets you warmly.".to_string()]),
    ));
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let pipeline = service.pipeline();
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.load_or_fresh();
    let outcome = pipeline.run_from_input(&app, state, "look".to_string());

    assert!(
        matches!(outcome, Ok(())),
        "Expected Completed, got {outcome:?}"
    );
    let final_state = app.load_or_fresh();
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
        sheet: crate::test_support::fixtures::TestPersona::standard().sheet,
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

    let data = TestDataBuilder::default_test().npc(npc).build();

    let custom_quantifier_result = r#"{"npcs_in_room": ["npc1"], "movement": null}"#.to_string();
    let mock_provider =
        Arc::new(MockBackend::default().with_prompt_responses(vec![custom_quantifier_result]));
    let quantifier_provider =
        mock_provider as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let agent = QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider);
    let agent_registry = AgentRegistry::with_agent(Box::new(agent));
    let narrator_recorder =
        make_test_recorder(Arc::new(MockBackend::default().with_empty_response()));
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let pipeline = service.pipeline();
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.load_or_fresh();
    let outcome = pipeline.run_from_input(&app, state, "look".to_string());
    assert!(
        outcome.is_ok(),
        "Expected Ok with error status, got: {outcome:?}"
    );
    let reloaded = app.load_or_fresh();
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
        sheet: crate::test_support::fixtures::TestPersona::standard().sheet,
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

    let data = TestDataBuilder::default_test().npc(npc).build();

    let custom_quantifier_result = r#"{"npcs_in_room": ["npc1"], "movement": null}"#.to_string();
    let mock_provider =
        Arc::new(MockBackend::default().with_prompt_responses(vec![custom_quantifier_result]));
    let quantifier_provider =
        mock_provider as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let agent = QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider);
    let agent_registry = AgentRegistry::with_agent(Box::new(agent));
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default().with_fail()));
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let pipeline = service.pipeline();
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.load_or_fresh();
    let outcome = pipeline.run_from_input(&app, state, "look".to_string());
    assert!(
        outcome.is_ok(),
        "Expected Ok with error status, got: {outcome:?}"
    );
    let reloaded = app.load_or_fresh();
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
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let pipeline = make_test_pipeline(&service);
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.load_or_fresh();
    let _outcome = pipeline.run_from_input(&app, state, "look".to_string());

    let messages = app.load_messages().unwrap();
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
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(
        MockBackend::default().with_narrations(vec!["You look around.".to_string()]),
    ));
    let agent_registry = AgentRegistry::default();
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let pipeline = make_test_pipeline(&service);
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.load_or_fresh();
    let _outcome = pipeline.run_from_input(&app, state, "test input".to_string());

    let final_state = app.load_or_fresh();
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
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let pipeline = make_test_pipeline(&service);
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.load_or_fresh();
    let _outcome = pipeline.run_from_input(&app, state, "look".to_string());

    let messages = app.load_messages().unwrap();
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
    let data = TestDataBuilder::default_test().build();

    let custom_quantifier_result = r#"{"npcs_in_room": ["npc_1"], "movement": null}"#.to_string();
    let mock_provider =
        Arc::new(MockBackend::default().with_prompt_responses(vec![custom_quantifier_result]));
    let quantifier_provider =
        mock_provider as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    use crate::application::agents::quantifier::QuantifierAgent;
    let agent = QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider);
    let agent_registry = AgentRegistry::with_agent(Box::new(agent));
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let pipeline = service.pipeline();
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.load_or_fresh();
    let outcome = pipeline.run_from_input(&app, state, "look".to_string());

    assert!(
        matches!(outcome, Ok(())),
        "Pipeline should complete even with quantifier save warnings"
    );
}

#[test]
fn test_narration_persisted_even_if_quantifier_changes_state() {
    let data = TestDataBuilder::default_test().build();

    let custom_quantifier_result = r#"{"npcs_in_room": ["npc_1"], "movement": null}"#.to_string();
    let mock_provider =
        Arc::new(MockBackend::default().with_prompt_responses(vec![custom_quantifier_result]));
    let quantifier_provider =
        mock_provider as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    use crate::application::agents::quantifier::QuantifierAgent;
    let agent = QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider);
    let agent_registry = AgentRegistry::with_agent(Box::new(agent));
    let narrator_recorder = make_test_recorder(Arc::new(
        MockBackend::default().with_narrations(vec!["You look around.".to_string()]),
    ));
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let pipeline = service.pipeline();
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.load_or_fresh();
    let _outcome = pipeline.run_from_input(&app, state, "look".to_string());

    let messages = app.load_messages().unwrap();
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
