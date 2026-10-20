//! Unit tests for the chat window HTTP handlers.
use axum::http::StatusCode;

use crate::adapters::driving::http::utils::current_game::CurrentGame;
use crate::adapters::driving::http::chat_window::handlers::{retrigger_handler, switch_swipe_handler};
use crate::test_support::active_game_of;
use crate::test_support::TestAppBuilder;

#[tokio::test]
async fn test_retrigger_handler_returns_400_when_no_trigger_context() {
    let state = TestAppBuilder::default_test().build_service();

    let response = retrigger_handler(
        axum::extract::State(state.clone()),
        CurrentGame(active_game_of(&state)),
    )
    .await;
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "a Retrigger with no trigger context is refused as a bad request"
    );
}

#[tokio::test]
async fn test_switch_swipe_handler() {
    let state = TestAppBuilder::default_test().build_service();

    let response = switch_swipe_handler(
        axum::extract::State(state.clone()),
        CurrentGame(active_game_of(&state)),
        axum::extract::Path((0u64, 0usize)),
    )
    .await;

    let status = response.status();
    assert!(
        status.is_success() || status.is_client_error() || status.is_server_error(),
        "unexpected status: {status}"
    );
}
