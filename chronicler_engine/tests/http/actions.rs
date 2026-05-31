use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use std::sync::Arc;
use tower::ServiceExt;

use chronicler_engine::TestAppBuilder;
use chronicler_engine::storage::{Operation, Storage, TestOverride};

#[tokio::test]
async fn test_action_handler_load_state_failure_graceful_degradation() {
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        Operation::LoadLatestSnapshot,
        TestOverride::internal("simulated load failure"),
    ));

    let app = TestAppBuilder::default_test().storage(storage).build();

    let req = Request::builder()
        .uri("/action")
        .method(Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("command=look"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    // Graceful degradation: falls back to fresh state, returns 200
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_handler_snapshot_save_failure() {
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        Operation::SaveSnapshot,
        TestOverride::internal("simulated save failure"),
    ));

    let app = TestAppBuilder::default_test().storage(storage).build();

    let req = Request::builder()
        .uri("/action")
        .method(Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("command=look"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_action_confirm_handler_load_state_failure_graceful_degradation() {
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        Operation::LoadLatestSnapshot,
        TestOverride::internal("simulated load failure"),
    ));

    let app = TestAppBuilder::default_test().storage(storage).build();

    let req = Request::builder()
        .uri("/action/confirm")
        .method(Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from("command=look"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    // Graceful degradation: falls back to fresh state, returns 200
    assert_eq!(response.status(), StatusCode::OK);
}
