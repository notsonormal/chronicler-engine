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
    assert!(json.get("npcs_in_area").is_some());
    assert!(json.get("generation_status").is_some());
    assert!(json.get("generation_phase").is_some());
    assert!(json.get("npc_encounter_log").is_some());
    assert!(json.get("narration_history_tail").is_some());

    assert!(
        json["current_room_id"].is_string(),
        "current_room_id should be a string"
    );
    assert!(
        json["npcs_in_area"].is_array(),
        "npcs_in_area should be an array"
    );
    assert!(
        json["generation_status"].is_string(),
        "generation_status should be a string"
    );
    let status = json["generation_status"].as_str().unwrap();
    assert!(
        status == "Idle" || status == "Generating" || status.starts_with("Error("),
        "generation_status should be Idle, Generating, or Error(…), got: {status}"
    );
    assert!(
        json["generation_phase"].is_string(),
        "generation_phase should be a string"
    );
    let phase = json["generation_phase"].as_str().unwrap();
    assert!(
        matches!(phase, "Narrating" | "Quantifying" | "GeneratingEvent"),
        "generation_phase should be a known variant, got: {phase}"
    );
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
        "npcs_in_area",
        "generation_status",
        "generation_phase",
        "npc_encounter_log",
        "narration_history_tail",
        "narration_history_length",
        "dynamic_rooms",
        "dynamic_room_count",
        "last_error",
        "quantifier_confidence",
        "backend_name",
        "model_name",
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
}

#[tokio::test]
async fn test_debug_is_generating_returns_false_by_default() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/debug/is_generating")
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
        "is_generating should return 'false' by default"
    );
}

#[tokio::test]
async fn test_debug_is_generating_reflects_state() {
    let app = TestAppBuilder::default_test().is_generating(true).build();

    let req = Request::builder()
        .uri("/debug/is_generating")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 64)
        .await
        .unwrap();
    assert_eq!(
        &body[..],
        b"true",
        "is_generating should return 'true' when set via builder"
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
