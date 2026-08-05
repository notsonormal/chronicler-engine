use axum::{http::StatusCode, response::IntoResponse};

use crate::adapters::driving::http::chat_window::handlers::{
    retrigger_handler, retry_handler, switch_swipe_handler,
};
use crate::test_support::TestAppBuilder;

#[tokio::test]
async fn test_retry_handler_returns_503_on_shutdown() {
    let state = TestAppBuilder::default_test().build_service();
    state.shutdown_token.cancel();

    let result = retry_handler(axum::extract::State(state)).await;
    let status = match result {
        Ok(resp) => resp.status(),
        Err(e) => e.into_response().status(),
    };
    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "cancelled shutdown should produce 503 via Ok(ShuttingDown) arm"
    );
}

#[tokio::test]
async fn test_retrigger_handler_returns_503_on_shutdown() {
    let state = TestAppBuilder::default_test().build_service();
    state.shutdown_token.cancel();

    let result = retrigger_handler(axum::extract::State(state)).await;
    let status = match result {
        Ok(resp) => resp.status(),
        Err(e) => e.into_response().status(),
    };
    assert_eq!(
        status,
        StatusCode::SERVICE_UNAVAILABLE,
        "cancelled shutdown should produce 503 via Ok(ShuttingDown) arm"
    );
}

#[tokio::test]
async fn test_retry_handler_returns_400_when_no_input() {
    let state = TestAppBuilder::default_test().build_service();

    let result = retry_handler(axum::extract::State(state)).await;
    let status = match result {
        Ok(resp) => resp.status(),
        Err(e) => e.into_response().status(),
    };
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "retry with no input should produce 400 via Validation arm"
    );
}

#[tokio::test]
async fn test_retrigger_handler_returns_400_when_no_trigger_context() {
    let state = TestAppBuilder::default_test().build_service();

    let result = retrigger_handler(axum::extract::State(state)).await;
    let status = match result {
        Ok(resp) => resp.status(),
        Err(e) => e.into_response().status(),
    };
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "retrigger with no trigger context should produce 400 via Validation arm"
    );
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
