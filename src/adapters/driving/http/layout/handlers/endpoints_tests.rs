use crate::adapters::driving::http::utils::current_game::CurrentGame;
use crate::adapters::driving::http::layout::handlers::endpoints::{
    character_headshots_fragment, busy_status_handler, header_fragment, llm_messages_fragment,
    reset_busy_handler, story_log_fragment, visual_sidebar_fragment,
};
use crate::test_support::{active_game_of, body_text, TestAppBuilder};

#[tokio::test]
async fn test_header_fragment() {
    let state = TestAppBuilder::default_test().build_service();
    let result = body_text(
        header_fragment(
            axum::extract::State(state.clone()),
            CurrentGame(active_game_of(&state)),
        )
        .await,
    )
    .await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_story_log_fragment() {
    let state = TestAppBuilder::default_test().build_service();
    let result = body_text(
        story_log_fragment(
            axum::extract::State(state.clone()),
            CurrentGame(active_game_of(&state)),
        )
        .await,
    )
    .await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_character_headshots_fragment() {
    let state = TestAppBuilder::default_test().build_service();
    let result = body_text(
        character_headshots_fragment(
            axum::extract::State(state.clone()),
            CurrentGame(active_game_of(&state)),
        )
        .await,
    )
    .await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_visual_sidebar_fragment() {
    let state = TestAppBuilder::default_test().build_service();
    let result = body_text(
        visual_sidebar_fragment(
            axum::extract::State(state.clone()),
            CurrentGame(active_game_of(&state)),
        )
        .await,
    )
    .await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_llm_messages_fragment() {
    let state = TestAppBuilder::default_test().build_service();
    let result = body_text(llm_messages_fragment(axum::extract::State(state)).await).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_busy_status_idle() {
    let state = TestAppBuilder::default_test().build_service();

    let result = busy_status_handler(
        axum::extract::State(state.clone()),
        Some(CurrentGame(active_game_of(&state))),
    )
    .await;
    assert!(result.0.contains("idle"));
}

#[tokio::test]
async fn test_busy_status_failed() {
    let state = TestAppBuilder::default_test().build_service();

    let result = busy_status_handler(
        axum::extract::State(state.clone()),
        Some(CurrentGame(active_game_of(&state))),
    )
    .await;
    assert!(!result.0.is_empty());
}

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
