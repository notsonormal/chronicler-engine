// Tests for `response.rs` HTTP response helpers. Covers response
// builders, error→response conversion, and ctx_or_error boundary.

use axum::body::to_bytes;
use axum::http::StatusCode;

use crate::application::application_service::ApplicationError;
use crate::adapters::driving::http::fragments::renderers::response::{
    app_err_to_response, app_err_to_tuple, bad_request, ctx_or_error, internal_error, ok,
    ok_refresh, service_unavailable, service_unavailable_generating,
};

async fn body_string(resp: axum::response::Response) -> String {
    let bytes = to_bytes(resp.into_body(), 16384).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap_or_default()
}

#[tokio::test]
async fn ok_returns_200_with_body() {
    let resp = ok("hello");
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_string(resp).await, "hello");
}

#[tokio::test]
async fn ok_refresh_sets_hx_refresh_header_and_empty_body() {
    let resp = ok_refresh();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get("HX-Refresh")
            .map(|v| v.to_str().unwrap()),
        Some("true")
    );
    assert_eq!(body_string(resp).await, "");
}

#[tokio::test]
async fn bad_request_returns_400_with_body() {
    let resp = bad_request("invalid input");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_string(resp).await, "invalid input");
}

#[tokio::test]
async fn internal_error_returns_500_with_body() {
    let resp = internal_error("boom");
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body_string(resp).await, "boom");
}

#[tokio::test]
async fn service_unavailable_returns_503_with_body() {
    let resp = service_unavailable("try later");
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body_string(resp).await, "try later");
}

#[tokio::test]
async fn service_unavailable_generating_contains_status_span() {
    let resp = service_unavailable_generating();
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = body_string(resp).await;
    assert!(body.contains("status wait"));
    assert!(body.contains("Generation in progress"));
}

#[tokio::test]
async fn app_err_to_response_validation_returns_400() {
    let resp = app_err_to_response(ApplicationError::Validation("bad input".into()));
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = body_string(resp).await;
    assert!(body.contains("bad input"));
}

#[tokio::test]
async fn app_err_to_response_concurrent_generation_returns_503() {
    let resp = app_err_to_response(ApplicationError::ConcurrentGeneration);
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn app_err_to_response_shutting_down_returns_503() {
    let resp = app_err_to_response(ApplicationError::ShuttingDown);
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = body_string(resp).await;
    assert!(body.contains("shutting down"));
}

#[tokio::test]
async fn app_err_to_response_engine_returns_500() {
    let resp = app_err_to_response(ApplicationError::Engine(crate::error::EngineError::Io(
        "disk".into(),
    )));
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn app_err_to_tuple_validation_returns_400() {
    let (status, body) = app_err_to_tuple(ApplicationError::Validation("bad".into()));
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.contains("bad"));
}

#[test]
fn app_err_to_tuple_concurrent_generation_returns_503() {
    let (status, _) = app_err_to_tuple(ApplicationError::ConcurrentGeneration);
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

#[test]
fn app_err_to_tuple_shutting_down_returns_503() {
    let (status, body) = app_err_to_tuple(ApplicationError::ShuttingDown);
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(body.contains("shutting down"));
}

#[test]
fn app_err_to_tuple_engine_returns_500() {
    let (status, _) = app_err_to_tuple(ApplicationError::Engine(crate::error::EngineError::Io(
        "x".into(),
    )));
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn ctx_or_error_returns_ok_when_app_state_provides_ctx() {
    let state = crate::test_support::TestAppBuilder::default_test().build_app_state();
    let result = ctx_or_error(&state);
    assert!(
        result.is_ok(),
        "ctx_or_error should succeed against a built TestApp state"
    );
}
