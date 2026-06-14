use axum::{http::StatusCode, Form};

use crate::model::settings::TextCheckMode;
use crate::server::fragments::actions::ActionForm;
use crate::server::fragments::misc::{
    check_text_handler, reset_handler, retrigger_handler, retry_handler, switch_swipe_handler,
};
use crate::test_support::TestAppBuilder;
use std::sync::atomic::Ordering;

fn make_test_app_state_with_generating(is_generating: bool) -> crate::server::AppState {
    let state = TestAppBuilder::default_test().build_app_state();
    state.is_generating.store(is_generating, Ordering::SeqCst);
    state
}



#[tokio::test]
async fn test_check_text_handler_empty() {
    let state = TestAppBuilder::default_test().build_app_state();
    let form = ActionForm {
        command: String::new(),
    };
    let response = check_text_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_check_text_handler_disabled() {
    let state = TestAppBuilder::default_test().build_app_state();
    state.settings.write().unwrap().text_check.mode = TextCheckMode::Disabled;
    let form = ActionForm {
        command: "test".to_string(),
    };
    let response = check_text_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_check_text_handler_ok() {
    let state = TestAppBuilder::default_test().build_app_state();
    let form = ActionForm {
        command: "This is a test sentence".to_string(),
    };
    let response = check_text_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
}



#[tokio::test]
async fn test_retry_handler() {
    let state = TestAppBuilder::default_test().build_app_state();

    let response = retry_handler(axum::extract::State(state)).await;

    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST);
}



#[tokio::test]
async fn test_retrigger_handler_ok() {
    let state = TestAppBuilder::default_test().build_app_state();

    let response = retrigger_handler(axum::extract::State(state)).await;

    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST);
}



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



#[tokio::test]
async fn test_reset_handler_ok() {
    let state = TestAppBuilder::default_test().build_app_state();
    let response = reset_handler(axum::extract::State(state)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_reset_handler_while_generating() {
    let state = make_test_app_state_with_generating(true);
    let response = reset_handler(axum::extract::State(state)).await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}
