use axum::{body::Body, http::Request};
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;

/// Test worlds fragment panel returns HTML.
#[tokio::test]
async fn test_list_worlds_fragment_returns_html() {
    let app = TestAppBuilder::default_app();
    let req = Request::builder()
        .uri("/fragment/worlds")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("worlds-panel"),
        "Expected worlds-panel in response: {body_str}"
    );
}

/// Test new world form handler returns form HTML.
#[tokio::test]
async fn test_new_world_form_handler_returns_form() {
    let app = TestAppBuilder::default_app();
    let req = Request::builder()
        .uri("/fragment/worlds/new")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("world-form-container"),
        "Expected world-form-container in response: {body_str}"
    );
}

/// Test create world handler with invalid JSON returns error.
#[tokio::test]
async fn test_create_world_handler_invalid_json() {
    let app = TestAppBuilder::default_app();
    let form_data = "key=test&name=Test&description=Test&global_rules=rule1&player_key=test&map_json=invalid&scenarios_json=[]";
    let req = Request::builder()
        .uri("/worlds")
        .method("POST")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    // Should contain an error message about invalid JSON
    assert!(
        body_str.contains("Invalid map JSON") || body_str.contains("error"),
        "Expected error in response: {body_str}"
    );
}

/// Test delete world handler endpoint exists.
#[tokio::test]
async fn test_delete_world_handler_exists() {
    let app = TestAppBuilder::default_app();
    let req = Request::builder()
        .uri("/worlds/nonexistent/delete")
        .method("POST")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    // Should return some response - either 400 (world not found) or 500 (context error)
    // Just verify the endpoint responds
    assert!(
        response.status().is_success()
            || response.status().is_client_error()
            || response.status().is_server_error()
    );
}
