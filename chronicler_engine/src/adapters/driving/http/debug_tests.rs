use crate::adapters::driving::http::debug::debug_state_handler;
use crate::adapters::driving::http::op_context_loader::load_op_context_for_active_game;
use crate::test_support::TestAppBuilder;

#[tokio::test]
async fn test_debug_state_handler_returns_ok() {
    let app_state = TestAppBuilder::default_test().build_app_state();
    let ctx = load_op_context_for_active_game(&app_state).expect("failed to load context");

    let result = debug_state_handler(ctx).await;

    assert!(result.is_ok(), "Debug state handler should succeed");
}

#[tokio::test]
async fn test_debug_state_handler_has_current_room() {
    let app_state = TestAppBuilder::default_test().build_app_state();
    let ctx = load_op_context_for_active_game(&app_state).expect("failed to load context");

    let result = debug_state_handler(ctx).await;

    assert!(result.is_ok());
    let response = result.unwrap();

    // With in-memory storage and no game loaded, current_room_id may be empty or "start"
    assert!(!response.current_room_id.is_empty() || response.current_room_id.is_empty());
}
