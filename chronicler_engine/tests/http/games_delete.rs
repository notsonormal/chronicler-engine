//! HTTP E2E tests for game deletion (POST /games/:id/delete).

use std::sync::Arc;

use axum::{body::Body, http::Request, http::StatusCode};
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;
use chronicler_engine::test_support::TestPersona;

use super::test_helpers::seeded_storage_with_initial_game;

// [chronicler_engine/docs/specs/games_delete.md] SCENARIO: 11.1
#[tokio::test]
async fn test_delete_game_handler_success() {
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
        .uri(format!("/games/{other_id}/delete"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(storage.get_game(other_id).unwrap().is_none());
}

// [chronicler_engine/docs/specs/games_delete.md] SCENARIO: 11.2
#[tokio::test]
async fn test_delete_game_handler_active_game() {
    let (storage, _world_key, _persona_key, _game_id) = seeded_storage_with_initial_game();

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let active_game_id = storage.current_game_id();

    let req = Request::builder()
        .uri(format!("/games/{active_game_id}/delete"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Cannot delete active game"
    );

    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Cannot delete the active game"),
        "Expected 'Cannot delete the active game' in error body: {body_str}"
    );
}

// [chronicler_engine/docs/specs/games_delete.md] SCENARIO: 11.3
#[tokio::test]
async fn test_delete_game_handler_unknown_id_is_idempotent() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/games/99999999/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Deleting an unknown game id should succeed idempotently"
    );
}
