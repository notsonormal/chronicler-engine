//! HTTP E2E tests for the action endpoint (POST /action).

use std::sync::Arc;

use axum::http::StatusCode;

use chronicler_engine::adapters::driven::storage::{StorageCall, TestOverride};
use chronicler_engine::adapters::driving::http::AppState;

use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::domain::model::settings::{AppSettings, TextCheckMode, TextCheckSettings};
use chronicler_engine::domain::model::map::Room;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::{
    active_game_of, analysis_report, ScriptedModel, TestAppBuilder, TestDataBuilder, TestMap,
    empty_sqlite_storage,
};

use crate::support::app_wiring::{app_with_model, app_with_model_and_settings};
use crate::support::http_assertions::{
    assert_story_log_ends_with_message, llm_call_count, status_error_message,
};
use crate::support::http_fixtures::{
    break_start_room, corrupt_start_state, room_header_of, story_of, OFFERED_SET,
};
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
    assert!(wait_settled(&state, 2000).await, "action should complete");

    let messages = story_of(&state);
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
    assert_eq!(
        narrations.len(),
        2,
        "the opening Narration and the Action's Narration"
    );
    assert_eq!(
        narrations.last().map(|m| m.text()),
        Some("You see a small wooden room.")
    );
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
    assert!(wait_settled(&state, 2000).await);

    let messages = story_of(&state);
    let input_idx = messages
        .iter()
        .position(|m| m.message_type == MessageType::Input)
        .expect("Input should exist");
    let narration_idx = messages
        .iter()
        .rposition(|m| m.message_type == MessageType::Narration)
        .expect("Narration should exist");
    assert!(
        input_idx < narration_idx,
        "Input should appear before the Action's Narration"
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

    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(
            ScriptedModel::new()
                .narration("The main narration.")
                .after_narration(analysis_report(&["npc_1"], None))
                .event_narration("The NPC greets you warmly.")
                .after_narration(analysis_report(&["npc_1"], None)),
        ))
        .build_with_state();

    let resp = post_action(&app, "enter the shop").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 2000).await);

    let messages = story_of(&state);
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
                .after_narration(analysis_report(&["bartender"], None)),
        ))
        .build_with_state();

    let resp = post_action(&app, "talk to bartender").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 2000).await, "action should complete");

    let messages = story_of(&state);
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
                .after_narration(analysis_report(&["shopkeeper"], None))
                .event_narration("The shopkeeper greets you.")
                .after_narration(analysis_report(&["shopkeeper"], None))
                .narration("The shopkeeper greets you again.")
                .after_narration(analysis_report(&["shopkeeper"], None)),
        ))
        .build_with_state();

    let resp = post_action(&app, "talk to shopkeeper").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "first action should complete"
    );

    let resp = post_action(&app, "talk to shopkeeper").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "second action should complete"
    );

    let messages = story_of(&state);
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
    assert!(wait_settled(&state, 2000).await);

    let messages = story_of(&state);
    let inputs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert!(inputs.is_empty(), "empty input should not add Input");
    let narrations: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert_eq!(
        narrations.len(),
        2,
        "the opening Narration and the continuation"
    );
    assert_eq!(
        narrations.last().map(|m| m.text()),
        Some("The scene continues.")
    );
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

    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(
            ScriptedModel::new()
                .narration("You step into the shop.")
                .after_narration(analysis_report(&[], None))
                .event_narration("Gabriella emerges from the shadows.")
                .after_narration(analysis_report(&["gabriella"], None)),
        ))
        .build_with_state();

    let resp = post_action(&app, "enter shop").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 2000).await, "action should complete");

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
    let storage = Arc::new(empty_sqlite_storage());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .model(Arc::new(ScriptedModel::new().narration("You look around.")))
        .build_with_state();
    break_start_room(&storage, active_game_of(&state), "non_existent_room");

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 2000).await);

    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("The room non_existent_room is not on the map."),
        "the failure names the invalid room, got: {status}"
    );
    let narrations = story_of(&state)
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .count();
    assert_eq!(
        narrations, 1,
        "the Action in an invalid room appends no Narration beyond the opening"
    );
}

// [docs/specs/actions.md] SCENARIO: 2.2
#[tokio::test]
async fn test_failing_narrator_sets_error_status_http() {
    let (app, state) = app_with_model(Arc::new(ScriptedModel::new().failed(Role::Narrator)));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 2000).await);

    let status = fetch_generating_status(&app).await;
    let message = status_error_message(&status);
    assert!(!message.is_empty(), "error message should be non-empty");
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
                .after_narration(analysis_report(&["npc_1"], None))
                .failed(Role::Trigger),
        ))
        .build_with_state();

    let resp = post_action(&app, "examine the npc").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 2000).await);

    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("The event narration failed."),
        "the failure line of a failed event narration, got: {status}"
    );
    assert!(
        status.contains("answer could not be read."),
        "the disclosure holds the narrator call's own line, got: {status}"
    );
    let messages = story_of(&state);
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
            m.message_type == MessageType::System
                && m.text().contains("The event narration failed.")
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
            .failed(Role::Quantifier)
            .failed(Role::Quantifier),
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
            .after_narration("There is no room report in this answer.")
            .after_narration("There is no room report in this answer."),
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
        wait_settled(&state, 2000).await,
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
    let hold = model.hold(Role::Narrator);
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
            .after_narration(analysis_report(&[], None))
            .narration("You check your inventory.")
            .after_narration(analysis_report(&[], None)),
    ));

    for command in ["examine room", "look around", "check inventory"] {
        let resp = post_action(&app, command).await;
        assert!(resp.status().is_success(), "action {command} should accept");
        assert!(
            wait_settled(&state, 2000).await,
            "action '{command}' should complete"
        );
    }

    let messages = story_of(&state);
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
            .after_narration(analysis_report(&[], None))
            .narration("You look around.")
            .after_narration(analysis_report(&[], None)),
    ));

    let _ = post_action(&app, "examine room").await;
    assert!(wait_settled(&state, 2000).await, "action A should complete");

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 2000).await, "retry A should complete");

    let _ = post_action(&app, "look around").await;
    assert!(wait_settled(&state, 2000).await, "action B should complete");

    let messages = story_of(&state);
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
            .after_narration(analysis_report(&[], None))
            .narration("You examine the room again.")
            .after_narration(analysis_report(&[], None)),
    ));

    let _ = post_action(&app, "hello").await;
    assert!(wait_settled(&state, 2000).await);
    let _ = post_action(&app, "examine room").await;
    assert!(wait_settled(&state, 2000).await);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 2000).await, "retry should complete");

    let messages = story_of(&state);
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
        wait_settled(&state, 2000).await,
        "impersonate action should complete"
    );

    let messages = story_of(&state);
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

// [docs/specs/actions.md] SCENARIO: 1.18
#[tokio::test]
async fn test_bare_slash_impersonate_produces_input_http() {
    let (app, state) = app_with_model(Arc::new(ScriptedModel::new().turn("You say hello.")));

    let resp = post_action(&app, "/impersonate").await;
    assert!(
        resp.status().is_success(),
        "a bare /impersonate should accept"
    );
    assert!(
        wait_settled(&state, 2000).await,
        "the bare impersonate should complete"
    );

    let messages = story_of(&state);
    let inputs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert_eq!(inputs.len(), 1, "one Input entry: {messages:?}");
    let input = inputs[0];
    assert_eq!(input.text(), "You say hello.");
    assert!(input.impersonated(), "the impersonate flag is stored");
    assert_eq!(
        input.impersonate_direction(),
        None,
        "a bare /impersonate stores no direction"
    );
    assert_eq!(fetch_generating_status(&app).await, "idle");
}

// [docs/specs/actions.md] SCENARIO: 1.10
#[tokio::test]
async fn test_slash_guide_does_not_persist_input_http() {
    let (app, state) = app_with_model(Arc::new(ScriptedModel::new().turn("You look around.")));

    let resp = post_action(&app, "/guide look around").await;
    assert!(resp.status().is_success(), "/guide should accept");
    assert!(
        wait_settled(&state, 2000).await,
        "guide action should complete"
    );

    let messages = story_of(&state);
    let narrations: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert_eq!(
        narrations.len(),
        2,
        "the guide appends its Narration after the opening"
    );

    let inputs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert_eq!(inputs.len(), 0, "guide should not persist an Input entry");

    let narration = narrations
        .last()
        .expect("the guide appends a Narration after the opening");
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
        wait_settled(&state, 2000).await,
        "guide action should complete"
    );

    // The /guide leg above already wrote story text, so this /options dispatch
    // passes the check that needs a story with text to work from.
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
        wait_settled(&state, 2000).await,
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
async fn test_moving_narration_shows_the_room_it_arrives_in_http() {
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
                .after_narration(analysis_report(&[], Some("room2")))
                .narration("You look around the new room.")
                .after_narration(analysis_report(&[], None)),
        ))
        .build_with_state();

    assert!(post_action(&app, "walk north").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the Action that moves the player should complete"
    );

    let messages = story_of(&state);
    let narration = messages
        .iter()
        .rfind(|m| m.message_type == MessageType::Narration)
        .expect("the Action appends a Narration");
    assert_eq!(
        room_header_of(&state, narration.id).as_deref(),
        Some("Room room2"),
        "the Narration that moved the player shows the room it arrived in"
    );
    let input = messages
        .iter()
        .find(|m| m.message_type == MessageType::Input)
        .expect("the Action appends a player input");
    assert_eq!(
        room_header_of(&state, input.id),
        None,
        "the player input that leaves a room shows no room"
    );

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the later, stationary Action should complete"
    );

    let messages = story_of(&state);
    let later: Vec<_> = messages
        .iter()
        .filter(|m| m.is_input_or_narration())
        .skip(3)
        .collect();
    assert_eq!(
        later.len(),
        2,
        "the second Action adds a player input and a Narration"
    );
    let later_headers: Vec<_> = later.iter().map(|m| room_header_of(&state, m.id)).collect();
    assert!(
        later_headers.iter().all(Option::is_none),
        "the room name must not repeat on the Action after the movement: {later_headers:?}"
    );
}

fn last_narration_id(state: &AppState) -> u64 {
    story_of(state)
        .iter()
        .rfind(|m| m.message_type == MessageType::Narration)
        .expect("the Action appends a Narration")
        .id
}

fn current_room_id(state: &AppState) -> String {
    state
        .game_view_query
        .get_debug_state(active_game_of(state))
        .expect("get_debug_state should succeed")
        .current_room_id
}

// [docs/specs/actions.md] SCENARIO: 1.15
#[tokio::test]
async fn test_narration_that_names_the_current_room_shows_no_room_http() {
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .build();
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(
            ScriptedModel::new()
                .narration("You look around.")
                .after_narration(analysis_report(&[], Some("room_1"))),
        ))
        .build_with_state();

    assert!(post_action(&app, "look around").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the Action should complete"
    );

    assert_eq!(current_room_id(&state), "room_1", "the player did not move");
    assert_eq!(
        room_header_of(&state, last_narration_id(&state)),
        None,
        "naming the room the player is in changes no room name"
    );
}

// [docs/specs/actions.md] SCENARIO: 1.16
#[tokio::test]
async fn test_two_added_rooms_with_the_same_name_show_one_room_name_http() {
    let storage = Arc::new(empty_sqlite_storage());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .model(Arc::new(
            ScriptedModel::new()
                .narration("You walk on.")
                .after_narration(analysis_report(&[], Some("Misty Glade"))),
        ))
        .build_with_state();
    let seeded_room = Room {
        id: "dynamic_seeded".to_string(),
        ..Room::new_dynamic("Misty Glade", "A glade the player already stands in.")
    };
    corrupt_start_state(&storage, active_game_of(&state), |start| {
        start
            .movement
            .dynamic_rooms
            .insert(seeded_room.id.clone(), seeded_room.clone());
        start.movement.current_room_id = seeded_room.id.clone();
    });

    assert!(post_action(&app, "walk on").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the Action should complete"
    );

    let room_id = current_room_id(&state);
    assert!(
        room_id.starts_with("dynamic_") && room_id != seeded_room.id,
        "the move added a second room: {room_id}"
    );
    let (room_name, _image) = state
        .game_view_query
        .get_current_room_view(active_game_of(&state))
        .expect("the current room reads back");
    assert_eq!(
        room_name, "Misty Glade",
        "the second room carries the same name"
    );
    assert_eq!(
        room_header_of(&state, last_narration_id(&state)),
        None,
        "two added rooms with one name show one room name"
    );
}

// [docs/specs/actions.md] SCENARIO: 1.17
#[tokio::test]
async fn test_a_move_between_two_map_rooms_with_the_same_name_shows_no_room_http() {
    let mut map = TestMap::two_rooms("room_1", "room2");
    let shared_name = map.overworld.regions[0].rooms[0].name.clone();
    map.overworld.regions[0].rooms[1].name = shared_name;
    let data = TestDataBuilder::default_test().map(map).build();
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(
            ScriptedModel::new()
                .narration("You walk north.")
                .after_narration(analysis_report(&[], Some("room2"))),
        ))
        .build_with_state();

    assert!(post_action(&app, "walk north").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the Action should complete"
    );

    assert_eq!(current_room_id(&state), "room2", "the player moved");
    assert_eq!(
        room_header_of(&state, last_narration_id(&state)),
        None,
        "the two rooms share a name, so the move shows no room"
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
    assert!(wait_settled(&state, 2000).await, "confirm should complete");
}

// [docs/specs/actions.md] SCENARIO: 2.6
#[tokio::test]
async fn test_missing_persona_answers_500_with_the_missing_data_line_http() {
    let raw = "the persona record cannot be read";
    let line = "The data for this request is missing: the persona.";
    let storage = Arc::new(
        empty_sqlite_storage().with_failure(StorageCall::GetPersona, TestOverride::internal(raw)),
    );
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .model(Arc::new(ScriptedModel::new().turn("You look around.")))
        .build_with_state();
    let before = story_of(&state).len();

    let resp = post_action(&app, "look").await;
    assert_eq!(
        resp.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "an Action whose needed data is missing is not a user error"
    );
    let body = response_body(resp).await;
    assert!(
        body.contains(line),
        "the answer carries the missing-data line that names the record: {body}"
    );
    assert!(
        !body.contains(raw),
        "the unreadable error text stays out of the answer: {body}"
    );

    let after = story_of(&state).len();
    assert_eq!(
        after, before,
        "the refused Action gains no Message, so the story log keeps its entries"
    );

    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains(line),
        "the Game carries the same missing-data line: {status}"
    );
    let pre_start = status
        .find(r#"<pre class="error-detail-raw">"#)
        .expect("the disclosure holds the unreadable error");
    let pre_end = status[pre_start..]
        .find("</pre>")
        .expect("the disclosure's raw block closes")
        + pre_start;
    assert_eq!(
        status.matches(raw).count(),
        1,
        "the unreadable error text appears once: {status}"
    );
    assert!(
        status
            .find(raw)
            .is_some_and(|at| at > pre_start && at < pre_end),
        "the unreadable error text belongs inside the disclosure: {status}"
    );
}
