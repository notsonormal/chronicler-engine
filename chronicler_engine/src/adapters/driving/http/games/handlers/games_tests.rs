use axum::{http::StatusCode, response::IntoResponse};

use crate::adapters::driving::http::games::handlers::{list_games_fragment, switch_game_handler};
use crate::test_support::TestAppBuilder;

#[tokio::test]
async fn test_list_games_empty() {
    let state = TestAppBuilder::default_test().build_app_state();
    let response = list_games_fragment(axum::extract::State(state)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_switch_game_ok() {
    let state = TestAppBuilder::default_test().build_app_state();
    let app = state.application_service.clone();
    let _ = app.create_game("test-world", "test-persona");
    let games = app.list_games().unwrap();
    if let Some(game) = games.first() {
        let result =
            switch_game_handler(axum::extract::State(state), axum::extract::Path(game.id)).await;
        let status = match result {
            Ok(resp) => resp.status(),
            Err(e) => e.into_response().status(),
        };
        assert_eq!(status, StatusCode::OK);
    }
}
