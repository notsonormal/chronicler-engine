use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use tower::util::ServiceExt;

use chronicler_engine::create_app_for_testing;

use crate::create_test_state;

async fn fetch_body(app: Router, uri: &str) -> String {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert!(response.status().is_success(), "Expected success for {uri}");
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    String::from_utf8_lossy(&body).to_string()
}

#[tokio::test]
async fn test_header_fragment_returns_html() {
    let body = fetch_body(
        create_app_for_testing(create_test_state()),
        "/fragment/header",
    )
    .await;
    assert!(body.contains("class=\"header\""));
    assert!(body.contains("Chronicler Engine"));
}

#[tokio::test]
async fn test_story_log_fragment_returns_html() {
    let body = fetch_body(
        create_app_for_testing(create_test_state()),
        "/fragment/story-log",
    )
    .await;
    assert!(body.contains("id=\"story-log\""));
}

#[tokio::test]
async fn test_visual_sidebar_fragment_returns_html() {
    let body = fetch_body(
        create_app_for_testing(create_test_state()),
        "/fragment/visual-sidebar",
    )
    .await;
    assert!(body.contains("id=\"visual-sidebar\""));
}

#[tokio::test]
async fn test_visual_sidebar_renders_room_image() {
    let body = fetch_body(
        create_app_for_testing(create_test_state()),
        "/fragment/visual-sidebar",
    )
    .await;
    // Should contain the image, not "No Location Image"
    assert!(
        body.contains("data/images/test_room.png"),
        "Expected room image in sidebar: {body}"
    );
    assert!(
        !body.contains("No Location Image"),
        "Should not show placeholder when image exists: {body}"
    );
}

#[tokio::test]
async fn test_action_area_fragment_returns_html() {
    let body = fetch_body(
        create_app_for_testing(create_test_state()),
        "/fragment/action-area",
    )
    .await;
    assert!(
        body.contains("id=\"action-area\""),
        "Expected action-area id: {body}"
    );
}

#[tokio::test]
async fn test_action_handler_accepts_command() {
    let app = create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=look"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("status"));
}

#[tokio::test]
async fn test_action_handler_empty_command() {
    let app = create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command="))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Enter a command"),
        "Expected empty command error: {body_str}"
    );
}

#[tokio::test]
async fn test_hints_handler() {
    let body = fetch_body(create_app_for_testing(create_test_state()), "/hints").await;
    assert!(body.contains("Look"));
}

#[tokio::test]
async fn test_status_ready_handler() {
    let body = fetch_body(create_app_for_testing(create_test_state()), "/status/ready").await;
    assert!(body.contains("Ready"));
}

#[tokio::test]
async fn test_character_headshots_fragment() {
    let body = fetch_body(
        create_app_for_testing(create_test_state()),
        "/fragment/character-headshots",
    )
    .await;
    // The test state has npc_1 with a profile_image, so headshots should render
    assert!(
        body.contains("headshot"),
        "Expected headshot in fragment: {body}"
    );
}

#[tokio::test]
async fn test_generating_status_handler_idle() {
    let body = fetch_body(
        create_app_for_testing(create_test_state()),
        "/status/generating",
    )
    .await;
    // Should return "idle" when not generating
    assert!(body.contains("idle"));
}

#[tokio::test]
async fn test_generating_status_handler_narrating() {
    let mut state = create_test_state();
    state.narrative.generation.status =
        chronicler_engine::model::state::GenerationStatus::Generating;
    state.narrative.generation.phase = chronicler_engine::model::state::GenerationPhase::Narrating;

    let body = fetch_body(create_app_for_testing(state), "/status/generating").await;
    assert!(body.contains("narrating"));
}

#[tokio::test]
async fn test_generating_status_handler_quantifying() {
    let mut state = create_test_state();
    state.narrative.generation.status =
        chronicler_engine::model::state::GenerationStatus::Generating;
    state.narrative.generation.phase =
        chronicler_engine::model::state::GenerationPhase::Quantifying;

    let body = fetch_body(create_app_for_testing(state), "/status/generating").await;
    assert!(body.contains("quantifying"));
}

#[tokio::test]
async fn test_reset_generating_handler() {
    let app = create_app_for_testing(create_test_state());

    // reset-generating is POST, not GET
    let req = Request::builder()
        .uri("/status/reset-generating")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    // Should return "reset" on success
    assert!(body_str.contains("reset"));
}

#[tokio::test]
async fn test_edit_history_handler_success() {
    let mut state = create_test_state();
    let entry_id = {
        state.add_log(
            "Original text".to_string(),
            Some("Test".to_string()),
            chronicler_engine::model::state::LogType::Narration,
        );
        state.narrative.history.last().unwrap().id
    };

    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri(format!("/history/{entry_id}"))
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("text=Edited+text"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Edited"),
        "Expected success message: {body_str}"
    );
}

#[tokio::test]
async fn test_edit_history_handler_not_found() {
    let app = create_app_for_testing(create_test_state());

    // Try to edit a non-existent log entry (ID 9999) - correct path is /history/:id
    let req = Request::builder()
        .uri("/history/9999")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("text=Edited text"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    // Should return NOT_FOUND for non-existent entry
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_history_handler_success() {
    let mut state = create_test_state();

    // Add a log entry first
    state.add_log(
        "Test message".to_string(),
        Some("Test".to_string()),
        chronicler_engine::model::state::LogType::Narration,
    );

    let app = create_app_for_testing(state.clone());

    let req = Request::builder()
        .uri("/history/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify the entry was deleted by fetching the story log fragment
    let body = fetch_body(app, "/fragment/story-log").await;
    assert!(
        !body.contains("Test message"),
        "Log entry should be deleted from rendered story log"
    );
}

#[tokio::test]
async fn test_delete_history_handler_empty() {
    let app = create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/history/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_action_confirm_empty_command() {
    let app = create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/action/confirm")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command="))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Enter a command"),
        "Expected empty command error: {body_str}"
    );
}

#[tokio::test]
async fn test_action_concurrent_rejection() {
    let app = create_app_for_testing(create_test_state());

    // First async action sets is_generating = true
    let req1 = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=go north"))
        .unwrap();
    let response1 = app.clone().oneshot(req1).await.unwrap();
    assert!(response1.status().is_success());

    // Second async action while first is in flight
    let req2 = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=go south"))
        .unwrap();
    let response2 = app.oneshot(req2).await.unwrap();
    assert!(response2.status().is_success());
    let body = axum::body::to_bytes(response2.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Still thinking..."),
        "Expected concurrent rejection: {body_str}"
    );
}

#[tokio::test]
async fn test_action_sync_inventory() {
    let app = create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=inventory"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let hx_trigger = response.headers().get("HX-Trigger");
    assert!(
        hx_trigger.is_some(),
        "Expected HX-Trigger header for sync action"
    );
}

#[tokio::test]
async fn test_action_sync_quit() {
    let app = create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=quit"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let hx_trigger = response.headers().get("HX-Trigger");
    assert!(
        hx_trigger.is_some(),
        "Expected HX-Trigger header for sync action"
    );
}

#[tokio::test]
async fn test_retry_handler_no_input() {
    let app = create_app_for_testing(create_test_state());

    // retry is POST, not GET
    let req = Request::builder()
        .uri("/retry")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    // With no input history, should return BAD_REQUEST
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
