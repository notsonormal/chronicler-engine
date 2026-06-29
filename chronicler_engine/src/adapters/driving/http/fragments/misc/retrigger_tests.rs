use axum::http::StatusCode;

use crate::adapters::driving::http::fragments::misc::retrigger::retrigger_handler;
use crate::test_support::TestAppBuilder;

#[tokio::test]
async fn test_retrigger_handler() {
    let state = TestAppBuilder::default_test().build_app_state();

    let response = retrigger_handler(axum::extract::State(state)).await;

    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST);
}
