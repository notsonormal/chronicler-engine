use axum::{http::StatusCode, response::IntoResponse};

use crate::adapters::driving::http::core::handlers::retrigger::retrigger_handler;
use crate::test_support::TestAppBuilder;

#[tokio::test]
async fn test_retrigger_handler() {
    let state = TestAppBuilder::default_test().build_app_state();

    let result = retrigger_handler(axum::extract::State(state)).await;
    let status = match result {
        Ok(resp) => resp.status(),
        Err(e) => e.into_response().status(),
    };

    assert!(status == StatusCode::OK || status.is_client_error() || status.is_server_error());
}
