//! Unit tests for action pipeline orchestration and execution.

use std::collections::HashMap;
use std::sync::Arc;

use crate::adapters::driving::http::AppState;
use crate::application::pipeline::phase_error::PhaseError;
use crate::application::pipeline::phases::PipelineRun;
use crate::test_support::make_test_recorder;
use crate::application::agents::registry::AgentRegistry;
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::map::Room;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::adapters::driven::storage::{Storage, TestOverride};
use crate::adapters::driven::llm::providers::MockBackend;
use crate::test_support::{make_test_pipeline_with_backends, TestAppBuilder, TestDataBuilder};
use crate::test_support::fixtures::{TestGameState, TestMap, TestNpc};

fn make_test_state() -> GameState {
    TestGameState::in_room("start")
}

#[test]
fn test_pipeline_runs_to_completion() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let outcome = app.pipeline.run_from_input(state, "look".to_string());

    assert!(matches!(outcome, Ok(())));
    let final_state = app.message_service.load_or_fresh();
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
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let _outcome = app.pipeline.run_from_input(state, "look".to_string());

    let final_state = app.message_service.load_or_fresh();
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
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let outcome = app.pipeline.run_from_input(state, "look".to_string());

    assert!(
        outcome.is_ok(),
        "Expected Ok(()) after error-model unification, got {outcome:?}"
    );
    let final_state = app.message_service.load_or_fresh();
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
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let outcome = app.pipeline.run_from_input(state, "look".to_string());

    assert!(
        outcome.is_ok(),
        "Expected Ok(()) after error-model unification, got {outcome:?}"
    );
    let final_state = app.message_service.load_or_fresh();
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
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let outcome = app.pipeline.run_from_input(state, "look".to_string());

    assert!(matches!(outcome, Ok(())));
    let final_state = app.message_service.load_or_fresh();
    assert_eq!(
        final_state.scene.npcs_in_area.len(),
        1,
        "Custom quantifier should place npc1 in area"
    );
}

#[test]
fn test_trigger_continuation_save_post_trigger_error() {
    let mut state = make_test_state();
    let (failing_storage, handle) = Storage::new_in_memory().with_test_failures();
    let failing = Arc::new(failing_storage);
    handle.set(
        "save_snapshot",
        TestOverride::internal("simulated save failure"),
    );
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = AppState::from_wired(
        crate::test_support::build_test_wired_app(
            failing,
            Arc::new(crate::adapters::driven::storage::Storage::new_in_memory()),
            service,
        )
        .expect("build_test_wired_app: build_app_graph_for_tests should succeed"),
    );
    let trigger = crate::test_support::TestStoredTriggerContext::for_npc("npc1", "Test", "Hello");
    let map = Arc::new(TestMap::single_room("start"));
    let npcs = HashMap::from([("npc1".to_string(), TestNpc::named("npc1", "Test NPC"))]);
    let result = app
        .pipeline
        .phase_trigger_continuation(&mut state, &trigger, &map, &npcs);

    match result {
        Err(PhaseError::PersistFailed { label, .. }) => {
            assert_eq!(label, "pre-event snapshot");
        }
        other => panic!("Expected Err(PersistFailed pre-event snapshot), got {other:?}"),
    }
}

#[test]
fn phase_post_generation_returns_persist_error_on_pre_quantifier_save_failure() {
    use crate::domain::model::character::NpcCard;

    let (failing_storage, handle) = Storage::new_in_memory().with_test_failures();
    let failing = Arc::new(failing_storage);
    handle.set(
        "save_snapshot",
        TestOverride::internal("simulated pre-quantifier save failure"),
    );

    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let pipeline = make_test_pipeline_with_backends(failing, narrator_recorder, agent_registry);

    let mut state = TestGameState::in_room("start");
    let map = Arc::new(TestMap::single_room("start"));
    let persona = Arc::new(crate::test_support::fixtures::TestPersona::standard());
    let npcs: HashMap<String, NpcCard> = HashMap::new();

    let run = PipelineRun::new(&pipeline, 0);
    let result = run.phase_post_generation(
        &mut state,
        "look",
        "You look around.",
        &map,
        &persona,
        &npcs,
    );
    match result {
        Err(PhaseError::PersistFailed { label, .. }) => {
            assert_eq!(label, "pre-quantifier phase update");
        }
        other => panic!("Expected Err(PersistFailed pre-quantifier phase update), got {other:?}"),
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
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let outcome = app.pipeline.run_from_input(state, "look".to_string());

    assert!(
        matches!(outcome, Ok(())),
        "Expected Completed, got {outcome:?}"
    );
    let final_state = app.message_service.load_or_fresh();
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
    let has_event = final_state
        .narrative
        .history()
        .iter()
        .any(|e| e.event_header.is_some());
    assert!(has_event, "Trigger should add an event header");
    let narration_count = final_state
        .narrative
        .history()
        .iter()
        .filter(|e| e.message_type == MessageType::Narration)
        .count();
    assert!(
        narration_count >= 2,
        "Should have main narration + trigger continuation narration, found {narration_count}"
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
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let outcome = app.pipeline.run_from_input(state, "look".to_string());
    assert!(
        outcome.is_ok(),
        "Expected Ok with error status, got: {outcome:?}"
    );
    let reloaded = app.message_service.load_or_fresh();
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
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let outcome = app.pipeline.run_from_input(state, "look".to_string());
    assert!(
        outcome.is_ok(),
        "Expected Ok with error status, got: {outcome:?}"
    );
    let reloaded = app.message_service.load_or_fresh();
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
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let _outcome = app.pipeline.run_from_input(state, "look".to_string());

    let messages = app.message_service.load_messages().unwrap();
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
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let _outcome = app.pipeline.run_from_input(state, "test input".to_string());

    let final_state = app.message_service.load_or_fresh();
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
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let _outcome = app.pipeline.run_from_input(state, "look".to_string());

    let messages = app.message_service.load_messages().unwrap();
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
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let outcome = app.pipeline.run_from_input(state, "look".to_string());

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
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let _outcome = app.pipeline.run_from_input(state, "look".to_string());

    let messages = app.message_service.load_messages().unwrap();
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

#[test]
fn orchestrator_records_error_when_world_missing() {
    let data = TestDataBuilder::default_test().build();
    let (storage, handle) = {
        let base = Storage::new_in_memory();
        data.seed_into(&base);
        base.with_test_failures()
    };
    handle.set(
        "get_world",
        TestOverride::internal("simulated get_world failure"),
    );
    let app = TestAppBuilder::with_data(data)
        .storage(Arc::new(storage))
        .skip_seeding(true)
        .build_service();

    let state = app.message_service.load_or_fresh();
    let outcome = app.pipeline.run_from_input(state, "look".to_string());

    assert!(
        matches!(outcome, Ok(())),
        "Pipeline must return Ok(()) on fetch failure (no panic, no Failed variant), got {outcome:?}"
    );
    let final_state = app.message_service.load_or_fresh();
    assert!(
        matches!(
            final_state.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "Pipeline must set GenerationStatus::Error when get_world fails, got {:?}",
        final_state.narrative.input_buffer.status
    );
}

#[test]
fn orchestrator_records_error_when_persona_missing() {
    let data = TestDataBuilder::default_test().build();
    let (storage, handle) = {
        let base = Storage::new_in_memory();
        data.seed_into(&base);
        base.with_test_failures()
    };
    handle.set(
        "get_persona",
        TestOverride::internal("simulated get_persona failure"),
    );
    let app = TestAppBuilder::with_data(data)
        .storage(Arc::new(storage))
        .skip_seeding(true)
        .build_service();

    let state = app.message_service.load_or_fresh();
    let outcome = app.pipeline.run_from_input(state, "look".to_string());

    assert!(
        matches!(outcome, Ok(())),
        "Pipeline must return Ok(()) on fetch failure (no panic, no Failed variant), got {outcome:?}"
    );
    let final_state = app.message_service.load_or_fresh();
    assert!(
        matches!(
            final_state.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "Pipeline must set GenerationStatus::Error when get_persona fails, got {:?}",
        final_state.narrative.input_buffer.status
    );
}

#[test]
fn test_pipeline_persists_input_before_narration() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(
        MockBackend::default().with_narrations(vec!["You look around.".to_string()]),
    ));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let mut state = app.message_service.load_or_fresh();
    state.add_message(
        "look".to_string(),
        Some("Test Player".to_string()),
        MessageType::Input,
    );
    let _outcome = app.pipeline.run_from_input(state, "look".to_string());

    let messages = app.message_service.load_messages().unwrap();
    let input_idx = messages
        .iter()
        .position(|m| m.message_type == MessageType::Input && m.text() == "look");
    let narration_idx = messages
        .iter()
        .position(|m| m.message_type == MessageType::Narration);

    assert!(input_idx.is_some(), "Input message should be persisted");
    assert!(narration_idx.is_some(), "Narration should be produced");
    assert!(
        input_idx.unwrap() < narration_idx.unwrap(),
        "Input should appear before Narration in history"
    );
}

#[test]
fn load_or_fresh_unchanged_on_world_data_missing() {
    // Snapshot-only read; missing world rows must not panic. `build_fresh_initial_state` fetches world data only as fallback.
    let storage = {
        let base = Storage::new_in_memory();
        let id = base
            .create_game(
                "Missing World",
                "missing_world",
                "missing_persona",
                "Test Player",
                "Test Game",
            )
            .expect("test setup: create game");
        base.set_game_id(id);
        let state = GameState::new("room_1");
        let snapshot = GameStateSnapshot::from_game_state(&state);
        base.save_snapshot(&snapshot)
            .expect("test setup: save snapshot");
        base
    };
    let app = TestAppBuilder::default_test()
        .storage(Arc::new(storage))
        .skip_seeding(true)
        .build_service();

    let loaded = app.message_service.load_or_fresh();
    assert_eq!(
        loaded.movement.current_room_id, "room_1",
        "load_or_fresh must return snapshot-derived GameState when world data is missing, got {loaded:?}"
    );
}

#[test]
fn phase_narrate_resolves_dynamic_room_via_fallback() {
    let data = TestDataBuilder::default_test().build();
    let storage = Arc::new(Storage::new_in_memory());
    data.seed_into(&storage);
    let app = TestAppBuilder::with_data(data)
        .storage(Arc::clone(&storage))
        .skip_seeding(true)
        .build_service();

    let mut state = app.message_service.load_or_fresh();
    let dynamic_id = "dynamic_room_alcove".to_string();
    state.movement.dynamic_rooms.insert(
        dynamic_id.clone(),
        Room {
            id: dynamic_id.clone(),
            name: "Mysterious Alcove".to_string(),
            description: "An alcove not on any map.".to_string(),
            exits: HashMap::new(),
            items: vec![],
            image_path: None,
            navigation_description: None,
        },
    );
    state.movement.current_room_id = dynamic_id.clone();

    let outcome = app.pipeline.run_from_input(state, "look".to_string());

    assert!(
        matches!(outcome, Ok(())),
        "Pipeline must return Ok(()) when current_room_id is a dynamic room, got {outcome:?}"
    );
    let final_state = app.message_service.load_or_fresh();
    assert!(
        !matches!(
            final_state.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "Pipeline must NOT set GenerationStatus::Error when current_room_id is a dynamic room (room lookup must use the dynamic_rooms fallback), got {:?}",
        final_state.narrative.input_buffer.status
    );
}

#[test]
fn orchestrator_records_canonical_persona_not_found_when_persona_missing() {
    let data = TestDataBuilder::default_test().build();
    let storage = {
        let base = Storage::new_in_memory();
        // Seed only the world — persona row intentionally absent.
        base.seed_world(&data.world, &data.map)
            .expect("test setup: seed test world");
        let id = base
            .create_game(
                &data.world.name,
                &data.world.key,
                "__missing_persona__",
                &data.persona.sheet.name,
                "Test Game",
            )
            .expect("test setup: create game");
        base.set_game_id(id);
        base
    };
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::default_test()
        .storage(Arc::new(storage))
        .skip_seeding(true)
        .pipeline(service)
        .build_service();

    let state = app.message_service.load_or_fresh();
    let outcome = app.pipeline.run_from_input(state, "look".to_string());

    assert!(
        matches!(outcome, Ok(())),
        "Pipeline must return Ok(()) (no panic, no Failed variant), got {outcome:?}"
    );
    let final_state = app.message_service.load_or_fresh();
    let msg = match &final_state.narrative.input_buffer.status {
        GenerationStatus::Error(m) => m.clone(),
        other => panic!("expected GenerationStatus::Error, got {other:?}"),
    };
    assert!(
        msg.contains("Persona not found: __missing_persona__"),
        "expected canonical 'Persona not found: __missing_persona__' in error message, got: {msg}"
    );
}

// Empty input produces a continuation narration without adding an Input message (spec S1.5, HTTP-covered).
#[test]
fn test_pipeline_empty_input_produces_continuation() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let _outcome = app.pipeline.run_from_input(state, String::new());

    let final_state = app.message_service.load_or_fresh();
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "Empty input should complete generation: {:?}",
        final_state.narrative.input_buffer.status
    );
    let has_narration = final_state
        .narrative
        .history()
        .iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(
        has_narration,
        "Empty input should produce continuation narration"
    );
    let has_input = final_state
        .narrative
        .history()
        .iter()
        .any(|e| e.message_type == MessageType::Input);
    assert!(!has_input, "Empty input should not add an Input message");
}

// Nonexistent room sets generation status to Error (spec S2.1, HTTP-covered).
#[test]
fn test_pipeline_room_not_found_sets_error_status() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let mut state = app.message_service.load_or_fresh();
    state.movement.current_room_id = "non_existent_room".to_string();
    let _outcome = app.pipeline.run_from_input(state, "look".to_string());

    let final_state = app.message_service.load_or_fresh();
    let msg = match &final_state.narrative.input_buffer.status {
        GenerationStatus::Error(m) => m.clone(),
        other => panic!("expected Error status, got {other:?}"),
    };
    assert!(
        msg.contains("Room not found"),
        "expected 'Room not found' in error message, got: {msg}"
    );
}

// execute_action clears last_trigger at the start of each action (internal state, unit-tier).
#[test]
fn test_execute_action_clears_last_trigger() {
    use crate::test_support::TestStoredTriggerContext;

    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let mut state = app.message_service.load_or_fresh();
    state.narrative.last_trigger = Some(TestStoredTriggerContext::for_npc(
        "npc_1",
        "Old",
        "The old trigger fires.",
    ));
    app.message_service
        .save_state(&state)
        .expect("save_state should succeed");

    app.pipeline.execute_action("look".to_string());

    let final_state = app.message_service.load_or_fresh();
    assert!(
        final_state.narrative.last_trigger.is_none(),
        "last_trigger should be cleared by execute_action"
    );
}

// Phase stays Narrating on narration failure (internal state, unit-tier).
#[test]
fn test_pipeline_phase_stays_narrating_on_narration_failure() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default().with_fail()));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let _outcome = app.pipeline.run_from_input(state, "look".to_string());

    let final_state = app.message_service.load_or_fresh();
    assert_eq!(
        final_state.narrative.input_buffer.phase,
        GenerationPhase::Narrating,
        "Phase should remain Narrating after failed narration"
    );
    assert!(
        final_state
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_some(),
        "Status should be Error after narration failure"
    );
}

// Empty narrator response sets Error status and persists no narration (spec S2.3, HTTP-covered).
#[test]
fn test_pipeline_empty_narration_sets_error_message_and_no_narration() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder =
        make_test_recorder(Arc::new(MockBackend::default().with_empty_response()));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let _outcome = app.pipeline.run_from_input(state, "look".to_string());

    let final_state = app.message_service.load_or_fresh();
    let msg = match &final_state.narrative.input_buffer.status {
        GenerationStatus::Error(m) => m.clone(),
        other => panic!("expected Error status, got {other:?}"),
    };
    assert!(
        msg.contains("empty"),
        "error message should mention 'empty', got: {msg}"
    );
    let has_narration = final_state
        .narrative
        .history()
        .iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(
        !has_narration,
        "no narration should be persisted on empty response"
    );
}

// Quantifier detects movement and updates current_room_id (unit-tier; room change not HTTP-observable, spec S1.3 dropped).
#[test]
fn test_pipeline_quantifier_detects_movement() {
    use crate::application::agents::quantifier::QuantifierAgent;

    let map = TestMap::two_rooms("room_1", "room_2");
    let data = TestDataBuilder::default_test().map(map).build();

    let quantifier_result =
        r#"{"npcs_in_room": [], "movement": {"type": "entering", "destination": "room_2"}}"#
            .to_string();
    let mock_provider =
        Arc::new(MockBackend::default().with_prompt_responses(vec![quantifier_result]));
    let quantifier_provider =
        mock_provider as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let agent = QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider);
    let agent_registry = AgentRegistry::with_agent(Box::new(agent));
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let _outcome = app
        .pipeline
        .run_from_input(state, "go to room 2".to_string());

    let final_state = app.message_service.load_or_fresh();
    assert_ne!(
        final_state.movement.current_room_id, "room_1",
        "Player should have moved from starting room"
    );
    assert_eq!(
        final_state.movement.current_room_id, "room_2",
        "Player should be in destination room"
    );
}

// Pipeline cancels when the shutdown token is already cancelled (spec S4.1, unit-tier).
#[test]
fn test_pipeline_cancels_when_token_already_cancelled() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    app.shutdown_token.cancel();
    let state = app.message_service.load_or_fresh();
    let _outcome = app.pipeline.run_from_input(state, "look".to_string());

    let final_state = app.message_service.load_or_fresh();
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "Should reset to Idle when cancelled"
    );
}

// Pre-main snapshot saved before narration (driven-adapter tier).
#[test]
fn test_pre_main_snapshot_saved_before_narration() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let (app, storage) = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service_with_storage();

    let state = app.message_service.load_or_fresh();
    let _outcome = app
        .pipeline
        .run_from_input(state, "examine the room".to_string());

    let latest = storage
        .load_latest_snapshot()
        .expect("load_latest_snapshot should succeed")
        .expect("a snapshot should exist after action");
    assert!(
        latest.db_id.is_some(),
        "pre-main snapshot should be persisted"
    );
}

// Pre-event snapshot saved before trigger continuation (driven-adapter tier).
#[test]
fn test_pre_event_snapshot_saved_before_continuation() {
    use crate::application::agents::quantifier::QuantifierAgent;
    use crate::domain::model::character::NpcCard;
    use crate::domain::model::trigger::{
        ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement,
    };

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

    let quantifier_result = r#"{"npcs_in_room": ["npc1"], "movement": null}"#.to_string();
    let mock_provider =
        Arc::new(MockBackend::default().with_prompt_responses(vec![quantifier_result]));
    let quantifier_provider =
        mock_provider as Arc<dyn crate::application::ports::llm_provider::LlmProvider>;
    let agent = QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider);
    let agent_registry = AgentRegistry::with_agent(Box::new(agent));
    let narrator_recorder = make_test_recorder(Arc::new(
        MockBackend::default().with_narrations(vec!["The NPC greets you warmly.".to_string()]),
    ));
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let (app, storage) = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service_with_storage();

    let state = app.message_service.load_or_fresh();
    let _outcome = app
        .pipeline
        .run_from_input(state, "examine the npc".to_string());

    let latest = storage
        .load_latest_snapshot()
        .expect("load_latest_snapshot should succeed")
        .expect("a snapshot should exist after trigger continuation");
    assert!(
        latest.db_id.is_some(),
        "pre-event snapshot should be persisted"
    );
}

// Delayed LLM completes without deadlock (spec S3.3, HTTP-covered).
#[test]
fn test_delayed_llm_completes_without_deadlock() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default().with_delay(200)));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let state = app.message_service.load_or_fresh();
    let _outcome = app
        .pipeline
        .run_from_input(state, "look around".to_string());

    let final_state = app.message_service.load_or_fresh();
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "Status should be Idle after delayed action completes"
    );
    assert_eq!(
        final_state.narrative.input_buffer.phase,
        GenerationPhase::default(),
        "Phase should be reset after completion"
    );
}

// Cancellation resets generation state to Idle (spec S4.1, unit-tier).
#[test]
fn test_cancellation_resets_state_to_idle() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default().with_delay(50)));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    app.shutdown_token.cancel();
    let state = app.message_service.load_or_fresh();
    let _outcome = app
        .pipeline
        .run_from_input(state, "look around".to_string());

    let final_state = app.message_service.load_or_fresh();
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "Status should be Idle after cancellation"
    );
}

// Pipeline cancels after main narration completes (spec S4.2, unit-tier).
#[tokio::test]
async fn test_pipeline_cancels_after_main_narration() {
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    let mock_narrator_backend = Arc::new(MockBackend::default().with_delay(50));
    let narrator_recorder = make_test_recorder(Arc::clone(&mock_narrator_backend)
        as Arc<dyn crate::application::ports::llm_provider::LlmProvider>);
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(TestDataBuilder::default_test().build())
        .pipeline(service.clone())
        .build_service();

    let token = app.shutdown_token.clone();
    let app_clone = app.clone();
    let handle = tokio::task::spawn_blocking(move || {
        app_clone.pipeline.execute_action("look around".to_string());
    });

    let started = tokio::time::timeout(Duration::from_secs(5), async {
        while !mock_narrator_backend
            .narration_started
            .load(Ordering::SeqCst)
        {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await;
    assert!(started.is_ok(), "narration should start within timeout");
    token.cancel();
    handle.await.expect("action task should complete");

    let final_state = app.message_service.load_or_fresh();
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "Status should be Idle after cancellation at post-narration checkpoint"
    );
}

// Pipeline cancels during trigger continuation (spec S4.3, unit-tier).
#[tokio::test]
async fn test_pipeline_cancels_during_trigger_continuation() {
    use crate::application::agents::quantifier::QuantifierAgent;
    use crate::domain::model::character::NpcCard;
    use crate::domain::model::trigger::{
        ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement,
    };
    use std::sync::atomic::Ordering;
    use std::time::Duration;

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

    let mock_narrator_backend = Arc::new(MockBackend::default().with_trigger_delay(50));
    let narrator_recorder = make_test_recorder(Arc::clone(&mock_narrator_backend)
        as Arc<dyn crate::application::ports::llm_provider::LlmProvider>);
    let quantifier_result = r#"{"npcs_in_room": ["npc1"], "movement": null}"#.to_string();
    let quantifier_provider: Arc<dyn crate::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default().with_prompt_responses(vec![quantifier_result]));
    let agent = QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider);
    let agent_registry = AgentRegistry::with_agent(Box::new(agent));
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let token = app.shutdown_token.clone();
    let app_clone = app.clone();
    let handle = tokio::task::spawn_blocking(move || {
        app_clone
            .pipeline
            .execute_action("enter the shop".to_string());
    });

    let started = tokio::time::timeout(Duration::from_secs(5), async {
        while !mock_narrator_backend.trigger_started.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await;
    assert!(started.is_ok(), "trigger should start within timeout");
    token.cancel();
    handle.await.expect("action task should complete");

    let final_state = app.message_service.load_or_fresh();
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "Status should be Idle after cancellation at post-trigger checkpoint"
    );
    let has_narration = final_state
        .narrative
        .history()
        .iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(has_narration, "Main narration should be preserved");
}

// Streaming narration saved before quantifier completes (mid-flight observation, unit-tier).
#[test]
fn test_streaming_narration_saved_before_quantifier_complete() {
    use std::thread;
    use std::time::Duration;

    let quantifier_provider: Arc<dyn crate::application::ports::llm_provider::LlmProvider> =
        Arc::new(
            MockBackend::default()
                .with_prompt_responses(vec![r#"{"npcs_in_room": []}"#.to_string()])
                .with_delay(500),
        );
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let service = crate::test_support::make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        quantifier_provider,
    );
    let (app, _storage) = TestAppBuilder::with_data(TestDataBuilder::default_test().build())
        .pipeline(service)
        .build_service_with_storage();

    let app_clone = app.clone();
    let message_service = Arc::clone(&app.message_service);
    let handle = thread::spawn(move || {
        app_clone.pipeline.execute_action("look around".to_string());
    });

    let start = std::time::Instant::now();
    let mut narration_found = false;
    while start.elapsed() < Duration::from_millis(400) {
        if message_service
            .load_messages()
            .map(|msgs| {
                msgs.iter()
                    .any(|m| m.message_type == MessageType::Narration)
            })
            .unwrap_or(false)
        {
            narration_found = true;
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        narration_found,
        "Narration should be saved before quantifier completes (quantifier takes 500ms)"
    );

    handle.join().expect("Action thread should complete");

    let final_state = app.message_service.load_or_fresh();
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "Should complete after quantifier finishes"
    );
    let final_narration_count = message_service
        .load_messages()
        .unwrap()
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .count();
    assert_eq!(
        final_narration_count, 1,
        "Should have exactly 1 narration (no duplicates), found {final_narration_count}"
    );
}
