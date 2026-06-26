//! Unit tests for worlds_fragment handlers

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;

use super::test_helpers::fetch_body;

fn make_world_form_data(key: &str, name: &str, map_json: &str, scenarios_json: &str) -> String {
    format!(
        "key={}&name={}&description=Test+World&global_rules=rule1&map_json={}&scenarios_json={}",
        key,
        name,
        urlencoding::encode(map_json),
        urlencoding::encode(scenarios_json)
    )
}

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

#[tokio::test]
async fn test_list_worlds_fragment_shows_world_count() {
    let app = TestAppBuilder::default_app();
    let body_str = fetch_body(app.clone(), "/fragment/worlds").await;

    assert!(
        body_str.contains("Test World"),
        "Expected 'Test World' in list: {body_str}"
    );
}

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
    assert!(
        body_str.contains("name=\"key\""),
        "Expected key field in form: {body_str}"
    );
    assert!(
        body_str.contains("Name:") || body_str.contains("name=\"name\""),
        "Expected name field in form: {body_str}"
    );
}

#[tokio::test]
async fn test_new_world_form_handler_has_no_persona_dropdown() {
    let app = TestAppBuilder::default_app();
    let req = Request::builder()
        .uri("/fragment/worlds/new")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        !body_str.contains("player_key") && !body_str.contains("persona"),
        "Worlds form should NOT contain player_key or persona selectors: {body_str}"
    );
}

#[tokio::test]
async fn test_create_world_handler_valid_data() {
    let app = TestAppBuilder::default_app();

    let map_json = json!({
        "overworld": {
            "id": "test_map",
            "name": "Test Map",
            "regions": []
        }
    })
    .to_string();

    let scenarios_json = json!([]).to_string();
    let form_data = make_world_form_data("new_world", "New World", &map_json, &scenarios_json);

    let req = Request::builder()
        .uri("/worlds")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert!(
        response.status().is_success(),
        "Expected success: {:?}",
        response.status()
    );
    assert!(
        response.headers().get("hx-refresh").is_none(),
        "Expected no HX-Refresh header (inline swap)"
    );
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("worlds-panel"),
        "Expected worlds-panel HTML in response: {body_str}"
    );
}

#[tokio::test]
async fn test_create_world_handler_invalid_map_json() {
    let app = TestAppBuilder::default_app();

    let form_data = "key=test&name=Test&description=Test&global_rules=rule1&map_json=invalid_json&scenarios_json=[]";

    let req = Request::builder()
        .uri("/worlds")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Invalid map JSON") || body_str.contains("error"),
        "Expected error about invalid JSON: {body_str}"
    );
}

#[tokio::test]
async fn test_create_world_handler_invalid_scenarios_json() {
    let app = TestAppBuilder::default_app();

    let map_json = json!({
        "overworld": {
            "id": "test",
            "name": "Test",
            "regions": []
        }
    })
    .to_string();

    let form_data = format!(
        "key=test&name=Test&description=Test&global_rules=rule1&map_json={}&scenarios_json=invalid",
        urlencoding::encode(&map_json)
    );

    let req = Request::builder()
        .uri("/worlds")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Invalid scenarios JSON") || body_str.contains("error"),
        "Expected error about invalid scenarios JSON: {body_str}"
    );
}

#[tokio::test]
async fn test_create_world_handler_missing_key() {
    let app = TestAppBuilder::default_app();

    let map_json = json!({
        "overworld": { "id": "test", "name": "Test", "regions": [] }
    })
    .to_string();

    let form_data = format!(
        "name=Test&description=Test&global_rules=rule1&map_json={}&scenarios_json=[]",
        urlencoding::encode(&map_json)
    );

    let req = Request::builder()
        .uri("/worlds")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 422 for missing required key field"
    );
}

#[tokio::test]
async fn test_edit_world_form_handler_not_found() {
    let app = TestAppBuilder::default_app();
    let req = Request::builder()
        .uri("/fragment/worlds/nonexistent/edit")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_world_handler_not_found() {
    let app = TestAppBuilder::default_app();

    let map_json = json!({
        "overworld": { "id": "test", "name": "Test", "regions": [] }
    })
    .to_string();

    let scenarios_json = json!([]).to_string();
    let form_data = make_world_form_data("nonexistent", "Updated", &map_json, &scenarios_json);

    let req = Request::builder()
        .uri("/worlds/nonexistent")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("not found") || body_str.contains("error"),
        "Expected not found error: {body_str}"
    );
}

#[tokio::test]
async fn test_update_world_handler_valid_data() {
    let app = TestAppBuilder::default_app();

    let map_json = json!({
        "overworld": {
            "id": "test_map",
            "name": "Updated Map",
            "regions": []
        }
    })
    .to_string();

    let scenarios_json = json!([]).to_string();
    let form_data = make_world_form_data("test", "Updated Test World", &map_json, &scenarios_json);

    let req = Request::builder()
        .uri("/worlds/test")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert!(
        response.status().is_success(),
        "Expected success: {:?}",
        response.status()
    );
    assert!(
        response.headers().get("hx-refresh").is_none(),
        "Expected no HX-Refresh header (inline swap)"
    );
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("worlds-panel"),
        "Expected worlds-panel HTML in response: {body_str}"
    );
}

#[tokio::test]
async fn test_update_world_handler_invalid_json() {
    let app = TestAppBuilder::default_app();

    let form_data =
        "key=test&name=Test&description=Test&global_rules=rule1&map_json=invalid&scenarios_json=[]";

    let req = Request::builder()
        .uri("/worlds/test")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Invalid map JSON") || body_str.contains("error"),
        "Expected error about invalid JSON: {body_str}"
    );
}

#[tokio::test]
async fn test_delete_world_handler_idempotent() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/worlds/nonexistent/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(
        response.status().is_success(),
        "Expected success for idempotent delete: {:?}",
        response.status()
    );
}

#[tokio::test]
async fn test_delete_world_handler_blocked_by_games() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/worlds/test/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Expected 200 with inline error: {:?}",
        response.status()
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body = String::from_utf8_lossy(&body_bytes).to_string();
    assert!(
        body.contains("games"),
        "Error body should mention games: {body}"
    );
    assert!(
        body.contains("error-message"),
        "Error body should have error-message class: {body}"
    );
}
