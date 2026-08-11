//! HTTP integration test for reset-handler error handling.

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use tower::ServiceExt;

use chronicler_engine::TestAppBuilder;
use chronicler_engine::adapters::driven::storage::{Storage, TestOverride};

#[tokio::test]
async fn test_reset_handler_failure_returns_internal_server_error() {
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        "get_game",
        TestOverride::internal("simulated reset failure"),
    ));
    let app = TestAppBuilder::default_test().storage(storage).build();

    let request = Request::builder()
        .uri("/reset")
        .method(Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
