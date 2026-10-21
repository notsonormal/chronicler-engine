//! HTTP integration test for reset-handler error handling.

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use tower::ServiceExt;

use chronicler_engine::TestAppBuilder;
use chronicler_engine::adapters::driven::storage::{StorageCall, TestOverride};
use chronicler_engine::test_support::empty_sqlite_storage;

#[tokio::test]
async fn test_reset_handler_failure_returns_internal_server_error() {
    let storage = Arc::new(empty_sqlite_storage().with_failure(
        StorageCall::GetGame,
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
