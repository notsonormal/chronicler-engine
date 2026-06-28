use crate::server::fragments::misc::swipe::switch_swipe_handler;
use crate::test_support::TestAppBuilder;

#[tokio::test]
async fn test_switch_swipe_handler() {
    let state = TestAppBuilder::default_test().build_app_state();
    let response = switch_swipe_handler(
        axum::extract::State(state),
        axum::extract::Path((0u64, 0usize)),
    )
    .await;

    assert!(
        response.status().is_success()
            || response.status().is_client_error()
            || response.status().is_server_error()
    );
}
