//! HTTP-level tests for the settings fragment endpoint; verifies the settings panel renders and that settings state changes persist correctly.

use axum::{body::Body, http::Request};
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;

use crate::SettingsTestGuard;

#[tokio::test]
async fn test_settings_panel_returns_html() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/fragment/settings")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Connections"),
        "Expected 'Connections' in response: {body_str}"
    );
}

#[tokio::test]
async fn test_settings_panel_has_provider_select() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/fragment/settings")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("conn_provider"),
        "Expected conn_provider select element: {body_str}"
    );
    assert!(
        body_str.contains("OpenRouter"),
        "Expected OpenRouter option: {body_str}"
    );
    assert!(
        body_str.contains("DeepSeek"),
        "Expected DeepSeek option: {body_str}"
    );
}

#[tokio::test]
async fn test_settings_panel_has_model_input() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/fragment/settings")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("conn_model"),
        "Expected conn_model input: {body_str}"
    );
}

#[tokio::test]
async fn test_save_settings_switch_narrator() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/settings")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "narration_connection_id=openrouter-euryale&quantifier_connection_id=openrouter-gpt-4o-mini",
        ))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("saved"),
        "Expected success response: {body_str}"
    );
}

#[tokio::test]
async fn test_save_settings_switch_quantifier() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/settings")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "narration_connection_id=openrouter-gpt-4o-mini&quantifier_connection_id=ollama-gemma-4-26B",
        ))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("saved"),
        "Expected success response: {body_str}"
    );
}

#[tokio::test]
async fn test_save_settings_switch_both() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/settings")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "narration_connection_id=openrouter-euryale&quantifier_connection_id=ollama-gemma-4-26B",
        ))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("saved"),
        "Expected success response: {body_str}"
    );
}

#[tokio::test]
async fn test_settings_panel_has_single_user_message_checkbox() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/fragment/settings")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("single_user_message"),
        "Expected single_user_message checkbox in settings panel: {body_str}"
    );
    assert!(
        body_str.contains("Single User Message"),
        "Expected 'Single User Message' label in settings panel: {body_str}"
    );
}
