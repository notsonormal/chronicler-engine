//! Tests for `ApplicationError::IntoResponse` mapping.

use axum::body::to_bytes;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::application::application_service::ApplicationError;
use crate::error::EngineError;

async fn body_string(resp: axum::response::Response) -> String {
    let bytes = to_bytes(resp.into_body(), 4096).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap_or_default()
}

#[tokio::test]
async fn validation_into_response_returns_400_with_message() {
    let resp = ApplicationError::validation("bad input").into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = body_string(resp).await;
    assert!(body.contains("bad input"), "body: {body}");
}

#[tokio::test]
async fn concurrent_generation_into_response_returns_503_with_wait_body() {
    let resp = ApplicationError::ConcurrentGeneration.into_response();
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = body_string(resp).await;
    assert!(body.contains("Generation in progress"), "body: {body}");
}

#[tokio::test]
async fn shutting_down_into_response_returns_503_with_shutdown_body() {
    let resp = ApplicationError::ShuttingDown.into_response();
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = body_string(resp).await;
    assert!(body.contains("shutting down"), "body: {body}");
}

#[tokio::test]
async fn engine_displayable_into_response_returns_500_with_engine_message() {
    let resp =
        ApplicationError::Engine(EngineError::WorldHasGames { game_count: 1 }).into_response();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = body_string(resp).await;
    assert!(!body.contains("Internal Server Error"), "body: {body}");
    assert!(
        body.contains("Cannot delete world"),
        "body should contain engine message; got: {body}"
    );
    assert!(
        body.contains("1"),
        "body should include the game_count value; got: {body}"
    );
}

#[tokio::test]
async fn engine_non_displayable_into_response_returns_500_with_generic_body() {
    let resp = ApplicationError::Engine(EngineError::Render("secret internal detail".into()))
        .into_response();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = body_string(resp).await;
    assert!(body.contains("Internal Server Error"), "body: {body}");
    assert!(
        !body.contains("secret internal detail"),
        "non-displayable engine body must not leak internal details; body: {body}"
    );
}

#[tokio::test]
async fn html_escape_in_body() {
    let resp = ApplicationError::validation("<script>alert(\"x\")</script>").into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = body_string(resp).await;
    assert!(body.contains("&lt;script&gt;"), "body: {body}");
    assert!(body.contains("&quot;"), "body: {body}");
    assert!(
        !body.contains("<script>"),
        "body must escape <script>: {body}"
    );
}
