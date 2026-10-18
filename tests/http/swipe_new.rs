//! HTTP E2E tests for the retry endpoint (POST /swipe/new).

use std::sync::Arc;

use axum::http::StatusCode;
use tower::util::ServiceExt;

use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::domain::model::message::Message;
use chronicler_engine::domain::model::settings::AppSettings;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::state::game_state_snapshot::GameStateSnapshot;
use chronicler_engine::test_support::{
    active_game_of, analysis_report, configured_runner, insert_message_with_swipe, seed_event_flow,
    seed_main_narration_flow, ModelRole, ScriptedModel, TestAppBuilder, TestDataBuilder, TestMap,
    TestStoredTriggerContext, empty_sqlite_storage,
};

use crate::support::app_wiring::{app_with_model, app_with_model_and_storage};
use crate::support::http_assertions::status_error_message;
use crate::support::http_requests::{fetch_generating_status, post_action, post_empty, wait_settled};

fn story_log_counts(state: &AppState) -> (usize, usize) {
    let messages = state
        .message_service
        .load_messages(active_game_of(state))
        .expect("the story log reads back");
    let count = |kind: MessageType| messages.iter().filter(|m| m.message_type == kind).count();
    (count(MessageType::Input), count(MessageType::Narration))
}

async fn app_with_an_event_that_meets_gabriella() -> (axum::Router, AppState, u64) {
    use chronicler_engine::domain::model::trigger::ComparisonOperator;
    use chronicler_engine::test_support::TestNpc;

    let storage = Arc::new(empty_sqlite_storage());
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .npcs(vec![TestNpc::with_room_scoped_trigger(
            "gabriella",
            "Gabriella",
            ComparisonOperator::Eq,
            0,
            "room2",
        )])
        .build();
    // Consumed in call order: the Action's narration and its post-generation
    // analysis, the event's narration and its own reconciliation, then the
    // event retry's pair.
    let model = Arc::new(
        ScriptedModel::new()
            .narration("You walk into the next room.")
            .analysis(analysis_report(&[], Some("room2")))
            .event_narration("Gabriella is here.")
            .analysis(analysis_report(&["gabriella"], None))
            .event_narration("Gabriella is here again.")
            .analysis(analysis_report(&[], None)),
    );
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .storage(Arc::clone(&storage))
        .model(model)
        .build_with_state();

    let resp = post_action(&app, "walk around").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "the trigger turn should complete"
    );

    let event_id = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap()
        .iter()
        .find(|m| m.event_header().is_some())
        .map(|m| m.id)
        .expect("the trigger turn appends an event narration");
    (app, state, event_id)
}

fn encounter_times_met(state: &AppState, npc_id: &str) -> u32 {
    state
        .game_view_query
        .get_debug_state(active_game_of(state))
        .expect("get_debug_state should succeed")
        .npc_encounter_log
        .get(npc_id)
        .map(|encounter| encounter.times_met)
        .unwrap_or(0)
}

// [docs/specs/swipe_new.md] SCENARIO: 9.1
#[tokio::test]
async fn test_retry_replaces_narration_with_new_swipe_http() {
    let (app, state) = app_with_model(Arc::new(
        ScriptedModel::new()
            .turn("First.")
            .narration("Second.")
            .analysis(analysis_report(&[], None)),
    ));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "action should complete");

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 1000).await, "retry should complete");

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let narrations: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert_eq!(narrations.len(), 1, "exactly one Narration message");
    assert_eq!(narrations[0].swipes.len(), 2, "retry appended a swipe");
    assert_eq!(narrations[0].text(), "Second.", "active swipe is the retry");
    assert_eq!(
        fetch_generating_status(&app).await,
        "idle",
        "retry should finish in the Idle state"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 9.2
#[tokio::test]
async fn test_retry_reruns_quantifier_moves_player_http() {
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .build();
    // The retry's own generation reports the movement; the first Action's does not.
    let model = ScriptedModel::new()
        .turn("You walk around.")
        .narration("You enter room2.")
        .analysis(analysis_report(&[], Some("room2")));
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(model))
        .build_with_state();

    let resp = post_action(&app, "walk around").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "action should complete");
    {
        let room = state
            .game_view_query
            .get_debug_state(active_game_of(&state))
            .expect("get_debug_state should succeed")
            .current_room_id;
        assert_eq!(room, "room_1", "first action: stay");
    }

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 1000).await, "retry should complete");
    let room = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed")
        .current_room_id;
    assert_eq!(room, "room2", "retry re-ran quantifier → moved to room2");

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let narration = messages
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .expect("the generation appends a Narration");
    assert_eq!(
        narration.room_report(),
        Some("Room room2"),
        "the redo's Swipe reports the room it moved the player into"
    );
    assert_eq!(
        narration.swipes[0].room_report, None,
        "the first Swipe's own generation did not move, so it reports no room"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 9.3
#[tokio::test]
async fn test_retry_preserves_input_message_http() {
    let (app, state) = app_with_model(Arc::new(
        ScriptedModel::new()
            .turn("You walk around.")
            .narration("You walk around again.")
            .analysis(analysis_report(&[], None)),
    ));

    let resp = post_action(&app, "walk around").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 1000).await, "retry should complete");

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let input = messages
        .iter()
        .find(|m| m.message_type == MessageType::Input)
        .expect("Input message must exist");
    assert_eq!(input.text(), "walk around", "Input text preserved by retry");
}

// [docs/specs/swipe_new.md] SCENARIO: 9.4
#[tokio::test]
async fn test_retry_uses_edited_input_text_http() {
    let storage = Arc::new(empty_sqlite_storage());
    let model = ScriptedModel::new()
        .turn("You walk around.")
        .turn("You sprint forward.");
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .model(Arc::new(model))
        .build_with_state();

    let resp = post_action(&app, "walk around").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let input = messages
        .iter()
        .find(|m| m.message_type == MessageType::Input)
        .expect("Input message");
    let input_id = input.id;
    let index = input.active_swipe_index;
    storage
        .update_swipe_text(input_id, index, "sprint forward")
        .expect("update_swipe_text should succeed");

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 1000).await, "retry should complete");

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let retry_narration = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .next_back()
        .expect("retry narration");
    assert!(
        retry_narration.text().contains("sprint forward"),
        "retry narration should reflect edited input, got: {}",
        retry_narration.text()
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 9.5
#[tokio::test]
async fn test_retry_reevaluates_triggers_http() {
    use chronicler_engine::domain::model::trigger::ComparisonOperator;
    use chronicler_engine::test_support::TestNpc;

    let shopkeeper = TestNpc::with_room_scoped_trigger(
        "shopkeeper",
        "Shopkeeper Sarah",
        ComparisonOperator::Eq,
        0,
        "room2",
    );
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .npcs(vec![shopkeeper])
        .build();
    // room_npcs defaults to ["npc_1"]; the shopkeeper is not in any room's
    // NPC list (their trigger is room-gated), so no trigger fires on action 1.

    // The retry's generation moves the player into room2, which fires the
    // room-gated trigger; the event's own reconciliation reports nobody new.
    let model = ScriptedModel::new()
        .turn("You walk around.")
        .narration("You enter room2.")
        .analysis(analysis_report(&[], Some("room2")))
        .event_narration("The shopkeeper greets you.")
        .analysis(analysis_report(&[], None));
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(model))
        .build_with_state();

    let resp = post_action(&app, "walk around").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "action should complete");
    {
        let events = state
            .message_service
            .load_messages(active_game_of(&state))
            .unwrap()
            .iter()
            .filter(|m| m.event_header().is_some())
            .count();
        assert_eq!(events, 0, "first action: no trigger (player not in room2)");
    }

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 1000).await, "retry should complete");
    let events = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap()
        .iter()
        .filter(|m| m.event_header().is_some())
        .count();
    assert_eq!(
        events, 1,
        "retry moved player to room2 → trigger should fire"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 9.6
#[tokio::test]
async fn test_retry_completes_when_quantifier_returns_no_movement_http() {
    let model = ScriptedModel::new()
        .turn("You walk around.")
        .turn("You walk around again.");
    let (app, state) = TestAppBuilder::default_test()
        .model(Arc::new(model))
        .build_with_state();

    let resp = post_action(&app, "walk around").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "retry should complete even with no movement"
    );
    assert_eq!(
        fetch_generating_status(&app).await,
        "idle",
        "status should not be Generating"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 9.8
#[tokio::test]
async fn test_retry_swipe_without_movement_reports_no_room_http() {
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .build();
    // The Action moves the player; the retry that follows stays where it is.
    let model = ScriptedModel::new()
        .narration("You walk north.")
        .analysis(analysis_report(&[], Some("room2")))
        .narration("You stand still.")
        .analysis(analysis_report(&[], None));
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(model))
        .build_with_state();

    assert!(post_action(&app, "walk north").await.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "the moving Action should complete"
    );

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 1000).await, "retry should complete");

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let narration = messages
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .expect("the generation appends a Narration");
    assert_eq!(
        narration.swipes.len(),
        2,
        "the redo appends one Swipe to the Narration"
    );
    assert_eq!(
        narration.room_report(),
        None,
        "a redo that stays put must not keep the room its sibling arrived in"
    );
    assert_eq!(
        narration.swipes[0].room_report.as_deref(),
        Some("Room room2"),
        "the first Swipe keeps the room its own generation arrived in"
    );

    let narration_id = narration.id;
    let resp = post_empty(&app, &format!("/message/{narration_id}/swipe/0")).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let switched = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap()
        .into_iter()
        .find(|m| m.id == narration_id)
        .expect("the Narration survives the switch");
    assert_eq!(
        switched.room_report(),
        Some("Room room2"),
        "switching to the first Swipe shows the room it arrived in"
    );

    let resp = post_empty(&app, &format!("/message/{narration_id}/swipe/1")).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let switched = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap()
        .into_iter()
        .find(|m| m.id == narration_id)
        .expect("the Narration survives the switch back");
    assert_eq!(
        switched.room_report(),
        None,
        "switching to the redo's Swipe shows its own, room-less generation"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 10.1
#[tokio::test]
async fn test_event_retry_replaces_event_narration_with_new_swipe_http() {
    let storage = Arc::new(empty_sqlite_storage());
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .build();
    let model = ScriptedModel::new()
        .event_narration("The event happens again.")
        .analysis(analysis_report(&[], None));
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .storage(Arc::clone(&storage))
        .model(Arc::new(model))
        .build_with_state();

    seed_event_flow(&state, &storage, "room2").unwrap();

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "event retry should complete"
    );

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let narrations: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert_eq!(narrations.len(), 2, "main + event narration");
    let event_narration = narrations
        .iter()
        .find(|m| m.event_header().is_some())
        .expect("event narration with event_header");
    assert_eq!(
        event_narration.swipes.len(),
        2,
        "event retry appends exactly one swipe (original + new)"
    );
    assert_eq!(
        event_narration.active_swipe_index, 1,
        "new swipe should be the active swipe"
    );
    assert_ne!(
        event_narration.text(),
        "Event narration",
        "active swipe text should be the new retry narration, not the original"
    );
    assert_eq!(
        fetch_generating_status(&app).await,
        "idle",
        "event retry should finish in the Idle state"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 10.2
#[tokio::test]
async fn test_event_retry_does_not_move_the_player_http() {
    let storage = Arc::new(empty_sqlite_storage());
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .build();
    // The seeded report is a tripwire: the event's own reconciliation reads it,
    // and a movement-applying pass would move the player back.
    let model = ScriptedModel::new()
        .event_narration("The event happens again.")
        .analysis(analysis_report(&[], Some("room_1")));
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .storage(Arc::clone(&storage))
        .model(Arc::new(model))
        .build_with_state();

    seed_event_flow(&state, &storage, "room2").unwrap();

    let room_before = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed")
        .current_room_id;

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "event retry should complete"
    );

    let room_after = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed")
        .current_room_id;
    assert_eq!(
        room_before, room_after,
        "event retry must not re-run quantifier (room unchanged)"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 10.4
#[tokio::test]
async fn test_event_retry_after_retrigger_adds_a_swipe_http() {
    let storage = Arc::new(empty_sqlite_storage());
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .build();
    let model = ScriptedModel::new()
        .event_narration("The event fires again.")
        .analysis(analysis_report(&[], None))
        .event_narration("The event fires once more.")
        .analysis(analysis_report(&[], None));
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .storage(Arc::clone(&storage))
        .model(Arc::new(model))
        .build_with_state();

    seed_main_narration_flow(&state, &storage, "room2").unwrap();
    // The trigger matched but its continuation never landed: the context is
    // persisted, which is what makes Retrigger available.
    {
        let mut gs = state.message_service.load_or_fresh(active_game_of(&state));
        gs.narrative.last_trigger = Some(TestStoredTriggerContext::standard());
        storage
            .save_snapshot(
                active_game_of(&state),
                &GameStateSnapshot::from_game_state(&gs),
            )
            .unwrap();
    }

    let resp = post_empty(&app, "/retrigger").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "retrigger should complete"
    );
    let retriggered_event = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap()
        .into_iter()
        .next_back()
        .expect("retrigger appends the event narration");
    assert!(
        retriggered_event.event_header().is_some(),
        "retrigger should append an event narration"
    );

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "event retry should complete"
    );

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let event = messages
        .iter()
        .find(|m| m.id == retriggered_event.id)
        .expect("the retriggered event survives the retry");
    assert_eq!(
        event.swipes.len(),
        2,
        "event retry appends exactly one swipe (retriggered + new)"
    );
    assert_eq!(
        event.active_swipe_index, 1,
        "new swipe should be the active swipe"
    );
    assert_eq!(
        fetch_generating_status(&app).await,
        "idle",
        "event retry should finish in the Idle state"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 10.5
#[tokio::test]
async fn test_event_retry_does_not_carry_the_previous_swipes_encounter_http() {
    let (app, state, event_id) = app_with_an_event_that_meets_gabriella().await;
    assert_eq!(
        encounter_times_met(&state, "gabriella"),
        1,
        "the event's own reconciliation meets Gabriella"
    );

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "event retry should complete"
    );

    assert_eq!(
        encounter_times_met(&state, "gabriella"),
        0,
        "the retry's Swipe must not carry the previous Swipe's encounter"
    );
    let debug = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed");
    assert!(
        debug.npcs_in_area.is_empty(),
        "Gabriella is not in the retry's scene, got {:?}",
        debug.npcs_in_area
    );
    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let event = messages
        .iter()
        .find(|m| m.id == event_id)
        .expect("the event survives the retry");
    assert_eq!(
        event.swipes.len(),
        2,
        "event retry appends exactly one swipe"
    );
    assert_eq!(fetch_generating_status(&app).await, "idle");
}

// [docs/specs/swipe_new.md] SCENARIO: 10.6
#[tokio::test]
async fn test_event_retry_after_a_deleted_action_keeps_its_own_trigger_http() {
    use chronicler_engine::domain::model::trigger::ComparisonOperator;
    use chronicler_engine::test_support::TestNpc;

    let storage = Arc::new(empty_sqlite_storage());
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        // The seeded event's own trigger context belongs to `npc_1`, so the
        // world keeps it and adds the Character the later turn fires.
        .npc(TestNpc::with_room_scoped_trigger(
            "gabriella",
            "Gabriella",
            ComparisonOperator::Eq,
            0,
            "room2",
        ))
        .build();
    let model = ScriptedModel::new()
        .turn("You look around.")
        .event_narration("Gabriella steps out.")
        .analysis(analysis_report(&[], None))
        .event_narration("Gabriella steps out again.")
        .analysis(analysis_report(&[], None));
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .storage(Arc::clone(&storage))
        .model(Arc::new(model))
        .build_with_state();

    seed_event_flow(&state, &storage, "room2").unwrap();
    let seeded_messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let seeded = seeded_messages
        .last()
        .expect("the seeded event narration")
        .clone();
    let seeded_count = seeded_messages.len();
    let seeded_header = seeded
        .event_header()
        .map(str::to_string)
        .expect("the seeded event has a header");

    let resp = post_action(&app, "look around").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "the later turn should complete"
    );
    let later = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap()
        .into_iter()
        .next_back()
        .expect("the later turn's event narration");
    assert_eq!(
        later.event_header().map(str::to_string),
        Some("Gabriella Encounter in room2".to_string()),
        "the later turn fires Gabriella's Trigger"
    );

    // The delete loop below leaves the seeded event last again; the latest
    // Snapshot still carries the other Trigger's context.
    while state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap()
        .len()
        > seeded_count
    {
        let resp = post_empty(&app, "/history/delete").await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "event retry should complete"
    );

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let retried = messages
        .iter()
        .find(|m| m.id == seeded.id)
        .expect("the seeded event survives the retry");
    assert_eq!(
        retried.swipes.len(),
        2,
        "event retry appends exactly one swipe"
    );
    assert_eq!(
        retried.event_header().map(str::to_string),
        Some(seeded_header),
        "the retry reuses the event's own trigger context"
    );
    assert_eq!(fetch_generating_status(&app).await, "idle");
}

// [docs/specs/swipe_new.md] SCENARIO: 10.7
#[tokio::test]
async fn test_event_retry_matches_with_a_prior_swipe_switch_http() {
    let (app, state, event_id) = app_with_an_event_that_meets_gabriella().await;

    // Re-selecting the event's only Swipe re-saves its Snapshot as the latest
    // one; the retry must not read the world from there.
    let resp = post_empty(&app, &format!("/message/{event_id}/swipe/0")).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "event retry should complete"
    );

    assert_eq!(
        encounter_times_met(&state, "gabriella"),
        0,
        "a prior swipe switch must not change what the retry starts from"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 10.8
#[tokio::test]
async fn test_event_retry_keeps_the_room_name_off_the_event_swipe_http() {
    let (app, state, event_id) = app_with_an_event_that_meets_gabriella().await;

    // The room name belongs to the Narration the Action's movement arrived on, so
    // the event continuation that follows it in the same generation carries none.
    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let event_narration = messages
        .iter()
        .find(|m| m.id == event_id)
        .expect("the event Message exists");
    assert_eq!(
        event_narration.room_report(),
        None,
        "the event continuation must not carry the room the main Narration arrived in"
    );
    let main_narration = messages
        .iter()
        .find(|m| m.message_type == MessageType::Narration && m.event_header().is_none())
        .expect("the Action's main Narration exists");
    assert_eq!(
        main_narration.room_report(),
        Some("Room room2"),
        "the movement into \"room2\" must report that room's display name"
    );

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "event retry should complete"
    );

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let event_narration = messages
        .iter()
        .find(|m| m.id == event_id)
        .expect("the event Message survives the retry");
    assert_eq!(
        event_narration.swipes.len(),
        2,
        "event retry appends exactly one swipe"
    );
    assert_eq!(
        event_narration.room_report(),
        None,
        "the redo's Swipe must not invent a room its siblings do not carry"
    );
    assert_eq!(
        event_narration.swipes[0].room_report, None,
        "switching between the event's Swipes must not add or remove a room name"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 11.1
#[tokio::test]
async fn test_retry_no_input_returns_400_http() {
    let app = TestAppBuilder::default_app();

    let req = axum::http::Request::builder()
        .uri("/swipe/new")
        .method(axum::http::Method::POST)
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Retry with no input should return bad request"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 11.2
#[tokio::test]
async fn test_retry_no_game_context_returns_400_http() {
    // No Game in storage and no active-Game pointer: the `CurrentGame`
    // extractor answers the 400 before the handler runs.
    let app = TestAppBuilder::default_test()
        .storage(Arc::new(empty_sqlite_storage()))
        .skip_seeding(true)
        .build();

    let req = axum::http::Request::builder()
        .uri("/swipe/new")
        .method(axum::http::Method::POST)
        .body(axum::body::Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Should fail when game context is missing"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 11.3
#[tokio::test]
async fn test_retry_anchor_no_snapshot_returns_500_http() {
    let storage = Arc::new(empty_sqlite_storage());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();

    let input_msg = Message::new("look", MessageType::Input, None, None);
    insert_message_with_swipe(&storage, active_game_of(&state), &input_msg).unwrap();

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(
        resp.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "anchor with no snapshot_id should 500"
    );
    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("snapshot_id") || status.contains("snapshot"),
        "error should mention snapshot, got: {status}"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 11.4
#[tokio::test]
async fn test_retry_anchor_snapshot_deleted_returns_500_http() {
    let storage = Arc::new(empty_sqlite_storage());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();

    let mut input_msg = Message::new("look", MessageType::Input, None, None);
    input_msg.set_snapshot_id(Some(999_999));
    insert_message_with_swipe(&storage, active_game_of(&state), &input_msg).unwrap();

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(
        resp.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "anchor with deleted snapshot should 500"
    );
    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("no snapshot found") || status.contains("snapshot"),
        "error should mention snapshot not found, got: {status}"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 11.5
#[tokio::test]
async fn test_retry_llm_failure_sets_error_http() {
    let (app, state) = app_with_model(Arc::new(ScriptedModel::new().failed(ModelRole::Narration)));

    {
        let mut gs = state.message_service.load_or_fresh(active_game_of(&state));
        gs.add_message("look".to_string(), MessageType::Input);
        gs.add_message("Original narration.".to_string(), MessageType::Narration);
        state
            .message_service
            .save_message_and_snapshot(active_game_of(&state), &mut gs)
            .expect("seed input + narration");
    }

    let swipes_before = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap()
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .map(|m| m.swipes.len())
        .unwrap_or(0);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "retry should complete (with error)"
    );

    let status = fetch_generating_status(&app).await;
    let message = status_error_message(&status);
    assert!(!message.is_empty(), "error message should be non-empty");
    let swipes_after = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap()
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .map(|m| m.swipes.len())
        .unwrap_or(0);
    assert_eq!(
        swipes_before, swipes_after,
        "failed retry should not append a swipe"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 11.6
#[tokio::test]
async fn test_retry_empty_narration_sets_error_http() {
    let (app, state) = app_with_model(Arc::new(ScriptedModel::new().empty(ModelRole::Narration)));

    {
        let mut gs = state.message_service.load_or_fresh(active_game_of(&state));
        gs.add_message("look".to_string(), MessageType::Input);
        gs.add_message("Original narration.".to_string(), MessageType::Narration);
        state
            .message_service
            .save_message_and_snapshot(active_game_of(&state), &mut gs)
            .expect("seed input + narration");
    }

    let swipes_before = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap()
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .map(|m| m.swipes.len())
        .unwrap_or(0);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "retry should complete (with error)"
    );

    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("empty"),
        "error message should mention 'empty', got: {status}"
    );
    let swipes_after = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap()
        .iter()
        .find(|m| m.message_type == MessageType::Narration)
        .map(|m| m.swipes.len())
        .unwrap_or(0);
    assert_eq!(
        swipes_before, swipes_after,
        "failed retry should not append a swipe"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 11.7
#[tokio::test]
async fn test_retry_room_not_found_sets_error_http() {
    let storage = Arc::new(empty_sqlite_storage());
    // The room resolves before the model call, so this retry makes none.
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .model(Arc::new(ScriptedModel::new()))
        .build_with_state();

    let mut gs = state.message_service.load_or_fresh(active_game_of(&state));
    gs.swipe.movement.current_room_id = "non_existent_room".to_string();
    gs.add_message("look".to_string(), MessageType::Input);
    let snap = GameStateSnapshot::from_game_state(&gs);
    let input_snap_id = storage
        .save_snapshot(active_game_of(&state), &snap)
        .unwrap();
    if let Some(last) = gs.narrative.history.last_mut() {
        last.set_snapshot_id(Some(input_snap_id));
        let id = storage
            .insert_message(active_game_of(&state), last)
            .unwrap();
        if let Some(swipe) = last.swipes.first() {
            let _ = storage.insert_swipe(id, swipe, 0);
        }
    }

    let mut gs = state.message_service.load_or_fresh(active_game_of(&state));
    gs.add_message("A calm scene.".to_string(), MessageType::Narration);
    let snap = GameStateSnapshot::from_game_state(&gs);
    let narration_snap_id = storage
        .save_snapshot(active_game_of(&state), &snap)
        .unwrap();
    if let Some(last) = gs.narrative.history.last_mut() {
        last.set_snapshot_id(Some(narration_snap_id));
        let id = storage
            .insert_message(active_game_of(&state), last)
            .unwrap();
        if let Some(swipe) = last.swipes.first() {
            let _ = storage.insert_swipe(id, swipe, 0);
        }
    }

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "retry should complete (with error)"
    );

    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("Room not found"),
        "error should mention room invalid, got: {status}"
    );

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let narration = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration && m.event_header().is_none())
        .next_back()
        .expect("non-event narration preserved");
    assert_eq!(
        narration.swipes.len(),
        1,
        "original narration's swipes unchanged after room-not-found error"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 11.8
#[tokio::test]
async fn test_event_retry_trigger_narration_failure_sets_error_http() {
    let storage = Arc::new(empty_sqlite_storage());
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .build();
    // The event narration fails, so the event's reconciliation never runs.
    let model = ScriptedModel::new().failed(ModelRole::Event);
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .storage(Arc::clone(&storage))
        .model(Arc::new(model))
        .build_with_state();

    seed_event_flow(&state, &storage, "room2").unwrap();
    let messages_before = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap()
        .len();

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "event retry should complete (with error)"
    );

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
                && m.event_header().is_none()
                && !m.text().is_empty()
        }),
        "main narration should be preserved"
    );
    assert_eq!(
        messages.len(),
        messages_before,
        "a failed trigger must not append a story-log entry: {messages:?}"
    );

    let debug = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed");
    assert_eq!(
        debug.current_room_id, "room2",
        "failed event retry must not roll the turn's movement back"
    );
    assert_eq!(
        debug
            .npc_encounter_log
            .get("npc_1")
            .and_then(|state| state.trigger_fired.get(&0))
            .copied(),
        Some(true),
        "failed event retry must not clear the fired trigger"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 11.9
#[tokio::test]
async fn test_failed_retry_does_not_roll_the_movement_back_http() {
    let storage = Arc::new(empty_sqlite_storage());
    let model = ScriptedModel::new().failed(ModelRole::Narration);
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .build();
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .storage(Arc::clone(&storage))
        .model(Arc::new(model))
        .build_with_state();

    // The settled turn moved the player; the Narration's anchor is the room
    // before that movement, as a real turn leaves it.
    seed_main_narration_flow(&state, &storage, "room2").unwrap();

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "retry should complete (with error)"
    );

    let status = fetch_generating_status(&app).await;
    assert!(
        !status_error_message(&status).is_empty(),
        "failed retry should show an error, got: {status}"
    );
    let debug = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed");
    assert_eq!(
        debug.current_room_id, "room2",
        "a failed retry must not roll the turn's movement back"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 11.10
#[tokio::test]
async fn test_retry_that_wrote_its_narration_keeps_that_world_http() {
    use chronicler_engine::domain::model::trigger::ComparisonOperator;
    use chronicler_engine::test_support::TestNpc;

    let storage = Arc::new(empty_sqlite_storage());
    // The narration lands, then the trigger it fires fails: the redo wrote a
    // narration and moved the player, so that world is the one to keep.
    let model = ScriptedModel::new()
        .narration("You enter room2.")
        .analysis(analysis_report(&[], Some("room2")))
        .failed(ModelRole::Event);
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .npcs(vec![TestNpc::with_room_scoped_trigger(
            "shopkeeper",
            "Shopkeeper Sarah",
            ComparisonOperator::Eq,
            0,
            "room2",
        )])
        .build();
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .storage(Arc::clone(&storage))
        .model(Arc::new(model))
        .build_with_state();

    seed_main_narration_flow(&state, &storage, "room_1").unwrap();

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "retry should complete (with error)"
    );

    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("Trigger narration failed"),
        "error should mention 'Trigger narration failed', got: {status}"
    );
    let debug = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed");
    assert_eq!(
        debug.current_room_id, "room2",
        "the world the failed retry wrote is the one on screen"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 12.1
#[tokio::test]
async fn test_retry_concurrent_generation_returns_still_thinking_http() {
    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let hold = model.hold(ModelRole::Narration);
    let (app, state) = app_with_model(Arc::clone(&model));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success(), "the first Action should accept");
    assert!(
        hold.wait_started(1000),
        "the held narration call should reach the model"
    );

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 16384)
        .await
        .expect("read body");
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Still thinking..."),
        "expected 'Still thinking...' in body, got: {body_str}"
    );

    hold.release();
    assert!(
        wait_settled(&state, 1000).await,
        "the held Action should complete once released"
    );
    let (inputs, narrations) = story_log_counts(&state);
    assert_eq!(
        inputs, 1,
        "no retry was started: the held Action's own input is the only one"
    );
    assert_eq!(
        narrations, 1,
        "no retry was started: the held Action's own narration is the only one"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 22.1
#[tokio::test]
async fn test_retry_re_impersonate_generates_input_swipe_http() {
    let model = Arc::new(ScriptedModel::new().turn("I look around cautiously."));
    let (app, state) = app_with_model(Arc::clone(&model));

    {
        let mut gs = state.message_service.load_or_fresh(active_game_of(&state));
        let mut input = Message::new("I look around.".to_string(), MessageType::Input, None, None);
        input.set_stored_inputs(true, None, Some("I look around.".to_string()));
        gs.narrative.history.append(input);
        state
            .message_service
            .save_message_and_snapshot(active_game_of(&state), &mut gs)
            .expect("seed impersonate input");
    }

    let swipes_before = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap()
        .into_iter()
        .find(|m| m.message_type == MessageType::Input)
        .map(|m| m.swipes.len())
        .unwrap_or(0);

    let room_before = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed")
        .current_room_id;

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 1000).await, "retry should complete");

    assert_eq!(
        fetch_generating_status(&app).await,
        "idle",
        "retry should finish in the Idle state"
    );

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let inputs: Vec<_> = messages
        .into_iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert_eq!(inputs.len(), 1, "exactly one Input message");
    let input = &inputs[0];
    assert_eq!(
        input.swipes.len(),
        swipes_before + 1,
        "retry appended a swipe to the Input"
    );
    assert_eq!(
        input.text(),
        "I look around cautiously.",
        "active swipe is the regenerated impersonation"
    );
    assert!(input.impersonated(), "new swipe remains an impersonation");

    let room_after = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed")
        .current_room_id;
    assert_eq!(
        room_before, room_after,
        "the player has not moved: the redo's analysis reports no movement"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 22.2
#[tokio::test]
async fn test_retry_user_regen_generates_input_swipe_http() {
    // A user-regen redo narrates only: it runs no analysis.
    let (app, state) = app_with_model(Arc::new(
        ScriptedModel::new().narration("I sprint forward."),
    ));

    {
        let mut gs = state.message_service.load_or_fresh(active_game_of(&state));
        let input = Message::new(
            "I walk forward.".to_string(),
            MessageType::Input,
            None,
            None,
        );
        gs.narrative.history.append(input);
        state
            .message_service
            .save_message_and_snapshot(active_game_of(&state), &mut gs)
            .expect("seed plain input");
    }

    let swipes_before = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap()
        .into_iter()
        .find(|m| m.message_type == MessageType::Input)
        .map(|m| m.swipes.len())
        .unwrap_or(0);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 1000).await, "retry should complete");

    assert_eq!(
        fetch_generating_status(&app).await,
        "idle",
        "retry should finish in the Idle state"
    );

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let inputs: Vec<_> = messages
        .into_iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert_eq!(inputs.len(), 1, "exactly one Input message");
    let input = &inputs[0];
    assert_eq!(
        input.swipes.len(),
        swipes_before + 1,
        "retry appended a swipe to the Input"
    );
    assert_eq!(
        input.text(),
        "I sprint forward.",
        "active swipe is the regenerated user message"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 22.3
#[tokio::test]
async fn test_retry_re_impersonate_preserves_record_http() {
    let (app, state) = app_with_model(Arc::new(
        ScriptedModel::new().turn("I sneak past the guard."),
    ));

    {
        let mut gs = state.message_service.load_or_fresh(active_game_of(&state));
        let mut input = Message::new("I sneak past.".to_string(), MessageType::Input, None, None);
        input.set_stored_inputs(true, None, Some("Sneak past the guard.".to_string()));
        gs.narrative.history.append(input);
        state
            .message_service
            .save_message_and_snapshot(active_game_of(&state), &mut gs)
            .expect("seed impersonate input with record");
    }

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 1000).await, "retry should complete");

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let input = messages
        .into_iter()
        .find(|m| m.message_type == MessageType::Input)
        .expect("Input message exists");
    assert!(
        input.impersonated(),
        "record still marks the turn as impersonated"
    );
    assert_eq!(
        input.impersonate_direction(),
        Some("Sneak past the guard."),
        "the impersonate direction re-applies on the redo"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 22.4
#[tokio::test]
async fn test_retry_re_impersonate_uses_current_preset_content_http() {
    let app_storage = Arc::new(empty_sqlite_storage());
    // The recorder persists every call into the storage the app reads its
    // forensics from; .storage() hands the builder that same instance, so the
    // preset update below hits the wired storage, not an orphan.
    let (app, state, app_storage) = app_with_model_and_storage(
        Arc::clone(&app_storage),
        Arc::new(ScriptedModel::new().turn("I look around cautiously.")),
        AppSettings::default(),
        configured_runner,
    );

    {
        let mut gs = state.message_service.load_or_fresh(active_game_of(&state));
        let mut input = Message::new("I look around.".to_string(), MessageType::Input, None, None);
        input.set_stored_inputs(true, None, Some("I look around.".to_string()));
        gs.narrative.history.append(input);
        state
            .message_service
            .save_message_and_snapshot(active_game_of(&state), &mut gs)
            .expect("seed impersonate input");
    }

    // Same id, new configuration: if the redo still pinned the old preset, the
    // marker would never reach the prompt.
    let marker = "MARKER-IMPERSONATE-PRESET-V2";
    {
        let mut preset = state
            .prompt_preset_service
            .get_preset("impersonate_default")
            .expect("get impersonate preset")
            .expect("impersonate_default seeded");
        preset.instructions = Some(format!(
            "{}\n{}",
            marker,
            preset.instructions.unwrap_or_default()
        ));
        app_storage
            .save_preset(&preset)
            .expect("update impersonate preset content");
    }

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 1000).await, "retry should complete");

    let forensics = state
        .game_view_query
        .list_latest_llm_messages(10)
        .expect("list llm forensics");
    let narrator_call = forensics
        .iter()
        .find(|m| m.agent_name.contains("narrator"))
        .expect("narrator forensics recorded");
    let prompt = format!(
        "{} {}",
        narrator_call.system_prompt, narrator_call.user_prompt
    );
    assert!(
        prompt.contains(marker),
        "the redo must generate with the preset's CURRENT content; prompt was: {prompt}"
    );

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let input = messages
        .into_iter()
        .find(|m| m.message_type == MessageType::Input)
        .expect("Input message exists");
    assert!(
        input.impersonated(),
        "the redo's active swipe stays impersonated"
    );
    assert_eq!(
        input.impersonate_direction(),
        Some("I look around."),
        "the impersonate direction re-applies on the redo"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 22.5
#[tokio::test]
async fn test_guided_retry_reports_the_room_its_redo_arrives_in_http() {
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .build();
    // Consumed in call order: the moving Action, the guided generation, then the redo.
    let model = ScriptedModel::new()
        .narration("You walk north.")
        .analysis(analysis_report(&[], Some("room2")))
        .narration("You look for the key.")
        .analysis(analysis_report(&[], None))
        .narration("You look for the key again.")
        .analysis(analysis_report(&[], Some("room_1")));
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(model))
        .build_with_state();

    assert!(post_action(&app, "walk north").await.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "the moving Action should complete"
    );
    assert!(
        post_action(&app, "/guide look for the key")
            .await
            .status()
            .is_success()
    );
    assert!(
        wait_settled(&state, 1000).await,
        "the guided generation should complete"
    );

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let guided = messages
        .iter()
        .find(|m| m.is_guided())
        .expect("the guided generation appends a guided Narration");
    assert_eq!(
        guided.room_report(),
        None,
        "a guided Narration that stayed put must not take the previous Action's room"
    );

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 1000).await,
        "the guided retry should complete"
    );

    let messages = state
        .message_service
        .load_messages(active_game_of(&state))
        .unwrap();
    let guided = messages
        .iter()
        .find(|m| m.is_guided())
        .expect("the guided Message survives the retry");
    assert_eq!(
        guided.swipes.len(),
        2,
        "the redo appends one Swipe to the guided Narration"
    );
    assert_eq!(
        guided.room_report(),
        Some("Room room_1"),
        "the redo's Swipe reports the room its own generation arrived in"
    );
    assert_eq!(
        guided.swipes[0].room_report, None,
        "the first Swipe keeps its own, room-less generation"
    );
}
