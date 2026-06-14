//! [DOC: docs/system/dashboard.md]
//! Integration tests for games_fragment handlers

use std::sync::Arc;

use axum::{body::Body, http::Request};
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;
use chronicler_engine::storage::Storage;

use super::test_helpers::fetch_body;

/// Test list_games_fragment returns HTML with games panel.
#[tokio::test]
async fn test_list_games_fragment_returns_html() {
    let app = TestAppBuilder::default_app();
    let req = Request::builder()
        .uri("/fragment/games")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("games-panel") || body_str.contains("games-list"),
        "Expected games panel in response: {body_str}"
    );
}

/// Test list_games_fragment shows active game indicator.
#[tokio::test]
async fn test_list_games_fragment_shows_active_game() {
    let app = TestAppBuilder::default_app();
    let body_str = fetch_body(app.clone(), "/fragment/games").await;

    // Should show the active game - default test app seeds a game
    assert!(
        body_str.contains("game-item"),
        "Expected game-item in games list: {body_str}"
    );
}

/// Test create_game_handler with empty world_key returns error.
#[tokio::test]
async fn test_create_game_handler_empty_world_key() {
    let app = TestAppBuilder::default_app();

    let form_data = "world_key=";
    let req = Request::builder()
        .uri("/games")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    // Should return error for empty world_key
    assert!(
        response.status().is_client_error() || response.status().is_server_error(),
        "Expected error for empty world_key: {:?}",
        response.status()
    );
}

/// Test games list is empty when no games exist.
#[tokio::test]
async fn test_list_games_fragment_empty_list() {
    let storage = Arc::new(Storage::new_in_memory());
    let app = TestAppBuilder::default_test().storage(storage).build();

    let req = Request::builder()
        .uri("/fragment/games")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert!(response.status().is_success());

    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("games-list") || body_str.contains("No games"),
        "Expected games list container even if empty: {body_str}"
    );
}
