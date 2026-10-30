//! HTTP E2E tests for game deletion (POST /games/:id/delete).

use std::sync::Arc;

use axum::{body::Body, http::Request, http::StatusCode};
use tower::util::ServiceExt;

use chronicler_engine::test_support::active_game_of;
use chronicler_engine::TestAppBuilder;

use crate::support::app_wiring::two_game_app;
use crate::support::http_fixtures::seeded_storage_with_initial_game;
use crate::support::http_requests::{assert_answers_nothing, fetch_part};

// [docs/specs/games.md] SCENARIO: 19.1
#[tokio::test]
async fn test_delete_game_handler_success() {
    let (app, _state, _storage, other_id) = two_game_app();
    let req = Request::builder()
        .uri(format!("/games/{other_id}/delete"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let panel = fetch_part(&app, "games").await;
    assert!(
        !panel.contains(&format!("data-id=\"{other_id}\"")),
        "the deleted game {other_id} must not render in the games panel"
    );
}

// [docs/specs/games.md] SCENARIO: 19.2
#[tokio::test]
async fn test_delete_game_handler_active_game() {
    let (storage, _world_key, _persona_key, _game_id) = seeded_storage_with_initial_game();

    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();

    let active_game_id = active_game_of(&state);

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

// [docs/specs/games.md] SCENARIO: 19.3
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

async fn delete_game_answering_nothing(app: &axum::Router, id: u64) {
    let req = Request::builder()
        .uri(format!("/games/{id}/delete"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_answers_nothing(response).await;
}

// [docs/specs/games.md] SCENARIO: 19.4
#[tokio::test]
async fn test_delete_game_answers_nothing_and_the_panel_shows_the_empty_state_after_the_last() {
    let (app, state, _storage, _other_id) = two_game_app();

    let active_id = active_game_of(&state);

    let saved_ids: Vec<u64> = state
        .game_catalogue
        .list_games()
        .unwrap()
        .into_iter()
        .map(|g| g.id)
        .filter(|id| *id != active_id.0)
        .collect();
    assert_eq!(
        saved_ids.len(),
        2,
        "the fixture plus one created game must leave two saved games"
    );

    delete_game_answering_nothing(&app, saved_ids[0]).await;
    let panel = fetch_part(&app, "games").await;
    assert!(
        panel.contains(&format!("data-id=\"{}\"", saved_ids[1])),
        "the panel must still list the saved game that was not deleted: {panel}"
    );
    assert!(
        !panel.contains("No other saved games."),
        "a panel that lists a saved game shows no empty state: {panel}"
    );

    delete_game_answering_nothing(&app, saved_ids[1]).await;
    let panel = fetch_part(&app, "games").await;
    assert!(
        panel.contains("No other saved games."),
        "the panel must show the empty state once the last saved game is gone: {panel}"
    );
}
