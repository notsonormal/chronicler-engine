//! HTTP E2E tests for the story-log fragment: delete flows through POST /history/delete, the fragment's shape, and the controls each entry renders.

use axum::http::StatusCode;

use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::TestStoredTriggerContext;
use chronicler_engine::TestAppBuilder;

use crate::support::http_assertions::{
    icon_buttons_without_matching_name, svgs_not_hidden_from_assistive_technology,
};
use crate::support::http_requests::{fetch_body, post_action, post_empty, wait_idle};

// [docs/specs/story_log.md] SCENARIO: 8.1
#[tokio::test]
async fn test_delete_last_between_actions_http() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();

    let resp = post_action(&app, "examine room").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "action A should complete");
    let narration_a = state
        .message_service
        .load_messages()
        .unwrap()
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .map(|m| m.text().to_string())
        .next()
        .expect("narration A should be persisted");

    let resp = post_empty(&app, "/history/delete").await;
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = post_action(&app, "look around").await;
    assert!(resp.status().is_success());
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
    let (app, state) = TestAppBuilder::default_test().build_with_state();

    let _ = post_action(&app, "examine room").await;
    assert!(wait_idle(&state, 1000).await);
    let _ = post_action(&app, "look around").await;
    assert!(wait_idle(&state, 1000).await);
    let narration_b = state
        .message_service
        .load_messages()
        .unwrap()
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .map(|m| m.text().to_string())
        .next_back()
        .expect("narration B should be the latest narration");

    let resp = post_empty(&app, "/history/delete").await;
    assert_eq!(resp.status(), StatusCode::OK);

    let _ = post_action(&app, "check door").await;
    assert!(wait_idle(&state, 1000).await);

    let messages = state.message_service.load_messages().unwrap();
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

// [docs/specs/story_log.md] SCENARIO: 8.3
#[tokio::test]
async fn test_delete_input_then_retry_fails_gracefully_http() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();

    let _ = post_action(&app, "examine room").await;
    assert!(wait_idle(&state, 1000).await);

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
        wait_idle(&state, 1000).await,
        "retry after delete should not leave state generating"
    );
}

// [docs/specs/story_log.md] SCENARIO: 8.4
#[tokio::test]
async fn test_delete_removes_entry_from_fragment_http() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();

    let _ = post_action(&app, "examine room").await;
    assert!(wait_idle(&state, 1000).await, "narration should persist");

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

// The shell swaps this fragment into its own `#story-log`, so a second
// container here would nest on every swap.
// [docs/specs/story_log.md] SCENARIO: 8.5
#[tokio::test]
async fn test_story_log_fragment_declares_no_log_container() {
    let app = TestAppBuilder::default_test()
        .log("You look around.", MessageType::Narration)
        .build();

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
    let app = TestAppBuilder::default_test()
        .last_trigger(TestStoredTriggerContext::standard())
        .log("look around", MessageType::Input)
        .log("You look around.", MessageType::Narration)
        .build();

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
