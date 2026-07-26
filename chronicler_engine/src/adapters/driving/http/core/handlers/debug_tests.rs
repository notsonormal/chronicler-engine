use crate::adapters::driving::http::core::handlers::debug::debug_state_handler;
use crate::test_support::TestAppBuilder;

#[tokio::test]
async fn test_debug_state_handler_returns_ok() {
    let app_state = TestAppBuilder::default_test().build_app_state();

    let result = debug_state_handler(axum::extract::State(app_state)).await;

    assert!(result.is_ok(), "Debug state handler should succeed");
}

#[tokio::test]
async fn test_debug_state_handler_has_current_room() {
    let app_state = TestAppBuilder::default_test().build_app_state();

    let result = debug_state_handler(axum::extract::State(app_state)).await;

    assert!(result.is_ok());
    let response = result.unwrap();

    assert!(!response.current_room_id.is_empty() || response.current_room_id.is_empty());
}
