use axum::{body::Body, http::Request};
use tower::util::ServiceExt;

use chronicler_engine::create_app_for_testing;

use crate::create_test_state;

#[tokio::test]
async fn test_debug_state_endpoint_returns_json() {
    let state = create_test_state();
    let app = create_app_for_testing(state);

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

    assert!(
        json.get("current_room_id").is_some(),
        "Debug state should include current_room_id"
    );
    assert!(
        json.get("npcs_in_area").is_some(),
        "Debug state should include npcs_in_area"
    );
    assert!(
        json.get("generation_status").is_some(),
        "Debug state should include generation_status"
    );
    assert!(
        json.get("generation_phase").is_some(),
        "Debug state should include generation_phase"
    );
    assert!(
        json.get("character_state").is_some(),
        "Debug state should include character_state"
    );
    assert!(
        json.get("narration_history_tail").is_some(),
        "Debug state should include narration_history_tail"
    );
}
