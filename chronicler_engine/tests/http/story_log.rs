//! HTTP E2E tests for the story-log delete endpoint (POST /history/delete).

use axum::http::StatusCode;

use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::TestAppBuilder;

use crate::test_helpers::{post_action, post_empty, wait_idle};

// [chronicler_engine/docs/specs/story_log.md] SCENARIO: 8.1
#[tokio::test]
async fn test_delete_last_between_actions_http() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();

    // Action A → narration A persisted
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

    // Delete-last removes narration A (the last message)
    let resp = post_empty(&app, "/history/delete").await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Action B → narration B persisted
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

// [chronicler_engine/docs/specs/story_log.md] SCENARIO: 8.2
#[tokio::test]
async fn test_delete_mid_sequence_http() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();

    // Action A → narration A
    let _ = post_action(&app, "examine room").await;
    assert!(wait_idle(&state, 1000).await);
    // Action B → narration B (now the last message)
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

    // Delete-last removes narration B
    let resp = post_empty(&app, "/history/delete").await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Action C → narration C
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

// [chronicler_engine/docs/specs/story_log.md] SCENARIO: 8.3
#[tokio::test]
async fn test_delete_input_then_retry_fails_gracefully_http() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();

    let _ = post_action(&app, "examine room").await;
    assert!(wait_idle(&state, 1000).await);

    // Delete the last message (the narration). The Input remains but no anchor narration.
    let resp = post_empty(&app, "/history/delete").await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Retry with no anchor narration must not leave state generating.
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
