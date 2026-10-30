use crate::adapters::driving::http::utils::current_game::CurrentGame;
use crate::adapters::driving::http::layout::handlers::reset_busy_handler;
use crate::test_support::{active_game_of, TestAppBuilder};

#[tokio::test]
async fn test_reset_busy_ok() {
    let state = TestAppBuilder::default_test().build_service();

    let result = reset_busy_handler(
        axum::extract::State(state.clone()),
        CurrentGame(active_game_of(&state)),
    )
    .await;
    assert_eq!(result.status(), axum::http::StatusCode::OK);
}
