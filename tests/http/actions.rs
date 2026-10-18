//! HTTP E2E tests for the action endpoint (POST /action).

use std::sync::Arc;

use axum::http::StatusCode;

use chronicler_engine::domain::model::settings::{AppSettings, TextCheckMode, TextCheckSettings};
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::{
    active_game_of, analysis_report, ModelRole, ScriptedModel, TestAppBuilder, TestDataBuilder,
    TestMap, empty_sqlite_storage,
};

use crate::support::app_wiring::{app_with_model, app_with_model_and_settings};
use crate::support::http_assertions::{
    assert_story_log_ends_with_message, llm_call_count, status_error_message,
};
use crate::support::http_fixtures::OFFERED_SET;
use crate::support::http_requests::{
    fetch_body, fetch_generating_status, post_action, post_action_check, post_empty, post_form,
    response_body, wait_settled,
};

// [docs/specs/actions.md] SCENARIO: 1.1
#[tokio::test]
async fn test_action_succeeds_one_narration_idle_http() {
    let (app, state) = app_with_model(Arc::new(
        ScriptedModel::new().turn("You see a small wooden room."),
    ));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "action should complete");

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
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
    assert_eq!(
        fetch_generating_status(&app).await,
        "idle",
        "action should finish in the Idle state"
    );
}

// [docs/specs/actions.md] SCENARIO: 1.2
#[tokio::test]
async fn test_input_persisted_before_narration_http() {
    let (app, state) = app_with_model(Arc::new(
        ScriptedModel::new().turn("It is a small wooden room."),
    ));

    let resp = post_action(&app, "examine the room").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
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
    assert_eq!(
        fetch_generating_status(&app).await,
        "idle",
        "input persistence should finish in the Idle state"
    );
}

// [docs/specs/actions.md] SCENARIO: 1.4
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

    // The turn's own analysis and the event's reconciliation both report npc_1.
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(
            ScriptedModel::new()
                .narration("The main narration.")
                .analysis(analysis_report(&["npc_1"], None))
                .event_narration("The NPC greets you warmly.")
                .analysis(analysis_report(&["npc_1"], None)),
        ))
        .build_with_state();

    let resp = post_action(&app, "enter the shop").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
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
    assert_eq!(
        fetch_generating_status(&app).await,
        "idle",
        "trigger action should finish in the Idle state"
    );
}

// [docs/specs/actions.md] SCENARIO: 1.7
#[tokio::test]
async fn test_no_trigger_npc_produces_narration_no_event_http() {
    use chronicler_engine::test_support::TestNpc;

    let npc = TestNpc::named("bartender", "Bartender");
    let data = TestDataBuilder::default_test().npcs(vec![npc]).build();

    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(
            ScriptedModel::new()
                .narration("The bartender nods.")
                .analysis(analysis_report(&["bartender"], None)),
        ))
        .build_with_state();

    let resp = post_action(&app, "talk to bartender").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "action should complete");

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let narrations: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert!(!narrations.is_empty(), "should have at least one Narration");
    for n in &narrations {
        assert!(
            n.event_header().is_none(),
            "no trigger should fire (no triggers defined); got event_header = {:?}",
            n.event_header()
        );
    }
    assert_eq!(
        fetch_generating_status(&app).await,
        "idle",
        "no-trigger action should finish in the Idle state"
    );
}

// [docs/specs/actions.md] SCENARIO: 1.8
#[tokio::test]
async fn test_trigger_does_not_refire_on_second_encounter_http() {
    use chronicler_engine::domain::model::trigger::ComparisonOperator;
    use chronicler_engine::test_support::TestNpc;

    let shopkeeper = TestNpc::with_times_met_trigger(
        "shopkeeper",
        "Shopkeeper Sarah",
        ComparisonOperator::Eq,
        0,
    );
    let data = TestDataBuilder::default_test()
        .npcs(vec![shopkeeper])
        .build();

    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(
            ScriptedModel::new()
                .narration("The shopkeeper greets you.")
                .analysis(analysis_report(&["shopkeeper"], None))
                .event_narration("The shopkeeper greets you.")
                .analysis(analysis_report(&["shopkeeper"], None))
                .narration("The shopkeeper greets you again.")
                .analysis(analysis_report(&["shopkeeper"], None)),
        ))
        .build_with_state();

    let resp = post_action(&app, "talk to shopkeeper").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "first action should complete"
    );

    let resp = post_action(&app, "talk to shopkeeper").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "second action should complete"
    );

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let trigger_narrations = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .filter(|m| m.event_header().is_some())
        .count();
    assert!(
        trigger_narrations <= 1,
        "one-shot trigger should fire at most once, got {trigger_narrations} trigger narrations"
    );
    assert_eq!(
        fetch_generating_status(&app).await,
        "idle",
        "second encounter should finish in the Idle state"
    );
}

// [docs/specs/actions.md] SCENARIO: 1.5
#[tokio::test]
async fn test_empty_command_continuation_no_input_http() {
    let (app, state) = app_with_model(Arc::new(ScriptedModel::new().turn("The scene continues.")));

    let resp = post_action(&app, "").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
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
    assert_eq!(
        fetch_generating_status(&app).await,
        "idle",
        "continuation should finish in the Idle state"
    );
}

// [docs/specs/actions.md] SCENARIO: 1.6
#[tokio::test]
async fn test_trigger_continuation_reruns_quantifier_detects_new_npc_http() {
    use chronicler_engine::domain::model::trigger::ComparisonOperator;
    use chronicler_engine::test_support::TestNpc;

    // evaluate_triggers iterates every world NPC, not just npcs_in_area, so the
    // shopkeeper's times_met trigger fires the continuation.
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

    // The turn's analysis reports nobody; the event's reconciliation reports
    // Gabriella, the Character the trigger's narration brings in.
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(
            ScriptedModel::new()
                .narration("You step into the shop.")
                .analysis(analysis_report(&[], None))
                .event_narration("Gabriella emerges from the shadows.")
                .analysis(analysis_report(&["gabriella"], None)),
        ))
        .build_with_state();

    let resp = post_action(&app, "enter shop").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "action should complete");

    let debug = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed");
    assert!(
        debug.npcs_in_area.contains(&"gabriella".to_string()),
        "gabriella should be in npcs_in_area after trigger continuation, got: {:?}",
        debug.npcs_in_area
    );
    let gabriella_state = debug
        .npc_encounter_log
        .get("gabriella")
        .expect("gabriella should have encounter-log entry");
    assert_eq!(gabriella_state.times_met, 1, "times_met should be 1");
    assert!(
        gabriella_state.currently_meeting,
        "currently_meeting should be true"
    );
}

// [docs/specs/actions.md] SCENARIO: 2.1
#[tokio::test]
async fn test_nonexistent_room_sets_error_status_http() {
    // The room lookup fails before the analysis call, so this script omits the
    // analysis.
    let (app, state) = app_with_model(Arc::new(ScriptedModel::new().narration("You look around.")));

    {
        let mut gs = state.message_service.load_or_fresh(active_game_of(&state));
        gs.swipe.movement.current_room_id = "non_existent_room".to_string();
        let _ = state
            .message_service
            .save_state(active_game_of(&state), &gs);
    }

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("Room not found"),
        "expected 'Room not found' in the status, got: {status}"
    );
    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    assert!(
        !messages
            .iter()
            .any(|m| m.message_type == MessageType::Narration),
        "the action in an invalid room must append no Narration: {messages:?}"
    );
}

// [docs/specs/actions.md] SCENARIO: 2.2
#[tokio::test]
async fn test_failing_narrator_sets_error_status_http() {
    let (app, state) = app_with_model(Arc::new(ScriptedModel::new().failed(ModelRole::Narration)));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

    let status = fetch_generating_status(&app).await;
    let message = status_error_message(&status);
    assert!(!message.is_empty(), "error message should be non-empty");
}

// [docs/specs/actions.md] SCENARIO: 2.3
#[tokio::test]
async fn test_empty_narrator_response_sets_error_no_narration_http() {
    let (app, state) = app_with_model(Arc::new(ScriptedModel::new().empty(ModelRole::Narration)));

    let resp = post_action(&app, "examine the room").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("empty"),
        "error message should mention 'empty', got: {status}"
    );
    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let has_new_narration_for_this_action = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .any(|m| !m.text().is_empty() && m.text() != "Welcome to the Test World, Test Player! You find yourself in a cozy room with wooden beams and a warm fire. The smell of fresh bread fills the air. A friendly innkeeper behind the bar glances your way and smiles.");
    assert!(
        !has_new_narration_for_this_action,
        "empty response should not persist a narration for this action"
    );
}

// [docs/specs/actions.md] SCENARIO: 2.4
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

    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(
            ScriptedModel::new()
                .narration("Main narration preserved.")
                .analysis(analysis_report(&["npc_1"], None))
                .failed(ModelRole::Event),
        ))
        .build_with_state();

    let resp = post_action(&app, "examine the npc").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("Trigger narration failed"),
        "error should mention 'Trigger narration failed', got: {status}"
    );
    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    assert!(
        messages.iter().any(|m| {
            m.message_type == MessageType::Narration
                && m.text().contains("Main narration preserved")
        }),
        "main narration should be preserved"
    );
    assert_eq!(
        messages.len(),
        messages
            .iter()
            .filter(|m| m.is_input_or_narration())
            .count(),
        "a failed trigger must not append a story-log entry: {messages:?}"
    );
    assert!(
        !messages.iter().any(|m| {
            m.message_type == MessageType::System && m.text().contains("Trigger narration failed")
        }),
        "the engine error must not be written into the story log: {messages:?}"
    );
    assert_eq!(
        messages.last().map(|m| m.message_type.clone()),
        Some(MessageType::Narration),
        "the main narration must stay the last entry: {messages:?}"
    );

    let story_log = fetch_body(&app, "/fragment/story-log").await;
    assert!(
        story_log.contains("retrigger-btn"),
        "the failed trigger must leave Retrigger available: {story_log}"
    );
    assert_story_log_ends_with_message(
        &story_log,
        messages
            .last()
            .expect("the main narration survives the failed trigger")
            .id,
        "the main narration must be the last rendered entry",
    );
}

// [docs/specs/actions.md] SCENARIO: 2.5
#[tokio::test]
async fn test_quantifier_transport_failure_completes_action_and_keeps_npcs_http() {
    // The quantifier retries once, so a failed analysis fails twice.
    assert_quantifier_failure_keeps_npcs(Arc::new(
        ScriptedModel::new()
            .narration("The innkeeper glances your way.")
            .failed(ModelRole::Analysis)
            .failed(ModelRole::Analysis),
    ))
    .await;
}

// [docs/specs/actions.md] SCENARIO: 2.5
#[tokio::test]
async fn test_quantifier_unparseable_answer_completes_action_and_keeps_npcs_http() {
    // The quantifier retries once, so an unparseable answer is read twice.
    assert_quantifier_failure_keeps_npcs(Arc::new(
        ScriptedModel::new()
            .narration("The innkeeper glances your way.")
            .analysis("There is no room report in this answer.")
            .analysis("There is no room report in this answer."),
    ))
    .await;
}

async fn assert_quantifier_failure_keeps_npcs(model: Arc<ScriptedModel>) {
    let (app, state) = app_with_model(Arc::clone(&model));

    let npcs_before = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed")
        .npcs_in_area;
    assert!(
        !npcs_before.is_empty(),
        "the setup must start from a non-empty NPC set: {npcs_before:?}"
    );

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "a quantifier failure must not stop the action"
    );

    assert_eq!(
        fetch_generating_status(&app).await,
        "idle",
        "the action must finish in the Idle state"
    );
    let npcs_after = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed")
        .npcs_in_area;
    assert_eq!(
        npcs_after, npcs_before,
        "the scene must keep its prior NPC set after a quantifier failure"
    );
    assert_eq!(
        llm_call_count(&state, "quantifier"),
        2,
        "the analysis retries once before it falls back"
    );
}

// [docs/specs/actions.md] SCENARIO: 3.3
#[tokio::test]
async fn test_delayed_llm_completes_without_deadlock_http() {
    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let hold = model.hold(ModelRole::Narration);
    let (app, state) = app_with_model(Arc::clone(&model));

    let resp = post_action(&app, "look around").await;
    assert!(
        resp.status() != StatusCode::INTERNAL_SERVER_ERROR,
        "should not 500"
    );
    assert!(
        hold.wait_started(1000),
        "the held narration call should reach the model"
    );
    hold.release();
    assert!(
        wait_settled(&state, 2000).await,
        "a held LLM call should complete once released"
    );
    assert_eq!(
        fetch_generating_status(&app).await,
        "idle",
        "status should not be Generating"
    );
}

// [docs/specs/actions.md] SCENARIO: 6.1
#[tokio::test]
async fn test_three_actions_in_sequence_http() {
    let (app, state) = app_with_model(Arc::new(
        ScriptedModel::new()
            .turn("You examine the room.")
            .narration("You look around.")
            .analysis(analysis_report(&[], None))
            .narration("You check your inventory.")
            .analysis(analysis_report(&[], None)),
    ));

    for command in ["examine room", "look around", "check inventory"] {
        let resp = post_action(&app, command).await;
        assert!(resp.status().is_success(), "action {command} should accept");
        assert!(
            wait_settled(&state, 1000).await,
            "action '{command}' should complete"
        );
    }

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
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

// [docs/specs/actions.md] SCENARIO: 6.2
#[tokio::test]
async fn test_sequential_execute_retry_execute_http() {
    let (app, state) = app_with_model(Arc::new(
        ScriptedModel::new()
            .turn("You examine the room.")
            .narration("You examine the room again.")
            .analysis(analysis_report(&[], None))
            .narration("You look around.")
            .analysis(analysis_report(&[], None)),
    ));

    let _ = post_action(&app, "examine room").await;
    assert!(wait_settled(&state, 1000).await, "action A should complete");

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 1000).await, "retry A should complete");

    let _ = post_action(&app, "look around").await;
    assert!(wait_settled(&state, 1000).await, "action B should complete");

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
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

// [docs/specs/actions.md] SCENARIO: 6.3
#[tokio::test]
async fn test_async_action_sequence_then_retry_http() {
    let (app, state) = app_with_model(Arc::new(
        ScriptedModel::new()
            .turn("You greet the room.")
            .narration("You examine the room.")
            .analysis(analysis_report(&[], None))
            .narration("You examine the room again.")
            .analysis(analysis_report(&[], None)),
    ));

    let _ = post_action(&app, "hello").await;
    assert!(wait_settled(&state, 1000).await);
    let _ = post_action(&app, "examine room").await;
    assert!(wait_settled(&state, 1000).await);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 1000).await, "retry should complete");

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let inputs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert_eq!(inputs.len(), 2, "should have 2 Input entries");
}

// [docs/specs/actions.md] SCENARIO: 1.9
#[tokio::test]
async fn test_slash_impersonate_produces_input_http() {
    let (app, state) = app_with_model(Arc::new(ScriptedModel::new().turn("You say hello.")));

    let resp = post_action(&app, "/impersonate hello").await;
    assert!(resp.status().is_success(), "/impersonate should accept");
    assert!(
        wait_settled(&state, 1000).await,
        "impersonate action should complete"
    );

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let inputs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert_eq!(
        inputs.len(),
        1,
        "impersonate should produce exactly one Input entry"
    );

    let raw_command: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Input && m.text().contains("/impersonate hello"))
        .collect();
    assert_eq!(
        raw_command.len(),
        0,
        "impersonate should not persist the raw command as an Input entry"
    );

    let input_entry = inputs.first().expect("exactly one Input entry");
    assert!(
        input_entry.impersonated(),
        "the impersonate flag must be stored on the swipe"
    );
    assert_eq!(
        input_entry.impersonate_direction(),
        Some("hello"),
        "the impersonate direction must be stored on the swipe"
    );
}

// [docs/specs/actions.md] SCENARIO: 1.10
#[tokio::test]
async fn test_slash_guide_does_not_persist_input_http() {
    let (app, state) = app_with_model(Arc::new(ScriptedModel::new().turn("You look around.")));

    let resp = post_action(&app, "/guide look around").await;
    assert!(resp.status().is_success(), "/guide should accept");
    assert!(
        wait_settled(&state, 1000).await,
        "guide action should complete"
    );

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let narrations: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert!(
        !narrations.is_empty(),
        "guide should produce at least one Narration entry"
    );

    let inputs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert_eq!(inputs.len(), 0, "guide should not persist an Input entry");

    let narration = narrations.first().expect("at least one Narration entry");
    assert_eq!(
        narration.guide_text(),
        Some("look around"),
        "the guide must be stored on the swipe"
    );
    assert!(
        !narration.impersonated(),
        "guide inputs must not set the impersonate flag"
    );
}

// [docs/specs/actions.md] SCENARIO: 1.12
#[tokio::test]
async fn test_slash_command_bypasses_text_check_http() {
    let settings = AppSettings {
        text_check: TextCheckSettings {
            mode: TextCheckMode::Spell,
            enable_auto_check: true,
            ignored_words: vec![],
        },
        ..Default::default()
    };
    let (app, state) = app_with_model_and_settings(
        Arc::new(
            ScriptedModel::new()
                .turn("You look at the castle.")
                .options(OFFERED_SET),
        ),
        settings,
    );

    let resp = post_action_check(&app, "/guide look at the casle").await;
    assert!(resp.status().is_success());
    assert!(
        resp.headers().get("HX-Retarget").is_some(),
        "steering command should dispatch directly (HX-Retarget set)"
    );
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        !body_str.contains("text-check-preview"),
        "steering command must bypass the text check: {body_str}"
    );
    assert!(
        wait_settled(&state, 1000).await,
        "guide action should complete"
    );

    // The /guide leg above already created scene history, so this /options
    // dispatch passes the empty-history validation.
    let resp = post_action_check(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(
        resp.headers().get("HX-Retarget").is_some(),
        "/options is an engine command and should dispatch directly (HX-Retarget set)"
    );
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        !body_str.contains("text-check-preview"),
        "/options must bypass the text check: {body_str}"
    );
    assert!(
        wait_settled(&state, 1000).await,
        "/options action should complete"
    );

    let resp = post_action_check(&app, "look at the casle").await;
    assert!(resp.status().is_success());
    assert!(
        resp.headers().get("HX-Retarget").is_none(),
        "plain input should show a preview, not dispatch (no HX-Retarget)"
    );
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("text-check-preview"),
        "plain input should surface the text-check preview: {body_str}"
    );
}

// [docs/specs/actions.md] SCENARIO: 1.14
#[tokio::test]
async fn test_moving_narration_reports_the_room_it_arrives_in_http() {
    let storage = Arc::new(empty_sqlite_storage());
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .build();
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .storage(Arc::clone(&storage))
        .model(Arc::new(
            ScriptedModel::new()
                .narration("You walk north into the next room.")
                .analysis(analysis_report(&[], Some("room2")))
                .narration("You look around the new room.")
                .analysis(analysis_report(&[], None)),
        ))
        .build_with_state();

    assert!(post_action(&app, "walk north").await.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "the Action that moves the player should complete"
    );

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let narration = messages
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .expect("the Action appends a Narration");
    assert_eq!(
        narration.location_header(),
        Some("Room room2"),
        "the Narration that moved the player must report the room it arrived in"
    );
    let input = messages
        .iter()
        .find(|m| m.message_type == MessageType::Input)
        .expect("the Action appends a player input");
    assert_eq!(
        input.location_header(),
        None,
        "the player input that leaves a room must not carry the old room's name"
    );

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "the later, stationary Action should complete"
    );

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let later: Vec<_> = messages
        .iter()
        .filter(|m| m.is_input_or_narration())
        .skip(2)
        .collect();
    assert_eq!(
        later.len(),
        2,
        "the second Action adds a player input and a Narration"
    );
    assert!(
        later.iter().all(|m| m.location_header().is_none()),
        "the room name must not leak onto the Action after the movement: {:?}",
        later
            .iter()
            .map(|m| m.location_header())
            .collect::<Vec<_>>()
    );
}

// [docs/specs/actions.md] SCENARIO: 1.13
#[tokio::test]
async fn test_confirm_retargets_status_display() {
    let (app, state) = app_with_model(Arc::new(ScriptedModel::new().turn("You look around.")));

    let resp = post_form(&app, "/action/confirm", "command=look").await;
    assert!(resp.status().is_success());
    assert_eq!(
        resp.headers().get("HX-Retarget"),
        Some(&axum::http::HeaderValue::from_static("#status-display")),
        "confirm must retarget the status display"
    );
    let body = response_body(resp).await;
    assert!(
        body.starts_with("<span class=\"status"),
        "confirm must return a bare status span: {body}"
    );
    assert!(
        !body.contains("id=\"action-area\"") && !body.contains("command-form"),
        "confirm must not replace the action area or the command form: {body}"
    );
    assert!(wait_settled(&state, 1000).await, "confirm should complete");
}
