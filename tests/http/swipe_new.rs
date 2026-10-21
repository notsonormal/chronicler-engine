//! HTTP E2E tests for the retry endpoint (POST /swipe/new).

use std::sync::Arc;

use axum::http::StatusCode;
use tower::util::ServiceExt;

use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::domain::model::settings::AppSettings;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::{
    active_game_of, analysis_report, configured_runner, empty_sqlite_storage, ScriptedModel,
    TestAppBuilder, TestDataBuilder, TestMap,
};

use crate::support::app_wiring::{app_with_a_triggered_npc, app_with_model, app_with_model_and_storage};
use crate::support::http_assertions::{assert_still_thinking, status_error_message};
use crate::support::http_fixtures::{break_start_room, room_header_of, story_of};
use crate::support::http_requests::{
    fetch_generating_status, post_action, post_empty, post_form, wait_settled,
};

fn story_log_counts(state: &AppState) -> (usize, usize) {
    let messages = story_of(state);
    let count = |kind: MessageType| messages.iter().filter(|m| m.message_type == kind).count();
    (count(MessageType::Input), count(MessageType::Narration))
}

fn newest_swipes(state: &AppState, message_type: MessageType) -> usize {
    story_of(state)
        .into_iter()
        .rfind(|m| m.message_type == message_type)
        .map(|m| m.swipes.len())
        .unwrap_or(0)
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
        wait_settled(&state, 2000).await,
        "the trigger turn should complete"
    );

    let event_id = story_of(&state)
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
    assert!(wait_settled(&state, 2000).await, "action should complete");

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 2000).await, "retry should complete");

    let messages = story_of(&state);
    let narrations: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert_eq!(
        narrations.len(),
        2,
        "the opening Narration and the Action's Narration"
    );
    let retried = narrations.last().expect("the Action appends a Narration");
    assert_eq!(retried.swipes.len(), 2, "retry appended a swipe");
    assert_eq!(retried.text(), "Second.", "active swipe is the retry");
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
    assert!(wait_settled(&state, 2000).await, "action should complete");
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
    assert!(wait_settled(&state, 2000).await, "retry should complete");
    let room = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed")
        .current_room_id;
    assert_eq!(room, "room2", "retry re-ran quantifier → moved to room2");

    let messages = story_of(&state);
    let narration = messages
        .iter()
        .rfind(|m| m.message_type == MessageType::Narration)
        .expect("the generation appends a Narration");
    assert_eq!(
        room_header_of(&state, narration.id).as_deref(),
        Some("Room room2"),
        "the redo's Swipe shows the room it moved the player into"
    );

    let resp = post_empty(&app, &format!("/message/{}/swipe/0", narration.id)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        room_header_of(&state, narration.id),
        None,
        "after a switch to the first Swipe, whose generation did not move, the Narration shows no room"
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
    assert!(wait_settled(&state, 2000).await);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 2000).await, "retry should complete");

    let messages = story_of(&state);
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
    assert!(wait_settled(&state, 2000).await);

    let messages = story_of(&state);
    let input = messages
        .iter()
        .find(|m| m.message_type == MessageType::Input)
        .expect("Input message");
    let input_id = input.id;
    let edit = post_form(&app, &format!("/history/{input_id}"), "text=sprint+forward").await;
    assert_eq!(edit.status(), StatusCode::OK, "the edit should succeed");

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 2000).await, "retry should complete");

    let messages = story_of(&state);
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
    assert!(wait_settled(&state, 2000).await, "action should complete");
    {
        let events = story_of(&state)
            .iter()
            .filter(|m| m.event_header().is_some())
            .count();
        assert_eq!(events, 0, "first action: no trigger (player not in room2)");
    }

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 2000).await, "retry should complete");
    let events = story_of(&state)
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
    assert!(wait_settled(&state, 2000).await);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 2000).await,
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
async fn test_retry_swipe_without_movement_shows_no_room_http() {
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .build();
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
        wait_settled(&state, 2000).await,
        "the moving Action should complete"
    );

    let moved = story_of(&state)
        .iter()
        .rfind(|m| m.message_type == MessageType::Narration)
        .expect("the Action appends a Narration")
        .id;
    assert_eq!(
        room_header_of(&state, moved).as_deref(),
        Some("Room room2"),
        "the first Swipe arrived in room2"
    );

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 2000).await, "retry should complete");

    let messages = story_of(&state);
    let narration = messages
        .iter()
        .rfind(|m| m.message_type == MessageType::Narration)
        .expect("the generation appends a Narration");
    assert_eq!(
        narration.swipes.len(),
        2,
        "the redo appends one Swipe to the Narration"
    );
    assert_eq!(
        room_header_of(&state, narration.id),
        None,
        "a redo that stays put must not keep the room its sibling arrived in"
    );

    let resp = post_empty(&app, &format!("/message/{}/swipe/0", narration.id)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        room_header_of(&state, narration.id).as_deref(),
        Some("Room room2"),
        "after a switch to the first Swipe, the Narration shows the room its own generation arrived in"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 10.1
#[tokio::test]
async fn test_event_retry_replaces_event_narration_with_new_swipe_http() {
    use chronicler_engine::domain::model::trigger::ComparisonOperator;
    use chronicler_engine::test_support::TestNpc;

    // The Action moves into "room2" and meets Gabriella, whose room-scoped Trigger
    // fires, so the story ends on an event Narration.
    let npc = TestNpc::with_room_scoped_trigger(
        "gabriella",
        "Gabriella",
        ComparisonOperator::Eq,
        0,
        "room2",
    );
    let model = ScriptedModel::new()
        .narration("You walk into the next room.")
        .analysis(analysis_report(&[], Some("room2")))
        .event_narration("Gabriella is here.")
        .analysis(analysis_report(&["gabriella"], None))
        .event_narration("The event happens again.")
        .analysis(analysis_report(&[], None));
    let (app, state) = app_with_a_triggered_npc(TestMap::two_rooms("room_1", "room2"), npc, model);

    assert!(post_action(&app, "walk north").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the Action that fires the Trigger should complete"
    );

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 2000).await,
        "event retry should complete"
    );

    let messages = story_of(&state);
    let narrations: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert_eq!(
        narrations.len(),
        3,
        "the opening Narration, the main narration and the event narration"
    );
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
    assert_eq!(
        event_narration.text(),
        "The event happens again.",
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
    use chronicler_engine::domain::model::trigger::ComparisonOperator;
    use chronicler_engine::test_support::TestNpc;

    // The Action moves into "room2" and fires Gabriella's Trigger. The retry's own
    // analysis names "room_1" as a tripwire: an event generation applies no
    // movement, so a movement-applying pass would move the player back.
    let npc = TestNpc::with_room_scoped_trigger(
        "gabriella",
        "Gabriella",
        ComparisonOperator::Eq,
        0,
        "room2",
    );
    let model = ScriptedModel::new()
        .narration("You walk into the next room.")
        .analysis(analysis_report(&[], Some("room2")))
        .event_narration("Gabriella is here.")
        .analysis(analysis_report(&["gabriella"], None))
        .event_narration("The event happens again.")
        .analysis(analysis_report(&[], Some("room_1")));
    let (app, state) = app_with_a_triggered_npc(TestMap::two_rooms("room_1", "room2"), npc, model);

    assert!(post_action(&app, "walk north").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the Action that fires the Trigger should complete"
    );

    let room_before = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed")
        .current_room_id;
    assert_eq!(room_before, "room2", "the Action moved the player");

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 2000).await,
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
    use chronicler_engine::domain::model::trigger::ComparisonOperator;
    use chronicler_engine::test_support::TestNpc;

    // The Action's Trigger fires, but its continuation fails: the main Narration
    // keeps its trigger context, which is what makes Retrigger available.
    let npc = TestNpc::with_times_met_trigger("npc_1", "Test NPC", ComparisonOperator::Eq, 0);
    let model = ScriptedModel::new()
        .narration("You look around.")
        .analysis(analysis_report(&["npc_1"], None))
        .failed(Role::Trigger)
        .event_narration("The event fires again.")
        .analysis(analysis_report(&["npc_1"], None))
        .event_narration("The event fires once more.")
        .analysis(analysis_report(&["npc_1"], None));
    let (app, state) = app_with_a_triggered_npc(TestMap::two_rooms("room_1", "room2"), npc, model);

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the Action should settle after its failed continuation"
    );

    let resp = post_empty(&app, "/retrigger").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 2000).await,
        "retrigger should complete"
    );
    let retriggered_event = story_of(&state)
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
        wait_settled(&state, 2000).await,
        "event retry should complete"
    );

    let messages = story_of(&state);
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
        wait_settled(&state, 2000).await,
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
    let messages = story_of(&state);
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

    // Two Characters, each with its own Trigger: `npc_1`'s fires on the first
    // Action, and Gabriella's room-scoped one fires when the later Action moves
    // the player into "room2".
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .npcs(vec![
            TestNpc::with_times_met_trigger("npc_1", "Test NPC", ComparisonOperator::Eq, 0),
            TestNpc::with_room_scoped_trigger(
                "gabriella",
                "Gabriella",
                ComparisonOperator::Eq,
                0,
                "room2",
            ),
        ])
        .build();
    let model = ScriptedModel::new()
        .narration("You look around.")
        .analysis(analysis_report(&["npc_1"], None))
        .event_narration("The NPC steps out.")
        .analysis(analysis_report(&["npc_1"], None))
        .narration("You step into the next room.")
        .analysis(analysis_report(&["gabriella"], Some("room2")))
        .event_narration("Gabriella steps out.")
        .analysis(analysis_report(&["gabriella"], None))
        .event_narration("The NPC steps out again.")
        .analysis(analysis_report(&["npc_1"], None));
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(model))
        .build_with_state();

    assert!(post_action(&app, "look around").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the first turn should complete"
    );
    let first_event = story_of(&state)
        .into_iter()
        .next_back()
        .expect("the first turn's event narration");
    assert_eq!(
        first_event.event_header().map(str::to_string),
        Some("Test NPC Introduction".to_string()),
        "the first turn fires npc_1's Trigger"
    );
    let seeded = first_event;
    let seeded_count = story_of(&state).len();
    let seeded_header = seeded
        .event_header()
        .map(str::to_string)
        .expect("the first event has a header");

    let resp = post_action(&app, "walk north").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the later turn should complete"
    );
    let later = story_of(&state)
        .into_iter()
        .next_back()
        .expect("the later turn's event narration");
    assert_eq!(
        later.event_header().map(str::to_string),
        Some("Gabriella Encounter in room2".to_string()),
        "the later turn fires Gabriella's Trigger"
    );

    // The delete loop below leaves the first event last again; its own record is
    // the Trigger the retry must reuse.
    while story_of(&state).len() > seeded_count {
        let resp = post_empty(&app, "/history/delete").await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 2000).await,
        "event retry should complete"
    );

    let messages = story_of(&state);
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

    // Re-selecting the event's only Swipe makes it the active Swipe again; the
    // retry must read the world from the Message before it.
    let resp = post_empty(&app, &format!("/message/{event_id}/swipe/0")).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 2000).await,
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
    let messages = story_of(&state);
    let event_narration = messages
        .iter()
        .find(|m| m.id == event_id)
        .expect("the event Message exists");
    assert_eq!(
        room_header_of(&state, event_narration.id),
        None,
        "the event continuation must not repeat the room the main Narration arrived in"
    );
    let main_narration = messages
        .iter()
        .rfind(|m| m.message_type == MessageType::Narration && m.event_header().is_none())
        .expect("the Action's main Narration exists");
    assert_eq!(
        room_header_of(&state, main_narration.id).as_deref(),
        Some("Room room2"),
        "the movement into \"room2\" must show that room's display name"
    );

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 2000).await,
        "event retry should complete"
    );

    let messages = story_of(&state);
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
        room_header_of(&state, event_narration.id),
        None,
        "the redo's Swipe must not invent a room its siblings do not carry"
    );

    let resp = post_empty(&app, &format!("/message/{event_id}/swipe/0")).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        room_header_of(&state, event_id),
        None,
        "after a switch to the first Swipe, the event Narration still shows no room"
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

// [docs/specs/swipe_new.md] SCENARIO: 11.5
#[tokio::test]
async fn test_retry_llm_failure_sets_error_http() {
    let (app, state) = app_with_model(Arc::new(
        ScriptedModel::new()
            .turn("Original narration.")
            .failed(Role::Narrator),
    ));

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the Action should complete"
    );

    let swipes_before = newest_swipes(&state, MessageType::Narration);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 2000).await,
        "retry should complete (with error)"
    );

    let status = fetch_generating_status(&app).await;
    let message = status_error_message(&status);
    assert!(!message.is_empty(), "error message should be non-empty");
    assert_eq!(
        newest_swipes(&state, MessageType::Narration),
        swipes_before,
        "failed retry should not append a swipe"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 11.6
#[tokio::test]
async fn test_retry_empty_narration_sets_error_http() {
    let (app, state) = app_with_model(Arc::new(
        ScriptedModel::new()
            .turn("Original narration.")
            .empty(Role::Narrator),
    ));

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the Action should complete"
    );

    let swipes_before = newest_swipes(&state, MessageType::Narration);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 2000).await,
        "retry should complete (with error)"
    );

    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("empty"),
        "error message should mention 'empty', got: {status}"
    );
    assert_eq!(
        newest_swipes(&state, MessageType::Narration),
        swipes_before,
        "failed retry should not append a swipe"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 11.7
#[tokio::test]
async fn test_retry_room_not_found_sets_error_http() {
    // The room resolves before the model call, so this retry makes none.
    let storage = Arc::new(empty_sqlite_storage());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .model(Arc::new(ScriptedModel::new().turn("A calm scene.")))
        .build_with_state();

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the Action should complete"
    );
    break_start_room(&storage, active_game_of(&state), "non_existent_room");

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 2000).await,
        "retry should complete (with error)"
    );

    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("The room non_existent_room is not on the map."),
        "the failure names the invalid room, got: {status}"
    );

    let messages = story_of(&state);
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
    use chronicler_engine::domain::model::trigger::ComparisonOperator;
    use chronicler_engine::test_support::TestNpc;

    // The Action fires npc_1's Trigger and its continuation lands; the retry's
    // event narration then fails, so the event's reconciliation never runs.
    let npc = TestNpc::with_times_met_trigger("npc_1", "Test NPC", ComparisonOperator::Eq, 0);
    let model = ScriptedModel::new()
        .narration("The main narration.")
        .analysis(analysis_report(&["npc_1"], Some("room2")))
        .event_narration("The NPC greets you.")
        .analysis(analysis_report(&["npc_1"], None))
        .failed(Role::Trigger);
    let (app, state) = app_with_a_triggered_npc(TestMap::two_rooms("room_1", "room2"), npc, model);

    assert!(post_action(&app, "walk north").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the Action should complete"
    );
    let messages_before = story_of(&state).len();

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 2000).await,
        "event retry should complete (with error)"
    );

    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("The event narration failed."),
        "the failure line of a failed event narration, got: {status}"
    );
    let messages = story_of(&state);
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
    // The settled Action moved the player; the redo starts from the state before
    // that movement, and its narrator call fails, so the movement stays.
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .build();
    let model = ScriptedModel::new()
        .narration("You walk north.")
        .analysis(analysis_report(&[], Some("room2")))
        .failed(Role::Narrator);
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(model))
        .build_with_state();

    assert!(post_action(&app, "walk north").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the moving Action should complete"
    );
    let room_moved = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed")
        .current_room_id;
    assert_eq!(room_moved, "room2", "the Action moved the player");

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 2000).await,
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

    // The Action stays in "room_1". The redo's narration lands and its analysis
    // moves the player into "room2", which fires the room-scoped Trigger, whose
    // narration then fails: the redo's own world is the one to keep.
    let npc = TestNpc::with_room_scoped_trigger(
        "shopkeeper",
        "Shopkeeper Sarah",
        ComparisonOperator::Eq,
        0,
        "room2",
    );
    let model = ScriptedModel::new()
        .narration("You look around.")
        .analysis(analysis_report(&[], None))
        .narration("You enter room2.")
        .analysis(analysis_report(&[], Some("room2")))
        .failed(Role::Trigger);
    let (app, state) = app_with_a_triggered_npc(TestMap::two_rooms("room_1", "room2"), npc, model);

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the Action should complete"
    );

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 2000).await,
        "retry should complete (with error)"
    );

    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("The event narration failed."),
        "the failure line of a failed event narration, got: {status}"
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
    let hold = model.hold(Role::Narrator);
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
    assert_still_thinking(&body_str);

    hold.release();
    assert!(
        wait_settled(&state, 2000).await,
        "the held Action should complete once released"
    );
    let (inputs, narrations) = story_log_counts(&state);
    assert_eq!(
        inputs, 1,
        "no retry was started: the held Action's own input is the only one"
    );
    assert_eq!(
        narrations, 2,
        "the opening Narration and the held Action's own one, and no retry"
    );
    let last = story_of(&state)
        .into_iter()
        .next_back()
        .expect("the held Action's Narration");
    assert_eq!(
        last.swipes.len(),
        1,
        "no retry was started: the held Action's Narration holds one Swipe"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 22.1
#[tokio::test]
async fn test_retry_re_impersonate_generates_input_swipe_http() {
    // The player's own turn is an Impersonate Input; the redo runs the analysis,
    // which moves the player into "room2".
    let data = TestDataBuilder::default_test()
        .map(TestMap::two_rooms("room_1", "room2"))
        .build();
    let model = ScriptedModel::new()
        .narration("I look around.")
        .analysis(analysis_report(&[], None))
        .narration("I look around cautiously.")
        .analysis(analysis_report(&[], Some("room2")));
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(model))
        .build_with_state();

    assert!(
        post_action(&app, "/impersonate I look around.")
            .await
            .status()
            .is_success()
    );
    assert!(
        wait_settled(&state, 2000).await,
        "the impersonate turn should complete"
    );

    let swipes_before = newest_swipes(&state, MessageType::Input);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 2000).await, "retry should complete");

    assert_eq!(
        fetch_generating_status(&app).await,
        "idle",
        "retry should finish in the Idle state"
    );

    let messages = story_of(&state);
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
        room_after, "room2",
        "a re-impersonate runs the analysis, so its movement applies"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 22.2
#[tokio::test]
async fn test_retry_user_regen_generates_input_swipe_http() {
    // The Action's own Narration is deleted, so the story ends on its typed Input.
    // A user-regen redo narrates only: it runs no analysis.
    let data = TestDataBuilder::default_test().build();
    let model = ScriptedModel::new()
        .narration("You walk forward.")
        .analysis(analysis_report(&[], None))
        .narration("I sprint forward.");
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(model))
        .build_with_state();

    assert!(
        post_action(&app, "I walk forward.")
            .await
            .status()
            .is_success()
    );
    assert!(
        wait_settled(&state, 2000).await,
        "the Action should complete"
    );
    let resp = post_empty(&app, "/history/delete").await;
    assert_eq!(resp.status(), StatusCode::OK, "the delete should accept");
    let room_before = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed")
        .current_room_id;

    let swipes_before = newest_swipes(&state, MessageType::Input);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 2000).await, "retry should complete");

    assert_eq!(
        fetch_generating_status(&app).await,
        "idle",
        "retry should finish in the Idle state"
    );

    let messages = story_of(&state);
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
    let room_after = state
        .game_view_query
        .get_debug_state(active_game_of(&state))
        .expect("get_debug_state should succeed")
        .current_room_id;
    assert_eq!(
        room_before, room_after,
        "the player has not moved: the quantifier did not re-run"
    );
}

// [docs/specs/swipe_new.md] SCENARIO: 22.3
#[tokio::test]
async fn test_retry_re_impersonate_preserves_record_http() {
    let data = TestDataBuilder::default_test().build();
    let model = ScriptedModel::new()
        .narration("I sneak past.")
        .analysis(analysis_report(&[], None))
        .narration("I sneak past the guard.")
        .analysis(analysis_report(&[], None));
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(model))
        .build_with_state();

    assert!(
        post_action(&app, "/impersonate Sneak past the guard.")
            .await
            .status()
            .is_success()
    );
    assert!(
        wait_settled(&state, 2000).await,
        "the impersonate turn should complete"
    );

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(wait_settled(&state, 2000).await, "retry should complete");

    let messages = story_of(&state);
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
        Arc::new(
            ScriptedModel::new()
                .narration("I look around.")
                .analysis(analysis_report(&[], None))
                .narration("I look around cautiously.")
                .analysis(analysis_report(&[], None)),
        ),
        AppSettings::default(),
        configured_runner,
    );

    assert!(
        post_action(&app, "/impersonate I look around.")
            .await
            .status()
            .is_success()
    );
    assert!(
        wait_settled(&state, 2000).await,
        "the impersonate turn should complete"
    );

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
    assert!(wait_settled(&state, 2000).await, "retry should complete");

    let forensics = state
        .game_view_query
        .list_latest_llm_messages(10)
        .expect("list llm forensics");
    let narrator_call = forensics
        .iter()
        .rfind(|m| m.agent_name.contains("narrator"))
        .expect("narrator forensics recorded");
    let prompt = format!(
        "{} {}",
        narrator_call.system_prompt, narrator_call.user_prompt
    );
    assert!(
        prompt.contains(marker),
        "the redo must generate with the preset's CURRENT content; prompt was: {prompt}"
    );

    let messages = story_of(&state);
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
async fn test_guided_retry_shows_the_room_its_redo_arrives_in_http() {
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
        wait_settled(&state, 2000).await,
        "the moving Action should complete"
    );
    assert!(
        post_action(&app, "/guide look for the key")
            .await
            .status()
            .is_success()
    );
    assert!(
        wait_settled(&state, 2000).await,
        "the guided generation should complete"
    );

    let messages = story_of(&state);
    let guided = messages
        .iter()
        .find(|m| m.is_guided())
        .expect("the guided generation appends a guided Narration");
    assert_eq!(
        room_header_of(&state, guided.id),
        None,
        "a guided Narration that stayed put must not repeat the previous Action's room"
    );

    let resp = post_empty(&app, "/swipe/new").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
        wait_settled(&state, 2000).await,
        "the guided retry should complete"
    );

    let messages = story_of(&state);
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
        room_header_of(&state, guided.id).as_deref(),
        Some("Room room_1"),
        "the redo's Swipe shows the room its own generation arrived in"
    );

    let resp = post_empty(&app, &format!("/message/{}/swipe/0", guided.id)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        room_header_of(&state, guided.id),
        None,
        "after a switch to the first Swipe, the guided Narration shows no room"
    );
}
