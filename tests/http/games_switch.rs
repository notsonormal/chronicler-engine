//! HTTP E2E tests for game switching (POST /games/:id/switch).

use std::sync::Arc;

use axum::{body::Body, http::Request, http::StatusCode};
use tower::util::ServiceExt;

use chronicler_engine::adapters::driven::storage::{StorageCall, TestOverride};
use chronicler_engine::test_support::{
    active_game_of, empty_sqlite_storage, seed_game_row, test_new_game, TestPersona,
};
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

// [docs/specs/games.md] SCENARIO: 18.5
#[tokio::test]
async fn test_switch_the_storage_rejects_keeps_the_active_game_http() {
    let (storage, failures) = empty_sqlite_storage().with_test_failures();
    let storage = Arc::new(storage);
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let active = active_game_of(&state);
    let other_id = seed_game_row(
        &storage,
        &test_new_game(
            "Test World",
            "Test World",
            "test_player",
            &TestPersona::standard().sheet.name,
            "Test World_2026-01-01_1",
        ),
    )
    .0;
    assert_ne!(
        other_id, active.0,
        "the second saved Game is not the active one"
    );

    failures.set(
        StorageCall::TouchGameUpdatedAt,
        TestOverride::internal("touch_game_updated_at failure"),
    );
    let req = Request::builder()
        .uri(format!("/games/{other_id}/switch"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(
        response.headers().get("HX-Refresh").is_none(),
        "a rejected switch must not refresh the page"
    );
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("touch_game_updated_at failure"),
        "the error fragment must name the storage failure: {body_str}"
    );
    assert_eq!(
        active_game_of(&state),
        active,
        "a rejected switch leaves the active Game where it was"
    );
}
