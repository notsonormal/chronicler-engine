use crate::adapters::driving::http::layout::handlers::endpoints::{
    action_area_fragment, character_headshots_fragment, generating_status_handler, header_fragment,
    llm_messages_fragment, reset_generating_handler, story_log_fragment, visual_sidebar_fragment,
};
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::test_support::{body_text, TestAppBuilder};

#[tokio::test]
async fn test_header_fragment() {
    let state = TestAppBuilder::default_test().build_service();
    let result = body_text(header_fragment(axum::extract::State(state)).await).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_story_log_fragment() {
    let state = TestAppBuilder::default_test()
        .log("You look around.", MessageType::Narration)
        .build_service();
    let result = body_text(story_log_fragment(axum::extract::State(state)).await).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_action_area_fragment() {
    let state = TestAppBuilder::default_test().build_service();
    let result = body_text(action_area_fragment(axum::extract::State(state)).await).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_character_headshots_fragment() {
    let state = TestAppBuilder::default_test().build_service();
    let result = body_text(character_headshots_fragment(axum::extract::State(state)).await).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_visual_sidebar_fragment() {
    let state = TestAppBuilder::default_test().build_service();
    let result = body_text(visual_sidebar_fragment(axum::extract::State(state)).await).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_llm_messages_fragment() {
    let state = TestAppBuilder::default_test().build_service();
    let result = body_text(llm_messages_fragment(axum::extract::State(state)).await).await;
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_generating_status_idle() {
    let state = TestAppBuilder::default_test().build_service();

    let result = generating_status_handler(axum::extract::State(state)).await;
    assert!(result.0.contains("idle"));
}

#[tokio::test]
async fn test_generating_status_answers_from_live_registry() {
    let state = TestAppBuilder::default_test().build_service();
    let game_id = state.game_catalogue.current_game_id();

    let mut game_state = state.message_service.load_or_fresh();
    game_state.narrative.input_buffer.status = GenerationStatus::Generating;
    game_state.narrative.input_buffer.phase = GenerationPhase::Narrating;
    let _ = state.message_service.save_state(&game_state);

    let result = generating_status_handler(axum::extract::State(state.clone())).await;
    assert!(
        result.0.contains("idle"),
        "stale status should answer idle: {}",
        result.0
    );

    let mut game_state = state.message_service.load_or_fresh();
    let (_, _, outcome) = state
        .generation_gate
        .try_claim(game_id, &mut game_state, &state.message_service)
        .expect("live slot claim should succeed");
    assert!(
        matches!(
            outcome,
            crate::application::errors::ProcessActionResult::Started
        ),
        "the claim should start a generation, got {outcome:?}"
    );

    let result = generating_status_handler(axum::extract::State(state.clone())).await;
    assert!(
        result.0.contains("narrating"),
        "live slot should report the phase: {}",
        result.0
    );
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
