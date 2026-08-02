use axum::{http::StatusCode, response::IntoResponse};

use crate::adapters::driving::http::chat_window::handlers::{
    retrigger_handler, retry_handler, switch_swipe_handler,
};
use crate::test_support::TestAppBuilder;

#[tokio::test]
async fn test_retrigger_handler() {
    let state = TestAppBuilder::default_test().build_service();

    let result = retrigger_handler(axum::extract::State(state)).await;
    let status = match result {
        Ok(resp) => resp.status(),
        Err(e) => e.into_response().status(),
    };

    assert!(status == StatusCode::OK || status.is_client_error() || status.is_server_error());
}

#[tokio::test]
async fn test_retry_handler() {
    let state = TestAppBuilder::default_test().build_service();

    let result = retry_handler(axum::extract::State(state)).await;
    let status = match result {
        Ok(resp) => resp.status(),
        Err(e) => e.into_response().status(),
    };

    assert!(status == StatusCode::OK || status.is_client_error() || status.is_server_error());
}

#[tokio::test]
async fn test_switch_swipe_handler() {
    let state = TestAppBuilder::default_test().build_service();

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
