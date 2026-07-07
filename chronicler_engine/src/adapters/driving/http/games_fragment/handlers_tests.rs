use axum::{http::StatusCode, response::IntoResponse};

use crate::adapters::driving::http::games_fragment::handlers::{
    list_games_fragment, switch_game_handler,
};
use crate::adapters::driving::http::op_context_loader::load_op_context_for_active_game;
use crate::test_support::TestAppBuilder;

#[tokio::test]
async fn test_list_games_empty() {
    let state = TestAppBuilder::default_test().build_app_state();
    let ctx = load_op_context_for_active_game(&state).expect("failed to load context");
    let response = list_games_fragment(axum::extract::State(state), ctx).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_switch_game_ok() {
    let state = TestAppBuilder::default_test().build_app_state();
    let ctx = load_op_context_for_active_game(&state).expect("Failed to load context");
    let _ = state.application_service.create_game(ctx.clone());
    let games = state.application_service.list_games(ctx.clone()).unwrap();
    if let Some(game) = games.first() {
        let result = switch_game_handler(
            axum::extract::State(state),
            axum::extract::Path(game.id),
            ctx,
        )
        .await;
        let status = match result {
            Ok(resp) => resp.status(),
            Err(e) => e.into_response().status(),
        };
        assert_eq!(status, StatusCode::OK);
    }
}
