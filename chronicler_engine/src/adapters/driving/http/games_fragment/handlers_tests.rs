use axum::{http::StatusCode};

use crate::adapters::driving::http::games_fragment::handlers::{
    list_games_fragment, switch_game_handler,
};
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
    let ctx = state
        .as_game_service_context()
        .expect("Failed to load context");
    let _ = state.application_service.create_game(ctx.clone());
    let games = state.application_service.list_games(ctx).unwrap();
    if let Some(game) = games.first() {
        let response =
            switch_game_handler(axum::extract::State(state), axum::extract::Path(game.id)).await;
        assert_eq!(response.status(), StatusCode::OK);
    }
}
