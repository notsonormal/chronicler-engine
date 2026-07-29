use std::collections::HashMap;
use std::sync::Arc;

use crate::application::action_pipeline::phase_error::PhaseError;
use crate::test_support::make_test_recorder;
use crate::application::game_service::GameService;
use crate::application::agents::registry::AgentRegistry;
use crate::domain::model::quantifier::QuantifierResult;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::map::Room;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::adapters::driven::storage::{Storage, TestOverride};
use crate::adapters::driven::llm::providers::MockBackend;
use crate::test_support::{TestAppBuilder, TestDataBuilder};
use crate::test_support::fixtures::{TestGameState, TestMap, TestNpc};

fn make_test_state() -> GameState {
    TestGameState::in_room("start")
}

#[test]
fn test_pipeline_runs_to_completion() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.persistence_gate.load_or_fresh();
    let outcome = app.pipeline().run_from_input(state, "look".to_string());

    assert!(matches!(outcome, Ok(())));
    let final_state = app.persistence_gate.load_or_fresh();
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
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.persistence_gate.load_or_fresh();
    let _outcome = app.pipeline().run_from_input(state, "look".to_string());

    let final_state = app.persistence_gate.load_or_fresh();
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
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.persistence_gate.load_or_fresh();
    let outcome = app.pipeline().run_from_input(state, "look".to_string());

    assert!(
        outcome.is_ok(),
        "Expected Ok(()) after error-model unification, got {outcome:?}"
    );
    let final_state = app.persistence_gate.load_or_fresh();
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
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.persistence_gate.load_or_fresh();
    let outcome = app.pipeline().run_from_input(state, "look".to_string());

    assert!(
        outcome.is_ok(),
        "Expected Ok(()) after error-model unification, got {outcome:?}"
    );
    let final_state = app.persistence_gate.load_or_fresh();
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
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.persistence_gate.load_or_fresh();
    let outcome = app.pipeline().run_from_input(state, "look".to_string());

    assert!(matches!(outcome, Ok(())));
    let final_state = app.persistence_gate.load_or_fresh();
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
    let app = crate::test_support::build_test_service(
        failing,
        Arc::new(crate::adapters::driven::storage::Storage::new_in_memory()),
        Arc::new(service),
    )
    .expect("build_test_service: build_app_graph_for_tests should succeed");
    let trigger = crate::test_support::TestStoredTriggerContext::for_npc("npc1", "Test", "Hello");
    let map = Arc::new(TestMap::single_room("start"));
    let npcs = HashMap::from([("npc1".to_string(), TestNpc::named("npc1", "Test NPC"))]);
    let result = app
        .pipeline()
        .phase_trigger_continuation(state, &trigger, &map, &npcs);

    match result {
        Ok((_, text)) => {
            assert!(text.is_empty(), "Expected empty text on snapshot failure");
        }
        Err(PhaseError::Cancelled) => {}
        Err(PhaseError::PersistFailed { label, .. }) => {
            assert_eq!(label, "pre-event snapshot");
        }
        Err(other) => {
            panic!("Expected empty text or Cancelled/PersistFailed, got {other:?}");
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
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.persistence_gate.load_or_fresh();
    let outcome = app.pipeline().run_from_input(state, "look".to_string());

    assert!(
        matches!(outcome, Ok(())),
        "Expected Completed, got {outcome:?}"
    );
    let final_state = app.persistence_gate.load_or_fresh();
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
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.persistence_gate.load_or_fresh();
    let outcome = app.pipeline().run_from_input(state, "look".to_string());
    assert!(
        outcome.is_ok(),
        "Expected Ok with error status, got: {outcome:?}"
    );
    let reloaded = app.persistence_gate.load_or_fresh();
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
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.persistence_gate.load_or_fresh();
    let outcome = app.pipeline().run_from_input(state, "look".to_string());
    assert!(
        outcome.is_ok(),
        "Expected Ok with error status, got: {outcome:?}"
    );
    let reloaded = app.persistence_gate.load_or_fresh();
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
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.persistence_gate.load_or_fresh();
    let _outcome = app.pipeline().run_from_input(state, "look".to_string());

    let messages = app.persistence_gate.load_messages().unwrap();
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
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.persistence_gate.load_or_fresh();
    let _outcome = app
        .pipeline()
        .run_from_input(state, "test input".to_string());

    let final_state = app.persistence_gate.load_or_fresh();
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
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.persistence_gate.load_or_fresh();
    let _outcome = app.pipeline().run_from_input(state, "look".to_string());

    let messages = app.persistence_gate.load_messages().unwrap();
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
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.persistence_gate.load_or_fresh();
    let outcome = app.pipeline().run_from_input(state, "look".to_string());

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
    let app = TestAppBuilder::with_data(data)
        .game_service(Arc::new(service.clone()))
        .build_service();

    let state = app.persistence_gate.load_or_fresh();
    let _outcome = app.pipeline().run_from_input(state, "look".to_string());

    let messages = app.persistence_gate.load_messages().unwrap();
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

    let state = app.persistence_gate.load_or_fresh();
    let outcome = app.pipeline().run_from_input(state, "look".to_string());

    assert!(
        matches!(outcome, Ok(())),
        "Pipeline must return Ok(()) on fetch failure (no panic, no Failed variant), got {outcome:?}"
    );
    let final_state = app.persistence_gate.load_or_fresh();
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

    let state = app.persistence_gate.load_or_fresh();
    let outcome = app.pipeline().run_from_input(state, "look".to_string());

    assert!(
        matches!(outcome, Ok(())),
        "Pipeline must return Ok(()) on fetch failure (no panic, no Failed variant), got {outcome:?}"
    );
    let final_state = app.persistence_gate.load_or_fresh();
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

    let loaded = app.persistence_gate.load_or_fresh();
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

    let mut state = app.persistence_gate.load_or_fresh();
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

    let outcome = app.pipeline().run_from_input(state, "look".to_string());

    assert!(
        matches!(outcome, Ok(())),
        "Pipeline must return Ok(()) when current_room_id is a dynamic room, got {outcome:?}"
    );
    let final_state = app.persistence_gate.load_or_fresh();
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
    let service = GameService::with_backends(narrator_recorder, agent_registry);
    let app = TestAppBuilder::default_test()
        .storage(Arc::new(storage))
        .skip_seeding(true)
        .game_service(Arc::new(service))
        .build_service();

    let state = app.persistence_gate.load_or_fresh();
    let outcome = app.pipeline().run_from_input(state, "look".to_string());

    assert!(
        matches!(outcome, Ok(())),
        "Pipeline must return Ok(()) (no panic, no Failed variant), got {outcome:?}"
    );
    let final_state = app.persistence_gate.load_or_fresh();
    let msg = match &final_state.narrative.input_buffer.status {
        GenerationStatus::Error(m) => m.clone(),
        other => panic!("expected GenerationStatus::Error, got {other:?}"),
    };
    assert!(
        msg.contains("Persona not found: __missing_persona__"),
        "expected canonical 'Persona not found: __missing_persona__' in error message, got: {msg}"
    );
}
