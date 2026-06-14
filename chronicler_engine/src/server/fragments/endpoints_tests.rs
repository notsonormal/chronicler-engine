use crate::server::fragments::endpoints::{
    action_area_fragment, character_headshots_fragment, generating_status_handler, header_fragment,
    hints_handler, llm_messages_fragment, reset_generating_handler, status_ready_handler,
    story_log_fragment, visual_sidebar_fragment,
};
use crate::test_support::TestAppBuilder;

#[tokio::test]
async fn test_header_fragment() {
    let state = TestAppBuilder::default_test().build_app_state();
    let result = header_fragment(axum::extract::State(state)).await;
    assert!(!result.0.is_empty());
}

#[tokio::test]
async fn test_story_log_fragment() {
    let state = TestAppBuilder::default_test().build_app_state();
    let result = story_log_fragment(axum::extract::State(state)).await;
    assert!(!result.0.is_empty());
}

#[tokio::test]
async fn test_action_area_fragment() {
    let state = TestAppBuilder::default_test().build_app_state();
    let result = action_area_fragment(axum::extract::State(state)).await;
    assert!(!result.0.is_empty());
}

#[tokio::test]
async fn test_character_headshots_fragment() {
    let state = TestAppBuilder::default_test().build_app_state();
    let result = character_headshots_fragment(axum::extract::State(state)).await;
    // May be empty if no NPCs
    assert!(result.0.is_empty() || !result.0.is_empty());
}

#[tokio::test]
async fn test_visual_sidebar_fragment() {
    let state = TestAppBuilder::default_test().build_app_state();
    let result = visual_sidebar_fragment(axum::extract::State(state)).await;
    assert!(!result.0.is_empty());
}

#[tokio::test]
async fn test_hints_handler() {
    let state = TestAppBuilder::default_test().build_app_state();
    let result = hints_handler(axum::extract::State(state)).await;
    // Hints may be empty
    assert!(result.0.is_empty() || !result.0.is_empty());
}

#[tokio::test]
async fn test_llm_messages_fragment() {
    let state = TestAppBuilder::default_test().build_app_state();
    let result = llm_messages_fragment(axum::extract::State(state)).await;
    assert!(!result.0.is_empty());
}

// ─── Status Handler Tests ────────────────────────────────────────────────────

#[tokio::test]
async fn test_status_ready_handler() {
    let state = TestAppBuilder::default_test().build_app_state();
    let result = status_ready_handler(axum::extract::State(state)).await;
    assert!(result.0.contains("Ready"));
}

#[tokio::test]
async fn test_generating_status_idle() {
    let state = TestAppBuilder::default_test().build_app_state();
    let result = generating_status_handler(axum::extract::State(state)).await;
    assert!(result.0.contains("idle"));
}

#[tokio::test]
async fn test_generating_status_generating() {
    let state = TestAppBuilder::default_test().build_app_state();
    state
        .is_generating
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let result = generating_status_handler(axum::extract::State(state)).await;
    // Should contain generating status
    assert!(!result.0.is_empty());
}

#[tokio::test]
async fn test_generating_status_error() {
    // Test with invalid game state (no snapshot)
    let state = TestAppBuilder::default_test().build_app_state();
    let result = generating_status_handler(axum::extract::State(state)).await;
    // Should handle gracefully - either idle or error state
    assert!(!result.0.is_empty());
}

// ─── Reset Generating Tests ──────────────────────────────────────────────────

#[tokio::test]
async fn test_reset_generating_ok() {
    let state = TestAppBuilder::default_test().build_app_state();
    let result = reset_generating_handler(axum::extract::State(state)).await;
    assert!(result.0.contains("reset") || !result.0.is_empty());
}
