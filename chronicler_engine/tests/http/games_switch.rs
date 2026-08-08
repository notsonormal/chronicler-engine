//! HTTP E2E tests for game switching (POST /games/:id/switch).

use std::sync::Arc;

use axum::{body::Body, http::Request, http::StatusCode};
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;
use chronicler_engine::test_support::TestPersona;

use super::test_helpers::seeded_storage_with_initial_game;

// [chronicler_engine/docs/specs/games_switch.md] SCENARIO: 10.1
#[tokio::test]
async fn test_switch_game_handler_success() {
    let (storage, _world_key, persona_key, _initial_game_id) = seeded_storage_with_initial_game();

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let other_id = storage
        .create_game(
            "Test World",
            "Test World",
            &persona_key,
            &TestPersona::standard().sheet.name,
            "Test World_2026-01-01_1",
        )
        .unwrap();
    assert_ne!(other_id, storage.current_game_id());

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
    assert_eq!(storage.current_game_id(), other_id);
}

// [chronicler_engine/docs/specs/games_switch.md] SCENARIO: 10.2
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
