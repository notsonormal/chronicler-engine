//! HTTP E2E tests for the story-log fragment: delete flows through POST /history/delete, the fragment's shape, and the controls each entry renders.

use std::sync::Arc;

use axum::http::StatusCode;

use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::trigger::ComparisonOperator;
use chronicler_engine::test_support::{
    active_game_of, analysis_report, empty_sqlite_storage, ScriptedModel, TestAppBuilder,
    TestDataBuilder, TestNpc,
};

use crate::support::http_assertions::{
    icon_buttons_without_matching_name, svgs_not_hidden_from_assistive_technology,
};
use crate::support::http_fixtures::{append_narrations, story_of};
use crate::support::http_requests::{
    fetch_body, fetch_generating_status, post_action, post_empty, post_form, response_body,
    wait_settled,
};

// [docs/specs/story_log.md] SCENARIO: 8.1
#[tokio::test]
async fn test_delete_last_between_actions_http() {
    let (app, state) = TestAppBuilder::default_test()
        .model(Arc::new(
            ScriptedModel::new()
                .turn("You examine the room.")
                .narration("You look around.")
                .analysis(analysis_report(&[], None)),
        ))
        .build_with_state();

    let resp = post_action(&app, "examine room").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "action A should complete");
    let narration_a = story_of(&state)
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .map(|m| m.text().to_string())
        .next_back()
        .expect("narration A should be persisted");

    let resp = post_empty(&app, "/history/delete").await;
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = post_action(&app, "look around").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "action B should complete");

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
        !narrations.is_empty(),
        "narration B should be present after action B"
    );
    assert!(
        !narrations.iter().any(|m| m.text() == narration_a),
        "deleted narration A should not reappear"
    );
}

// [docs/specs/story_log.md] SCENARIO: 8.2
#[tokio::test]
async fn test_delete_mid_sequence_http() {
    let (app, state) = TestAppBuilder::default_test()
        .model(Arc::new(
            ScriptedModel::new()
                .turn("You examine the room.")
                .narration("You look around.")
                .analysis(analysis_report(&[], None))
                .narration("You check the door.")
                .analysis(analysis_report(&[], None)),
        ))
        .build_with_state();

    let _ = post_action(&app, "examine room").await;
    assert!(wait_settled(&state, 1000).await);
    let _ = post_action(&app, "look around").await;
    assert!(wait_settled(&state, 1000).await);
    let narration_b = story_of(&state)
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .map(|m| m.text().to_string())
        .next_back()
        .expect("narration B should be the latest narration");

    let resp = post_empty(&app, "/history/delete").await;
    assert_eq!(resp.status(), StatusCode::OK);

    let _ = post_action(&app, "check door").await;
    assert!(wait_settled(&state, 1000).await);

    let messages = story_of(&state);
    let inputs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert_eq!(inputs.len(), 3, "should have 3 Input entries");
    assert!(
        !messages
            .iter()
            .filter(|m| m.message_type == MessageType::Narration)
            .any(|m| m.text() == narration_b),
        "deleted narration B should not reappear"
    );
}

// [docs/specs/story_log.md] SCENARIO: 8.9
#[tokio::test]
async fn test_edit_of_an_unknown_message_is_refused_http() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();
    let before: Vec<(u64, String)> = story_of(&state)
        .into_iter()
        .map(|message| (message.id, message.text().to_string()))
        .collect();

    let resp = post_form(&app, "/history/999999", "text=Changed.").await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = response_body(resp).await;
    assert!(
        body.contains("Message not found"),
        "the refusal names what the edit could not find: {body}"
    );

    let after: Vec<(u64, String)> = story_of(&state)
        .into_iter()
        .map(|message| (message.id, message.text().to_string()))
        .collect();
    assert_eq!(after, before, "the refused edit changes no Message");
}

// [docs/specs/story_log.md] SCENARIO: 8.10
#[tokio::test]
async fn test_delete_of_the_only_message_is_refused_http() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();
    let before = story_of(&state);
    assert_eq!(
        before.len(),
        1,
        "the seeded Game holds only its opening Narration"
    );

    let resp = post_empty(&app, "/history/delete").await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = response_body(resp).await;
    assert!(
        body.contains("The story keeps its last message"),
        "the refusal says why the delete cannot run: {body}"
    );

    let after = story_of(&state);
    assert_eq!(after.len(), 1, "the refused delete drops no Message");
    assert_eq!(
        after[0].text(),
        before[0].text(),
        "the opening Narration stays"
    );
}

// [docs/specs/story_log.md] SCENARIO: 8.3
#[tokio::test]
async fn test_delete_input_then_retry_fails_gracefully_http() {
    let (app, state) = TestAppBuilder::default_test()
        .model(Arc::new(
            ScriptedModel::new()
                .turn("You examine the room.")
                .narration("The room is empty.")
                .analysis(analysis_report(&[], None)),
        ))
        .build_with_state();

    let _ = post_action(&app, "examine room").await;
    assert!(wait_settled(&state, 1000).await);

    // The delete leaves an Input with no anchor narration.
    let resp = post_empty(&app, "/history/delete").await;
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = post_empty(&app, "/swipe/new").await;
    assert_ne!(
        resp.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "retry after delete should not 500"
    );
    assert!(
        wait_settled(&state, 1000).await,
        "retry after delete should not leave state generating"
    );
}

// [docs/specs/story_log.md] SCENARIO: 8.4
#[tokio::test]
async fn test_delete_removes_entry_from_fragment_http() {
    let (app, state) = TestAppBuilder::default_test()
        .model(Arc::new(ScriptedModel::new().turn("You examine the room.")))
        .build_with_state();

    let _ = post_action(&app, "examine room").await;
    assert!(wait_settled(&state, 1000).await, "narration should persist");

    let before = fetch_body(&app, "/fragment/story-log").await;
    let before_count = before.matches("class=\"log-entry").count();
    assert!(
        before_count >= 2,
        "the fragment must render at least 2 entries to delete one: {before}"
    );

    let resp = post_empty(&app, "/history/delete").await;
    assert_eq!(resp.status(), StatusCode::OK);

    let after = fetch_body(&app, "/fragment/story-log").await;
    let after_count = after.matches("class=\"log-entry").count();
    assert_eq!(
        after_count,
        before_count - 1,
        "deleting must drop exactly one rendered log entry"
    );
}

/// The `.log-entry` blocks of the fragment, as the Message id and the markup of each.
fn log_entries(body: &str) -> Vec<(u64, &str)> {
    body.split(r#"<div class="log-entry"#)
        .skip(1)
        .map(|entry| {
            let id = entry
                .split(r#"data-id=""#)
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .and_then(|id| id.parse().ok())
                .expect("every log entry names its Message");
            (id, entry)
        })
        .collect()
}

/// A Game whose story holds 53 Messages: its opening and 52 padded Narrations.
fn long_story_app() -> (
    axum::Router,
    chronicler_engine::adapters::driving::http::AppState,
) {
    let storage = Arc::new(empty_sqlite_storage());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    append_narrations(&storage, active_game_of(&state), 52, |index| {
        format!("Padded Narration {index}.")
    });
    assert_eq!(story_of(&state).len(), 53, "the story holds 53 Messages");
    (app, state)
}

// [docs/specs/story_log.md] SCENARIO: 8.11
#[tokio::test]
async fn test_the_story_log_of_a_long_story_ends_at_the_last_message() {
    let (app, state) = long_story_app();
    let story = story_of(&state);

    let body = fetch_body(&app, "/fragment/story-log").await;

    let entries = log_entries(&body);
    assert_eq!(entries.len(), 50, "the window holds 50 entries: {body}");
    assert_eq!(
        entries.first().map(|(id, _)| *id),
        Some(story[3].id),
        "the first entry is the fourth stored Message"
    );
    assert_eq!(
        entries.last().map(|(id, _)| *id),
        story.last().map(|message| message.id),
        "the last entry is the last stored Message"
    );
    for (position, (id, markup)) in entries.iter().enumerate() {
        let is_last = position == entries.len() - 1;
        assert_eq!(
            markup.contains("delete-btn"),
            is_last,
            "only the last entry (id {id}) carries Delete"
        );
        assert_eq!(
            markup.contains(r#"title="Retry""#),
            is_last,
            "only the last entry (id {id}) carries Retry"
        );
    }
}

// [docs/specs/story_log.md] SCENARIO: 8.12
#[tokio::test]
async fn test_delete_last_in_a_long_story_removes_the_entry_the_player_sees() {
    let (app, state) = long_story_app();
    let story = story_of(&state);
    let before = fetch_body(&app, "/fragment/story-log").await;
    assert_eq!(log_entries(&before).len(), 50);

    let resp = post_empty(&app, "/history/delete").await;
    assert_eq!(resp.status(), StatusCode::OK);

    let after = fetch_body(&app, "/fragment/story-log").await;
    let entries = log_entries(&after);
    assert_eq!(entries.len(), 50, "the window stays full: {after}");
    let deleted = story[52].id;
    assert_eq!(
        entries.last().map(|(id, _)| *id),
        Some(story[51].id),
        "the last entry is the Message before the deleted one"
    );
    assert!(
        entries.iter().all(|(id, _)| *id != deleted),
        "no entry holds the deleted Message"
    );
    assert_eq!(
        entries.first().map(|(id, _)| *id),
        Some(story[2].id),
        "the first entry is the third stored Message"
    );
}

// The shell swaps this fragment into its own `#story-log`, so a second
// container here would nest on every swap.
// [docs/specs/story_log.md] SCENARIO: 8.5
#[tokio::test]
async fn test_story_log_fragment_declares_no_log_container() {
    // The seeded Game's opening Narration is the entry the fragment renders.
    let app = TestAppBuilder::default_app();

    let body = fetch_body(&app, "/fragment/story-log").await;

    assert!(
        body.contains(r#"class="log-entry"#),
        "fragment must still render the entries: {body}"
    );
    assert!(
        !body.contains(r#"id="story-log""#),
        "fragment must not declare a second #story-log: {body}"
    );
    assert!(
        !body.contains(r#"class="story-log""#),
        "fragment must not wrap entries in a .story-log container: {body}"
    );
}

// [docs/specs/story_log.md] SCENARIO: 8.6
#[tokio::test]
async fn test_story_log_icon_buttons_have_accessible_names() {
    // The Action's Trigger fires but its continuation fails, so the main Narration
    // keeps the trigger context that offers Retrigger on the last entry.
    let data = TestDataBuilder::default_test()
        .npcs(vec![TestNpc::with_times_met_trigger(
            "npc_1",
            "Test NPC",
            ComparisonOperator::Eq,
            0,
        )])
        .build();
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(
            ScriptedModel::new()
                .narration("You look around.")
                .analysis(analysis_report(&["npc_1"], None))
                .failed(Role::Trigger),
        ))
        .build_with_state();
    assert!(post_action(&app, "look around").await.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "the Action should settle after its failed continuation"
    );

    let body = fetch_body(&app, "/fragment/story-log").await;

    for control in ["edit-btn", "delete-btn", "retrigger-btn", "swipe-btn"] {
        assert!(
            body.contains(control),
            "the fixture must render {control}, or the check below proves nothing: {body}"
        );
    }
    let unnamed = icon_buttons_without_matching_name(&body);
    assert!(
        unnamed.is_empty(),
        "icon-only buttons need an aria-label equal to their title: {unnamed:?}"
    );
    let exposed = svgs_not_hidden_from_assistive_technology(&body);
    assert!(
        exposed.is_empty(),
        "icons must carry aria-hidden=\"true\": {exposed:?}"
    );
}

// [docs/specs/story_log.md] SCENARIO: 8.8
#[tokio::test]
async fn test_story_log_shows_the_text_while_the_analysis_holds_the_job() {
    // The text save runs before the analysis, so the story log shows the text
    // while the analysis call is still held.
    let model = Arc::new(
        ScriptedModel::new()
            .narration("The room waits.")
            .analysis(analysis_report(&[], None)),
    );
    let hold = model.hold(Role::Quantifier);
    let (app, state) = TestAppBuilder::default_test()
        .model(Arc::clone(&model))
        .build_with_state();

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the held analysis call should reach the model"
    );

    assert_eq!(
        fetch_generating_status(&app).await,
        "quantifying",
        "the status names the step the held call runs in"
    );
    let log = fetch_body(&app, "/fragment/story-log").await;
    assert!(
        log.contains("The room waits."),
        "the story log shows the text from the save on: {log}"
    );

    hold.release();
    assert!(
        wait_settled(&state, 1000).await,
        "the released Job should complete"
    );
}

// [docs/specs/story_log.md] SCENARIO: 8.13
#[tokio::test]
async fn test_in_a_long_story_the_oldest_entry_shows_its_room() {
    let (app, _state) = long_story_app();

    let body = fetch_body(&app, "/fragment/story-log").await;

    let entries = log_entries(&body);
    assert_eq!(entries.len(), 50, "the window holds 50 entries: {body}");
    let (first_id, first) = entries[0];
    assert!(
        first.contains(r#"<span class="location-header">Test Room</span>"#),
        "the oldest entry (id {first_id}) shows its room: {first}"
    );
    for (id, markup) in &entries[1..] {
        assert!(
            !markup.contains("location-header"),
            "entry {id} holds the room the entry before it holds, so it shows none"
        );
    }
}
