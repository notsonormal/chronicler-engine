//! HTTP E2E tests for the games list fragment (GET /fragment/games).

use std::sync::Arc;

use axum::{body::Body, http::Request};
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;
use chronicler_engine::adapters::driven::storage::Storage;

use super::test_helpers::fetch_body;

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

#[tokio::test]
async fn test_list_games_fragment_shows_active_game() {
    let app = TestAppBuilder::default_app();
    let body_str = fetch_body(&app, "/fragment/games").await;

    assert!(
        body_str.contains("game-item"),
        "Expected game-item in games list: {body_str}"
    );
}

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
