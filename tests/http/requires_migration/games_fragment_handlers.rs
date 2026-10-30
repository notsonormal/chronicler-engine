//! HTTP E2E tests for the games list part of the screen.

use std::sync::Arc;

use chronicler_engine::TestAppBuilder;
use chronicler_engine::test_support::empty_sqlite_storage;

use crate::support::http_requests::fetch_part;

#[tokio::test]
async fn test_list_games_fragment_returns_html() {
    let app = TestAppBuilder::default_app();
    let body_str = fetch_part(&app, "games").await;
    assert!(
        body_str.contains("games-panel") || body_str.contains("games-list"),
        "Expected games panel in response: {body_str}"
    );
}

#[tokio::test]
async fn test_list_games_fragment_shows_active_game() {
    let app = TestAppBuilder::default_app();
    let body_str = fetch_part(&app, "games").await;

    assert!(
        body_str.contains("game-item"),
        "Expected game-item in games list: {body_str}"
    );
}

#[tokio::test]
async fn test_list_games_fragment_empty_list() {
    let storage = Arc::new(empty_sqlite_storage());
    let app = TestAppBuilder::default_test().storage(storage).build();

    let body_str = fetch_part(&app, "games").await;
    assert!(
        body_str.contains("games-list") || body_str.contains("No games"),
        "Expected games list container even if empty: {body_str}"
    );
}
