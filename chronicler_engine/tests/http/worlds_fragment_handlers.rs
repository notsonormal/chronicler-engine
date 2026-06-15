//! [DOC: docs/system/worlds.md]
//! Unit tests for worlds_fragment handlers

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;

use super::test_helpers::fetch_body;

/// Helper to create valid world form data.
fn make_world_form_data(key: &str, name: &str, map_json: &str, scenarios_json: &str) -> String {
    format!(
        "key={}&name={}&description=Test+World&global_rules=rule1&player_key=test_player&map_json={}&scenarios_json={}&starting_room_id=start",
        key,
        name,
        urlencoding::encode(map_json),
        urlencoding::encode(scenarios_json)
    )
}

/// Test list_worlds_fragment returns HTML with expected structure.
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

/// Test list_worlds_fragment shows world count.
#[tokio::test]
async fn test_list_worlds_fragment_shows_world_count() {
    let app = TestAppBuilder::default_app();
    let body_str = fetch_body(app.clone(), "/fragment/worlds").await;

    // Should contain the test world name from seeded data
    assert!(
        body_str.contains("Test World"),
        "Expected 'Test World' in list: {body_str}"
    );
}

/// Test new_world_form_handler returns form HTML.
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

/// Test new_world_form_handler includes persona dropdown.
#[tokio::test]
async fn test_new_world_form_handler_has_persona_dropdown() {
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
        body_str.contains("player_key") || body_str.contains("persona"),
        "Expected player_key/persona selector in form: {body_str}"
    );
}

/// Test create_world_handler with valid data creates world.
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
    // Handler returns ok_refresh() which sends HX-Refresh header with empty body
    assert!(
        response.headers().get("hx-refresh").is_some(),
        "Expected HX-Refresh header in response"
    );
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.is_empty(),
        "Expected empty body from ok_refresh(): {body_str}"
    );
}

/// Test create_world_handler with invalid map JSON returns error.
#[tokio::test]
async fn test_create_world_handler_invalid_map_json() {
    let app = TestAppBuilder::default_app();

    let form_data = "key=test&name=Test&description=Test&global_rules=rule1&player_key=test&map_json=invalid_json&scenarios_json=[]&starting_room_id=start";

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

/// Test create_world_handler with invalid scenarios JSON returns error.
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
        "key=test&name=Test&description=Test&global_rules=rule1&player_key=test&map_json={}&scenarios_json=invalid&starting_room_id=start",
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

/// Test create_world_handler with missing required fields.
#[tokio::test]
async fn test_create_world_handler_missing_key() {
    let app = TestAppBuilder::default_app();

    let map_json = json!({
        "overworld": { "id": "test", "name": "Test", "regions": [] }
    })
    .to_string();

    // Missing key field
    let form_data = format!(
        "name=Test&description=Test&global_rules=rule1&player_key=test&map_json={}&scenarios_json=[]&starting_room_id=start",
        urlencoding::encode(&map_json)
    );

    let req = Request::builder()
        .uri("/worlds")
        .method(http::Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    // Missing key field causes form deserialization to fail with 422 Unprocessable Entity
    assert_eq!(
        response.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 422 for missing required key field"
    );
}

/// Test edit_world_form_handler for non-existent world returns error.
#[tokio::test]
async fn test_edit_world_form_handler_not_found() {
    let app = TestAppBuilder::default_app();
    let req = Request::builder()
        .uri("/fragment/worlds/nonexistent/edit")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    // Returns 404 Not Found for nonexistent world
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// Test update_world_handler for non-existent world returns error.
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

/// Test update_world_handler with valid data updates world.
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
    // Handler returns ok_refresh() which sends HX-Refresh header with empty body
    assert!(
        response.headers().get("hx-refresh").is_some(),
        "Expected HX-Refresh header in response"
    );
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.is_empty(),
        "Expected empty body from ok_refresh(): {body_str}"
    );
}

/// Test update_world_handler with invalid JSON returns error.
#[tokio::test]
async fn test_update_world_handler_invalid_json() {
    let app = TestAppBuilder::default_app();

    let form_data = "key=test&name=Test&description=Test&global_rules=rule1&player_key=test&map_json=invalid&scenarios_json=[]&starting_room_id=start";

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

/// Test delete_world_handler is idempotent (succeeds even for nonexistent worlds).
#[tokio::test]
async fn test_delete_world_handler_idempotent() {
    let app = TestAppBuilder::default_app();

    // Delete is idempotent - succeeds even if world doesn't exist
    let req = Request::builder()
        .uri("/worlds/nonexistent/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    // Should return success (idempotent operation)
    assert!(
        response.status().is_success(),
        "Expected success for idempotent delete: {:?}",
        response.status()
    );
}

/// Test delete_world_handler returns inline error when games reference the world.
#[tokio::test]
async fn test_delete_world_handler_blocked_by_games() {
    let app = TestAppBuilder::default_app();

    // The default app seeds world "test" and creates a game referencing it
    // Attempt to delete that world — should return inline error HTML
    let req = Request::builder()
        .uri("/worlds/test/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();

    // The delete returns 200 OK with inline error HTML (is_user_displayable path)
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
