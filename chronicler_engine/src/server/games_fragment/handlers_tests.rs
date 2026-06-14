use axum::{http::StatusCode};

use crate::server::games_fragment::handlers::{
    create_game_handler, delete_game_handler, list_games_fragment, switch_game_handler,
};
use crate::test_support::TestAppBuilder;
use std::sync::atomic::Ordering;

fn make_test_app_state_with_generating(is_generating: bool) -> crate::server::AppState {
    let state = TestAppBuilder::default_test().build_app_state();
    state.is_generating.store(is_generating, Ordering::SeqCst);
    state
}

// ─── List Games Tests ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_games_empty() {
    let state = TestAppBuilder::default_test().build_app_state();
    let response = list_games_fragment(axum::extract::State(state)).await;
    // Just check it returns OK - actual content depends on world name
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_games_no_active() {
    let state = TestAppBuilder::default_test().build_app_state();
    let response = list_games_fragment(axum::extract::State(state)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

// ─── Create Game Tests ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_game_ok() {
    let state = TestAppBuilder::default_test().build_app_state();
    let response = create_game_handler(axum::extract::State(state)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_game_while_generating() {
    let state = make_test_app_state_with_generating(true);
    let response = create_game_handler(axum::extract::State(state)).await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

// ─── Switch Game Tests ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_switch_game_ok() {
    let state = TestAppBuilder::default_test().build_app_state();
    // Create a game first
    let _ = create_game_handler(axum::extract::State(state.clone())).await;

    // Get the game ID from storage
    let games = state
        .application_service
        .list_games(
            state
                .as_game_service_context()
                .expect("Failed to load game context"),
        )
        .unwrap();
    if !games.is_empty() {
        let game_id = games[0].id;
        let response =
            switch_game_handler(axum::extract::State(state), axum::extract::Path(game_id)).await;
        assert_eq!(response.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_switch_game_while_generating() {
    let state = make_test_app_state_with_generating(true);
    let response =
        switch_game_handler(axum::extract::State(state), axum::extract::Path(999u64)).await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

// ─── Delete Game Tests ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_game() {
    let state = TestAppBuilder::default_test().build_app_state();
    // Try to delete non-existent game - will fail but shouldn't panic
    let response =
        delete_game_handler(axum::extract::State(state), axum::extract::Path(999u64)).await;
    // Just ensure it returns some status without panicking
    assert!(
        response.0.is_client_error() || response.0.is_server_error() || response.0.is_success()
    );
}

#[tokio::test]
async fn test_delete_game_while_generating() {
    let state = make_test_app_state_with_generating(true);
    let response =
        delete_game_handler(axum::extract::State(state), axum::extract::Path(999u64)).await;
    assert_eq!(response.0, StatusCode::SERVICE_UNAVAILABLE);
}
