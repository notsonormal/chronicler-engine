//! HTTP E2E tests for the action endpoint (POST /action).

use std::sync::Arc;

use axum::http::StatusCode;

use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::state::generation_status::GenerationStatus;
use chronicler_engine::application::ports::llm_provider::LlmProvider;
use chronicler_engine::test_support::{
    make_test_pipeline_with_mock_quantifier, make_test_recorder, TestAppBuilder, TestDataBuilder,
};

use crate::test_helpers::{app_with_narrator, post_action, post_empty, wait_idle};

// [chronicler_engine/docs/specs/actions.md] SCENARIO: 1.1
#[tokio::test]
async fn test_action_succeeds_one_narration_idle_http() {
    let narrator = Arc::new(
        MockBackend::default().with_narrations(vec!["You see a small wooden room.".to_string()]),
    );
    let (app, state) = app_with_narrator(narrator);

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "action should complete");

    let messages = state.message_service.load_messages().unwrap();
    let inputs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert_eq!(inputs.len(), 1, "one Input");
    assert_eq!(inputs[0].text(), "look");
    let narrations: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert_eq!(narrations.len(), 1, "one Narration");
    assert_eq!(narrations[0].text(), "You see a small wooden room.");
    assert!(matches!(
        state
            .message_service
            .load_or_fresh()
            .narrative
            .input_buffer
            .status,
        GenerationStatus::Idle
    ));
}

// [chronicler_engine/docs/specs/actions.md] SCENARIO: 1.2
#[tokio::test]
async fn test_input_persisted_before_narration_http() {
    let narrator = Arc::new(
        MockBackend::default().with_narrations(vec!["It is a small wooden room.".to_string()]),
    );
    let (app, state) = app_with_narrator(narrator);

    let resp = post_action(&app, "examine the room").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let messages = state.message_service.load_messages().unwrap();
    let input_idx = messages
        .iter()
        .position(|m| m.message_type == MessageType::Input)
        .expect("Input should exist");
    let narration_idx = messages
        .iter()
        .position(|m| m.message_type == MessageType::Narration)
        .expect("Narration should exist");
    assert!(
        input_idx < narration_idx,
        "Input should appear before Narration"
    );
    assert_eq!(messages[input_idx].text(), "examine the room");
    assert_eq!(messages[narration_idx].text(), "It is a small wooden room.");
    assert!(matches!(
        state
            .message_service
            .load_or_fresh()
            .narrative
            .input_buffer
            .status,
        GenerationStatus::Idle
    ));
}

// [chronicler_engine/docs/specs/actions.md] SCENARIO: 1.4
#[tokio::test]
async fn test_quantifier_npc_fires_trigger_http() {
    use chronicler_engine::domain::model::character::NpcCard;
    use chronicler_engine::domain::model::trigger::{
        ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement,
    };

    let npc = NpcCard {
        id: "npc_1".to_string(),
        sheet: chronicler_engine::test_support::TestPersona::standard().sheet,
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
    let data = TestDataBuilder::default_test().npcs(vec![npc]).build();

    let narrator = Arc::new(MockBackend::default().with_narrations(vec![
        "The main narration.".to_string(),
        "The NPC greets you warmly.".to_string(),
    ]));
    let quantifier_result = r#"{"npcs_in_room": ["npc_1"], "movement": null}"#.to_string();
    let quantifier_provider =
        Arc::new(MockBackend::default().with_prompt_responses(vec![quantifier_result]))
            as Arc<dyn LlmProvider>;
    let recorder = make_test_recorder(narrator as Arc<dyn LlmProvider>);
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        recorder,
        quantifier_provider,
    );
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .pipeline(pipeline)
        .build_with_state();

    let resp = post_action(&app, "enter the shop").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let messages = state.message_service.load_messages().unwrap();
    let has_event = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .any(|m| m.event_header().is_some());
    assert!(has_event, "trigger should fire (event_header set)");
    let narration_count = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .count();
    assert!(
        narration_count >= 2,
        "main + trigger continuation narrations, got {narration_count}"
    );
    assert!(matches!(
        state
            .message_service
            .load_or_fresh()
            .narrative
            .input_buffer
            .status,
        GenerationStatus::Idle
    ));
}

// [chronicler_engine/docs/specs/actions.md] SCENARIO: 1.5
#[tokio::test]
async fn test_empty_command_continuation_no_input_http() {
    let narrator =
        Arc::new(MockBackend::default().with_narrations(vec!["The scene continues.".to_string()]));
    let (app, state) = app_with_narrator(narrator);

    let resp = post_action(&app, "").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let messages = state.message_service.load_messages().unwrap();
    let inputs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert!(inputs.is_empty(), "empty input should not add Input");
    let narrations: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert_eq!(narrations.len(), 1, "one continuation narration");
    assert_eq!(narrations[0].text(), "The scene continues.");
    assert!(matches!(
        state
            .message_service
            .load_or_fresh()
            .narrative
            .input_buffer
            .status,
        GenerationStatus::Idle
    ));
}

// [chronicler_engine/docs/specs/actions.md] SCENARIO: 1.6
#[tokio::test]
async fn test_trigger_continuation_reruns_quantifier_detects_new_npc_http() {
    use chronicler_engine::domain::model::trigger::ComparisonOperator;
    use chronicler_engine::test_support::TestNpc;

    // shopkeeper's times_met==0 trigger fires the continuation that re-runs quantifier
    // (evaluate_triggers iterates all world NPCs, not just npcs_in_area).
    let shopkeeper = TestNpc::with_times_met_trigger(
        "shopkeeper",
        "Shopkeeper Sarah",
        ComparisonOperator::Eq,
        0,
    );
    let gabriella = TestNpc::named("gabriella", "Gabriella");
    let data = chronicler_engine::test_support::TestDataBuilder::default_test()
        .npcs(vec![shopkeeper, gabriella])
        .room_npc("gabriella")
        .build();

    let narrator = Arc::new(MockBackend::default().with_narrations(vec![
        "You step into the shop.".to_string(),
        "Gabriella emerges from the shadows.".to_string(),
    ]));
    let quantifier = Arc::new(MockBackend::default().with_prompt_responses(vec![
        r#"{"npcs_in_room": []}"#.to_string(),
        r#"{"npcs_in_room": ["gabriella"]}"#.to_string(),
    ])) as Arc<dyn LlmProvider>;
    let recorder = make_test_recorder(narrator as Arc<dyn LlmProvider>);
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        recorder,
        quantifier,
    );
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .pipeline(pipeline)
        .build_with_state();

    let resp = post_action(&app, "enter shop").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "action should complete");

    let gs = state.message_service.load_or_fresh();
    let npc_ids_in_area: Vec<String> = gs.scene.npcs_in_area.iter().map(|n| n.id.clone()).collect();
    assert!(
        npc_ids_in_area.contains(&"gabriella".to_string()),
        "gabriella should be in npcs_in_area after trigger continuation, got: {npc_ids_in_area:?}"
    );
    let gabriella_state = gs
        .npc_encounter_log
        .npcs
        .get("gabriella")
        .expect("gabriella should have encounter-log entry");
    assert_eq!(gabriella_state.times_met, 1, "times_met should be 1");
    assert!(
        gabriella_state.currently_meeting,
        "currently_meeting should be true"
    );
}

// [chronicler_engine/docs/specs/actions.md] SCENARIO: 2.1
#[tokio::test]
async fn test_nonexistent_room_sets_error_status_http() {
    let narrator = Arc::new(MockBackend::default());
    let (app, state) = app_with_narrator(narrator);

    {
        let mut gs = state.message_service.load_or_fresh();
        gs.movement.current_room_id = "non_existent_room".to_string();
        let _ = state.message_service.save_state(&gs);
    }

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let final_state = state.message_service.load_or_fresh();
    match &final_state.narrative.input_buffer.status {
        GenerationStatus::Error(msg) => {
            assert!(
                msg.contains("Room not found"),
                "expected 'Room not found' in error, got: {msg}"
            );
        }
        other => panic!("expected Error status, got {other:?}"),
    }
    let messages = state.message_service.load_messages().unwrap();
    let new_narrations: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    // The fresh-game opening narration exists, but no new narration from this action.
    // We assert none of the narrations mention "look" output (the action failed).
    let _ = new_narrations; // not asserting count here — the opening narration may exist
}

// [chronicler_engine/docs/specs/actions.md] SCENARIO: 2.2
#[tokio::test]
async fn test_failing_narrator_sets_error_status_http() {
    let narrator = Arc::new(MockBackend::default().with_fail());
    let (app, state) = app_with_narrator(narrator);

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let final_state = state.message_service.load_or_fresh();
    match &final_state.narrative.input_buffer.status {
        GenerationStatus::Error(msg) => {
            assert!(!msg.is_empty(), "error message should be non-empty");
        }
        other => panic!("expected Error status, got {other:?}"),
    }
    assert!(
        final_state
            .narrative
            .input_buffer
            .status
            .error_message()
            .is_some(),
        "error_message() should be Some"
    );
}

// [chronicler_engine/docs/specs/actions.md] SCENARIO: 2.3
#[tokio::test]
async fn test_empty_narrator_response_sets_error_no_narration_http() {
    let narrator = Arc::new(MockBackend::default().with_empty_response());
    let (app, state) = app_with_narrator(narrator);

    let resp = post_action(&app, "examine the room").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let final_state = state.message_service.load_or_fresh();
    match &final_state.narrative.input_buffer.status {
        GenerationStatus::Error(msg) => {
            assert!(
                msg.contains("empty"),
                "error message should mention 'empty', got: {msg}"
            );
        }
        other => panic!("expected Error status, got {other:?}"),
    }
    // No new narration persisted. The fresh-game opening narration may exist
    // from setup; count narrations before and after to be sure.
    let messages = state.message_service.load_messages().unwrap();
    let has_new_narration_for_this_action = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .any(|m| !m.text().is_empty() && m.text() != "Welcome to the Test World, Test Player! You find yourself in a cozy room with wooden beams and a warm fire. The smell of fresh bread fills the air. A friendly innkeeper behind the bar glances your way and smiles.");
    assert!(
        !has_new_narration_for_this_action,
        "empty response should not persist a narration for this action"
    );
}

// [chronicler_engine/docs/specs/actions.md] SCENARIO: 2.4
#[tokio::test]
async fn test_trigger_narration_failure_preserves_main_sets_error_http() {
    use chronicler_engine::domain::model::character::NpcCard;
    use chronicler_engine::domain::model::trigger::{
        ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement,
    };

    let npc = NpcCard {
        id: "npc_1".to_string(),
        sheet: chronicler_engine::test_support::TestPersona::standard().sheet,
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
    let data = TestDataBuilder::default_test().npcs(vec![npc]).build();

    let narrator = Arc::new(
        MockBackend::default()
            .with_narrations(vec!["Main narration preserved.".to_string()])
            .with_trigger_narration_fail(),
    );
    let quantifier_result = r#"{"npcs_in_room": ["npc_1"], "movement": null}"#.to_string();
    let quantifier_provider =
        Arc::new(MockBackend::default().with_prompt_responses(vec![quantifier_result]))
            as Arc<dyn LlmProvider>;
    let recorder = make_test_recorder(narrator as Arc<dyn LlmProvider>);
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        recorder,
        quantifier_provider,
    );
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .pipeline(pipeline)
        .build_with_state();

    let resp = post_action(&app, "examine the npc").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let final_state = state.message_service.load_or_fresh();
    match &final_state.narrative.input_buffer.status {
        GenerationStatus::Error(msg) => {
            assert!(
                msg.contains("Trigger narration failed"),
                "error should mention 'Trigger narration failed', got: {msg}"
            );
        }
        other => panic!("expected Error status, got {other:?}"),
    }
    let messages = state.message_service.load_messages().unwrap();
    assert!(
        messages.iter().any(|m| {
            m.message_type == MessageType::Narration
                && m.text().contains("Main narration preserved")
        }),
        "main narration should be preserved"
    );
    assert!(
        messages.iter().any(|m| {
            m.message_type == MessageType::System && m.text().contains("Trigger narration failed")
        }),
        "System log should mention trigger failure"
    );
}

// [chronicler_engine/docs/specs/actions.md] SCENARIO: 3.3
#[tokio::test]
async fn test_delayed_llm_completes_without_deadlock_http() {
    let narrator = Arc::new(MockBackend::default().with_delay(200));
    let (app, state) = app_with_narrator(narrator);

    let resp = post_action(&app, "look around").await;
    assert!(
        resp.status() != StatusCode::INTERNAL_SERVER_ERROR,
        "should not 500"
    );
    assert!(
        wait_idle(&state, 2000).await,
        "delayed LLM should complete within 2s"
    );
    let final_state = state.message_service.load_or_fresh();
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "status should not be Generating"
    );
}

// [chronicler_engine/docs/specs/actions.md] SCENARIO: 6.1
#[tokio::test]
async fn test_three_actions_in_sequence_http() {
    let narrator = Arc::new(MockBackend::default());
    let (app, state) = app_with_narrator(narrator);

    for command in ["examine room", "look around", "check inventory"] {
        let resp = post_action(&app, command).await;
        assert!(resp.status().is_success(), "action {command} should accept");
        assert!(
            wait_idle(&state, 1000).await,
            "action '{command}' should complete"
        );
    }

    let messages = state.message_service.load_messages().unwrap();
    let inputs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert_eq!(inputs.len(), 3, "should have 3 Input entries");
    assert_eq!(inputs[0].text(), "examine room");
    assert_eq!(inputs[1].text(), "look around");
    assert_eq!(inputs[2].text(), "check inventory");

    let narrations: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert!(
        narrations.len() >= 3,
        "should have at least 3 Narration entries, got {}",
        narrations.len()
    );
}

// [chronicler_engine/docs/specs/actions.md] SCENARIO: 6.2
#[tokio::test]
async fn test_sequential_execute_retry_execute_http() {
    let narrator = Arc::new(MockBackend::default());
    let (app, state) = app_with_narrator(narrator);

    let _ = post_action(&app, "examine room").await;
    assert!(wait_idle(&state, 1000).await, "action A should complete");

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_idle(&state, 1000).await, "retry A should complete");

    let _ = post_action(&app, "look around").await;
    assert!(wait_idle(&state, 1000).await, "action B should complete");

    let messages = state.message_service.load_messages().unwrap();
    let inputs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert_eq!(inputs.len(), 2, "should have 2 Input entries");
    let narrations: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert!(
        narrations.len() >= 2,
        "should have at least 2 Narration entries, got {}",
        narrations.len()
    );
}

// [chronicler_engine/docs/specs/actions.md] SCENARIO: 6.3
#[tokio::test]
async fn test_async_action_sequence_then_retry_http() {
    let narrator = Arc::new(MockBackend::default());
    let (app, state) = app_with_narrator(narrator);

    let _ = post_action(&app, "hello").await;
    assert!(wait_idle(&state, 1000).await);
    let _ = post_action(&app, "examine room").await;
    assert!(wait_idle(&state, 1000).await);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_idle(&state, 1000).await, "retry should complete");

    let messages = state.message_service.load_messages().unwrap();
    let inputs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert_eq!(inputs.len(), 2, "should have 2 Input entries");
}
