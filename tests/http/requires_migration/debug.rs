//! HTTP integration tests for the debug endpoints: `/debug/state` returns the expected JSON shape and `/debug/is_busy` reflects the actual busy state.

use axum::{body::Body, http::Request};
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;

#[tokio::test]
async fn test_debug_state_endpoint_returns_json() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/debug/state")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response should be valid JSON");

    assert!(json.get("current_room_id").is_some());
    assert!(json.get("characters_present").is_some());
    assert!(json.get("generation_status").is_some());
    assert!(json.get("step").is_some());
    assert!(json.get("encounters").is_some());
    assert!(json.get("narration_history_tail").is_some());

    assert!(
        json["current_room_id"].is_string(),
        "current_room_id should be a string"
    );
    assert!(
        json["characters_present"].is_array(),
        "characters_present should be an array"
    );
    // Unit enum variants serialize as strings, tagged variants as objects
    let status = &json["generation_status"];
    assert!(
        status.is_string() || status.is_object(),
        "generation_status should be enum (string or object)"
    );
    if let Some(s) = status.as_str() {
        assert!(
            s == "Idle" || s == "Busy",
            "generation_status should be Idle or Busy, got: {s}"
        );
    } else if let Some(obj) = status.as_object() {
        assert!(
            obj.contains_key("Failed"),
            "Failed variant should have 'Failed' key"
        );
    }
}

#[tokio::test]
async fn test_debug_state_endpoint_includes_all_documented_fields() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/debug/state")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response should be valid JSON");

    let required_fields = [
        "current_room_id",
        "characters_present",
        "generation_status",
        "step",
        "encounters",
        "narration_history_tail",
        "narration_history_length",
        "dynamic_rooms",
        "dynamic_room_count",
        "last_error",
        "backend_name",
        "model_name",
        "current_options",
    ];

    for field in &required_fields {
        assert!(
            json.get(field).is_some(),
            "DebugStateResponse should include field '{field}'"
        );
    }

    assert!(
        json["narration_history_length"].is_number(),
        "narration_history_length should be a number"
    );
    assert!(
        json["dynamic_room_count"].is_number(),
        "dynamic_room_count should be a number"
    );
    assert!(
        json["current_options"].is_array(),
        "current_options should be an array (the offered option set)"
    );
}

#[tokio::test]
async fn test_debug_is_busy_returns_false_by_default() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/debug/is_busy")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 64)
        .await
        .unwrap();
    assert_eq!(
        &body[..],
        b"false",
        "is_busy should return 'false' by default"
    );
}

#[tokio::test]
async fn test_debug_backend_returns_json() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/debug/backend")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("Response should be valid JSON");

    assert!(
        json["backend_name"].is_string(),
        "backend_name should be a string"
    );
    assert!(
        json["model_name"].is_string(),
        "model_name should be a string"
    );
}
