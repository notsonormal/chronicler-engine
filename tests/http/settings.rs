//! HTTP E2E tests for the settings endpoints: panel rendering and POST /settings.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{self, Request, StatusCode};
use tower::util::ServiceExt;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driven::storage::TestOverride;
use chronicler_engine::TestAppBuilder;

use crate::SettingsTestGuard;

async fn body_string(response: axum::response::Response<Body>) -> String {
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    String::from_utf8_lossy(&body).to_string()
}

fn post_form_request(uri: &str, body: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(body.to_string()))
        .unwrap()
}

// [docs/specs/settings.md] SCENARIO: 20.1
#[tokio::test]
async fn test_settings_panel_renders_full_surface() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/fragment/settings")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert!(body.contains(r#"<div class="settings-panel">"#));
    assert!(body.contains("<h2>Connections</h2>"));
    assert!(body.contains("connection-card"));
    assert!(body.contains("<h3>Add LlmProviderConfig</h3>"));
    assert!(body.contains(r#"id="conn_name""#));
    assert!(body.contains(r#"name="conn_provider""#));
    assert!(body.contains("OpenRouter"));
    assert!(body.contains("DeepSeek"));
    assert!(body.contains("Ollama"));
    assert!(body.contains(r#"id="conn_model""#));
    assert!(body.contains(r#"id="conn_api_key""#));
    assert!(body.contains(r#"id="conn_base_url""#));
    assert!(body.contains(r#"name="single_user_message""#));
    assert!(body.contains("Single User Message"));
    assert!(body.contains("<h2>Text Check</h2>"));
    assert!(body.contains(r#"id="check_mode""#));
    assert!(body.contains(r#"name="enable_auto_check""#));
}

// [docs/specs/settings.md] SCENARIO: 20.2
#[tokio::test]
async fn test_post_settings_switches_narrator() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = post_form_request(
        "/settings",
        "narration_connection_id=openrouter-euryale&quantifier_connection_id=openrouter-gpt-4o-mini",
    );
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert_eq!(body, "Settings saved!");
}

// [docs/specs/settings.md] SCENARIO: 20.3
#[tokio::test]
async fn test_post_settings_switches_quantifier() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = post_form_request(
        "/settings",
        "narration_connection_id=openrouter-gpt-4o-mini&quantifier_connection_id=ollama-gemma-4-26B",
    );
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert_eq!(body, "Settings saved!");
}

// [docs/specs/settings.md] SCENARIO: 20.4
#[tokio::test]
async fn test_post_settings_switches_both_connections() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = post_form_request(
        "/settings",
        "narration_connection_id=openrouter-euryale&quantifier_connection_id=ollama-gemma-4-26B",
    );
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert_eq!(body, "Settings saved!");
}

// [docs/specs/settings.md] SCENARIO: 20.5
#[tokio::test]
async fn test_post_settings_accepts_unknown_connection_id() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = post_form_request(
        "/settings",
        "narration_connection_id=not-a-connection&quantifier_connection_id=openrouter-gpt-4o-mini",
    );
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert_eq!(body, "Settings saved!");
}

// [docs/specs/settings.md] SCENARIO: 20.6
#[tokio::test]
async fn test_post_settings_missing_field_returns_422() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = post_form_request("/settings", "narration_connection_id=openrouter-euryale");
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// [docs/specs/settings.md] SCENARIO: 20.7
#[tokio::test]
async fn test_post_settings_reports_save_failure() {
    let _guard = SettingsTestGuard::new();
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        "save_settings",
        TestOverride::internal("settings save failure"),
    ));
    let app = TestAppBuilder::default_test().storage(storage).build();

    let req = post_form_request(
        "/settings",
        "narration_connection_id=openrouter-euryale&quantifier_connection_id=openrouter-gpt-4o-mini",
    );
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert!(body.contains(r#"<span class='error'>Save failed:"#));
}
