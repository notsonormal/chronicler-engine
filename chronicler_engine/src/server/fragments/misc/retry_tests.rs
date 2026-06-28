use axum::http::StatusCode;

use crate::server::fragments::misc::retry::retry_handler;
use crate::test_support::TestAppBuilder;

#[tokio::test]
async fn test_retry_handler() {
    let state = TestAppBuilder::default_test().build_app_state();

    let response = retry_handler(axum::extract::State(state)).await;

    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST);
}
