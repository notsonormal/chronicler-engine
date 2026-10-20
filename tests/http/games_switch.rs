//! HTTP E2E tests for game switching (POST /games/:id/switch).

use axum::{body::Body, http::Request, http::StatusCode};
use tower::util::ServiceExt;

use chronicler_engine::test_support::active_game_of;
use chronicler_engine::TestAppBuilder;

use crate::support::app_wiring::two_game_app;

// [docs/specs/games.md] SCENARIO: 18.1
#[tokio::test]
async fn test_switch_game_handler_success() {
    let (app, state, _storage, other_id) = two_game_app();
    let req = Request::builder()
        .uri(format!("/games/{other_id}/switch"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("HX-Refresh").unwrap(),
        "true",
        "Should return HX-Refresh header"
    );
    assert_eq!(active_game_of(&state).0, other_id);
}

// [docs/specs/games.md] SCENARIO: 18.2
#[tokio::test]
async fn test_switch_game_handler_not_found() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/games/9999/switch")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Game not found"),
        "Expected 'Game not found' in error body: {body_str}"
    );
}
