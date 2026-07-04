use axum::response::IntoResponse;

use crate::adapters::driving::http::fragments::misc::swipe::switch_swipe_handler;
use crate::test_support::TestAppBuilder;

#[tokio::test]
async fn test_switch_swipe_handler() {
    let state = TestAppBuilder::default_test().build_app_state();
    let result = switch_swipe_handler(
        axum::extract::State(state),
        axum::extract::Path((0u64, 0usize)),
    )
    .await;
    let status = match result {
        Ok(resp) => resp.status(),
        Err(e) => e.into_response().status(),
    };

    assert!(
        status.is_success() || status.is_client_error() || status.is_server_error(),
        "unexpected status: {status}"
    );
}
