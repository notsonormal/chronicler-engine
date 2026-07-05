use axum::{body::Body, http::Request};
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;

use crate::SettingsTestGuard;

#[tokio::test]
async fn test_add_connection_openrouter() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/connections/add")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "conn_name=My+OpenRouter&conn_provider=openrouter&conn_model=openai/gpt-4o&conn_api_key=sk-test&conn_base_url=",
        ))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("My OpenRouter"),
        "Expected new connection name in response: {body_str}"
    );
}

#[tokio::test]
async fn test_add_connection_deepseek() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/connections/add")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "conn_name=My+DeepSeek&conn_provider=deepseek&conn_model=deepseek-chat&conn_api_key=&conn_base_url=",
        ))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("My DeepSeek"),
        "Expected new connection name in response: {body_str}"
    );
}

#[tokio::test]
async fn test_set_narrator() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/connections/openrouter-euryale/set-narrator")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Narrator"),
        "Expected Narrator badge on euryale: {body_str}"
    );
}

#[tokio::test]
async fn test_set_quantifier() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/connections/ollama-gemma-4-26B/set-quantifier")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Quantifier"),
        "Expected Quantifier badge on gemma: {body_str}"
    );
}

#[tokio::test]
async fn test_set_narrator_not_found() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/connections/nonexistent/set-narrator")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("LlmProviderConfig not found"),
        "Expected error for nonexistent connection: {body_str}"
    );
}

#[tokio::test]
async fn test_delete_connection() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/connections/ollama-gemma-4-26B/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.is_empty(),
        "Expected empty response (HTMX swap delete): '{body_str}'"
    );
}

#[tokio::test]
async fn test_delete_connection_not_found() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/connections/nonexistent/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("LlmProviderConfig not found"),
        "Expected error for nonexistent connection: {body_str}"
    );
}

#[tokio::test]
async fn test_edit_connection() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/connections/openrouter-gpt-4o-mini/edit")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "conn_name=Updated+Name&conn_provider=openrouter&conn_model=gpt-4o&conn_api_key=new-key&conn_base_url=",
        ))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Updated Name"),
        "Expected updated connection name: {body_str}"
    );
}

#[tokio::test]
async fn test_edit_connection_not_found() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/connections/nonexistent/edit")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "conn_name=Updated+Name&conn_provider=openrouter&conn_model=gpt-4o&conn_api_key=&conn_base_url=",
        ))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("LlmProviderConfig not found"),
        "Expected error for nonexistent connection: {body_str}"
    );
}

#[tokio::test]
async fn test_connection_card_fragment() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/fragment/connections/openrouter-gpt-4o-mini")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("openrouter-gpt-4o-mini"),
        "Expected connection card: {body_str}"
    );
}

#[tokio::test]
async fn test_connection_card_fragment_not_found() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/fragment/connections/nonexistent")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("LlmProviderConfig not found"),
        "Expected error for nonexistent connection: {body_str}"
    );
}

#[tokio::test]
async fn test_edit_connection_form() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/fragment/connections/openrouter-gpt-4o-mini/edit")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Edit openrouter-gpt-4o-mini"),
        "Expected edit form: {body_str}"
    );
    assert!(
        body_str.contains("conn_name"),
        "Expected conn_name field: {body_str}"
    );
}

#[tokio::test]
async fn test_edit_connection_form_not_found() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/fragment/connections/nonexistent/edit")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("LlmProviderConfig not found"),
        "Expected error for nonexistent connection: {body_str}"
    );
}

#[tokio::test]
async fn test_add_connection_with_single_user_message() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/connections/add")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "conn_name=My+Mock&conn_provider=mock&conn_model=mock-model&conn_api_key=&conn_base_url=&single_user_message=true",
        ))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("My Mock"),
        "Expected new connection name in response: {body_str}"
    );
}

#[tokio::test]
async fn test_edit_connection_preserves_single_user_message() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/connections/openrouter-gpt-4o-mini/edit")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(
            "conn_name=Updated+Name&conn_provider=openrouter&conn_model=gpt-4o&conn_api_key=new-key&conn_base_url=&single_user_message=true",
        ))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Updated Name"),
        "Expected updated connection name: {body_str}"
    );
}

#[tokio::test]
async fn test_edit_connection_form_has_single_user_message_checkbox() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/fragment/connections/openrouter-gpt-4o-mini/edit")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("single_user_message"),
        "Expected single_user_message checkbox in edit form: {body_str}"
    );
}
