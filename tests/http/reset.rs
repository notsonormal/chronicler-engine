//! HTTP E2E tests for the reset endpoint (POST /reset).

use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::TestAppBuilder;

use crate::test_helpers::{post_action, post_empty, wait_idle};

// [docs/specs/reset.md] SCENARIO: 7.1
#[tokio::test]
async fn test_reset_clears_history_http() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();

    let _ = post_action(&app, "examine room").await;
    assert!(wait_idle(&state, 1000).await);
    let before = state.message_service.load_messages().unwrap();
    assert!(
        before.iter().any(|m| m.message_type == MessageType::Input),
        "history should have an Input before reset"
    );

    let resp = post_empty(&app, "/reset").await;
    assert!(resp.status().is_success());

    let after = state.message_service.load_messages().unwrap();
    let inputs: Vec<_> = after
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert!(
        inputs.is_empty(),
        "previous action's Input should be gone after reset; got {inputs:?}"
    );
    let narrations: Vec<_> = after
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .collect();
    assert_eq!(
        narrations.len(),
        1,
        "reset should leave only the fresh opening narration"
    );
}

// [docs/specs/reset.md] SCENARIO: 7.2
#[tokio::test]
async fn test_reset_then_execute_works_http() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();

    let _ = post_action(&app, "examine room").await;
    assert!(wait_idle(&state, 1000).await);

    let resp = post_empty(&app, "/reset").await;
    assert!(resp.status().is_success());

    let _ = post_action(&app, "look around").await;
    assert!(
        wait_idle(&state, 1000).await,
        "action after reset should complete"
    );

    let messages = state.message_service.load_messages().unwrap();
    let inputs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert_eq!(
        inputs.len(),
        1,
        "only the second action's input should exist"
    );
    assert_eq!(inputs[0].text(), "look around");
}
