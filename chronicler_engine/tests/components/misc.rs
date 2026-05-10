use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::util::ServiceExt;

use chronicler_engine::create_app_for_testing_with_settings;
use chronicler_engine::model::settings::{AppSettings, TextCheckMode, TextCheckSettings};
use chronicler_engine::model::state::{GameState, LogType};

use crate::create_test_state;

fn text_check_settings(mode: TextCheckMode) -> AppSettings {
    AppSettings {
        text_check: TextCheckSettings {
            mode,
            enable_auto_check: true,
            ignored_words: vec![],
        },
        ..Default::default()
    }
}

fn state_with_input() -> GameState {
    let mut state = create_test_state();
    state.add_log("look around".to_string(), Some("Test Player".to_string()), LogType::Input);
    state
}

#[tokio::test]
async fn test_check_text_empty_command() {
    let app = chronicler_engine::create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/check-text")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command="))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("Enter text to check"), "Expected error message: {body_str}");
}

#[tokio::test]
async fn test_check_text_disabled_mode() {
    let app = create_app_for_testing_with_settings(
        create_test_state(),
        text_check_settings(TextCheckMode::Disabled),
    );

    let req = Request::builder()
        .uri("/check-text")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=hello"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Text check is disabled"),
        "Expected disabled message: {body_str}"
    );
}

#[tokio::test]
async fn test_check_text_finds_issues() {
    let app = create_app_for_testing_with_settings(
        create_test_state(),
        text_check_settings(TextCheckMode::Spell),
    );

    let req = Request::builder()
        .uri("/check-text")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=go to the casle"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 4096).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("text-check-preview"),
        "Expected preview HTML: {body_str}"
    );
}

#[tokio::test]
async fn test_check_text_no_issues() {
    let app = create_app_for_testing_with_settings(
        create_test_state(),
        text_check_settings(TextCheckMode::Spell),
    );

    let req = Request::builder()
        .uri("/check-text")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=go to the castle"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("No issues found"),
        "Expected no issues message: {body_str}"
    );
}

#[tokio::test]
async fn test_retry_no_input() {
    let app = chronicler_engine::create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/retry")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("No input to retry"), "Expected error: {body_str}");
}

#[tokio::test]
async fn test_retry_success() {
    let app = chronicler_engine::create_app_for_testing(state_with_input());

    let req = Request::builder()
        .uri("/retry")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Retrying..."),
        "Expected retry message: {body_str}"
    );
}

#[tokio::test]
async fn test_reset_handler() {
    let app = chronicler_engine::create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/reset")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 4096).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("header"), "Expected header in reset response: {body_str}");
    assert!(body_str.contains("story-log"), "Expected story log in reset response: {body_str}");
    assert!(
        body_str.contains("visual-sidebar"),
        "Expected sidebar in reset response: {body_str}"
    );
    assert!(
        body_str.contains("action-area"),
        "Expected action area in reset response: {body_str}"
    );
}
