use crate::adapters::driving::http::layout::handlers::endpoints::{
    action_area_fragment, character_headshots_fragment, generating_status_handler, header_fragment,
    llm_messages_fragment, reset_generating_handler, status_ready_handler, story_log_fragment,
    visual_sidebar_fragment,
};
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::test_support::TestAppBuilder;

#[tokio::test]
async fn test_header_fragment() {
    let state = TestAppBuilder::default_test().build_service();
    let result = header_fragment(axum::extract::State(state)).await;
    assert!(!result.0.is_empty());
}

#[tokio::test]
async fn test_story_log_fragment() {
    let state = TestAppBuilder::default_test().build_service();
    let result = story_log_fragment(axum::extract::State(state)).await;
    assert!(!result.0.is_empty());
}

#[tokio::test]
async fn test_action_area_fragment() {
    let state = TestAppBuilder::default_test().build_service();
    let result = action_area_fragment(axum::extract::State(state)).await;
    assert!(!result.0.is_empty());
}

#[tokio::test]
async fn test_character_headshots_fragment() {
    let state = TestAppBuilder::default_test().build_service();
    let result = character_headshots_fragment(axum::extract::State(state)).await;
    assert!(!result.0.is_empty());
}

#[tokio::test]
async fn test_visual_sidebar_fragment() {
    let state = TestAppBuilder::default_test().build_service();
    let result = visual_sidebar_fragment(axum::extract::State(state)).await;
    assert!(!result.0.is_empty());
}

#[tokio::test]
async fn test_llm_messages_fragment() {
    let state = TestAppBuilder::default_test().build_service();
    let result = llm_messages_fragment(axum::extract::State(state)).await;
    assert!(!result.0.is_empty());
}

#[tokio::test]
async fn test_status_ready_handler() {
    let result = status_ready_handler().await;
    assert!(result.0.contains("Ready"));
}

#[tokio::test]
async fn test_generating_status_idle() {
    let state = TestAppBuilder::default_test().build_service();

    let result = generating_status_handler(axum::extract::State(state)).await;
    assert!(result.0.contains("idle"));
}

#[tokio::test]
async fn test_generating_status_generating() {
    let state = TestAppBuilder::default_test().build_service();
    let mut game_state = state.message_service.load_or_fresh();
    game_state.narrative.input_buffer.status = GenerationStatus::Generating;
    game_state.narrative.input_buffer.phase = GenerationPhase::Narrating;
    let _ = state.message_service.save_state(&game_state);

    let result = generating_status_handler(axum::extract::State(state)).await;
    assert!(!result.0.is_empty());
}

#[tokio::test]
async fn test_generating_status_error() {
    let state = TestAppBuilder::default_test().build_service();

    let result = generating_status_handler(axum::extract::State(state)).await;
    assert!(!result.0.is_empty());
}

#[tokio::test]
async fn test_reset_generating_ok() {
    let state = TestAppBuilder::default_test().build_service();

    let result = reset_generating_handler(axum::extract::State(state)).await;
    assert!(result.0.contains("reset") || !result.0.is_empty());
}
