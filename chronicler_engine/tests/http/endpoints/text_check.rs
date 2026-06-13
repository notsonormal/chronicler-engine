use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;
use chronicler_engine::model::settings::{AppSettings, TextCheckMode, TextCheckSettings};

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

#[tokio::test]
async fn test_action_check_disabled_forwards_to_action() {
    let app = TestAppBuilder::default_test()
        .settings(text_check_settings(TextCheckMode::Disabled))
        .build();

    let req = Request::builder()
        .uri("/action/check")
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
    assert!(
        body_str.contains("status"),
        "Expected status fragment: {body_str}"
    );
}

#[tokio::test]
async fn test_action_check_empty_command() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/action/check")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command="))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_confirm_returns_full_action_area() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/action/confirm")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=look"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("id=\"action-area\""),
        "Expected action-area container: {body_str}"
    );
    assert!(
        body_str.contains(r#"<form id="command-form""#),
        "Expected command form: {body_str}"
    );
    assert!(
        !body_str.starts_with("<span class=\"status"),
        "Must not return bare status span: {body_str}"
    );
}

#[tokio::test]
async fn test_async_action_saves_input_to_story_log_with_sqlite() {
    use chronicler_engine::storage::Storage;
    use chronicler_engine::storage::db::DbPool;
    let tmp_dir =
        std::env::temp_dir().join(format!("chronicler_component_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&tmp_dir);
    let db_path = tmp_dir.join("test.db");
    let db_pool = DbPool::new(db_path.to_str().unwrap()).unwrap();
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));

    let app = TestAppBuilder::default_test().storage(storage).build();

    let req = Request::builder()
        .uri("/action/check")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=hello+test"))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Poll story log until input entry appears (async action writes to DB in background)
    let mut found_input = false;
    for _attempt in 0..50 {
        let req = Request::builder()
            .uri("/fragment/story-log")
            .method(http::Method::GET)
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        if body_str.contains("log-entry input") && body_str.contains("hello test") {
            found_input = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    assert!(
        found_input,
        "Async action should add input entry to story log within 5s (checked 50 times)"
    );
}

#[tokio::test]
async fn test_action_check_auto_check_disabled() {
    let app = TestAppBuilder::default_test()
        .settings(AppSettings {
            text_check: TextCheckSettings {
                mode: TextCheckMode::Spell,
                enable_auto_check: false,
                ignored_words: vec![],
            },
            ..Default::default()
        })
        .build();

    let req = Request::builder()
        .uri("/action/check")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=look"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let hx_retarget = response.headers().get("HX-Retarget");
    assert!(
        hx_retarget.is_some(),
        "Expected HX-Retarget header when auto-check is disabled"
    );
}

#[tokio::test]
async fn test_action_check_finds_issues() {
    let app = TestAppBuilder::default_test()
        .settings(text_check_settings(TextCheckMode::Spell))
        .build();

    let req = Request::builder()
        .uri("/action/check")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=go+to+the+casle"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("text-check-preview"),
        "Expected preview fragment: {body_str}"
    );
}

#[tokio::test]
async fn test_action_check_no_issues() {
    let app = TestAppBuilder::default_test()
        .settings(text_check_settings(TextCheckMode::SpellGrammar))
        .build();

    let req = Request::builder()
        .uri("/action/check")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=go+to+the+castle"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let hx_retarget = response.headers().get("HX-Retarget");
    assert!(
        hx_retarget.is_some(),
        "Expected HX-Retarget header when no issues and forwarding to action"
    );
}
