//! HTTP integration tests for the text-check endpoints: action-check dispatch (disabled vs. enabled), empty-command handling, and confirm-flow returning the full action area with check results.

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::post,
    Router,
};
use tower::util::ServiceExt;

use chronicler_engine::adapters::driving::http::chat_window::handlers::check_text_handler;
use chronicler_engine::application::ports::text_checker::{CheckResult, TextChecker};
use chronicler_engine::application::text_check_service::TextCheckService;
use chronicler_engine::domain::model::settings::{AppSettings, TextCheckMode, TextCheckSettings};
use chronicler_engine::error::EngineError;
use chronicler_engine::TestAppBuilder;

struct FailingTextChecker;

impl TextChecker for FailingTextChecker {
    fn check(
        &self,
        _text: &str,
        _mode: TextCheckMode,
        _ignored_words: &[String],
    ) -> Result<Option<CheckResult>, EngineError> {
        Err(EngineError::Io("simulated text check failure".to_string()))
    }
}

async fn story_log_contains(app: &axum::Router, needle: &str) -> bool {
    for _ in 0..100 {
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
        if body_str.contains(needle) {
            return true;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    false
}

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
    use chronicler_engine::adapters::driven::storage::Storage;
    use chronicler_engine::adapters::driven::storage::db::DbPool;
    let tmp_dir = tempfile::TempDir::new().unwrap();
    let db_path = tmp_dir.path().join("test.db");
    let db_pool = DbPool::new(db_path.to_str().unwrap()).unwrap();
    let storage = Arc::new(Storage::new_sqlite(db_pool, 1));

    use chronicler_engine::test_support::{TestWorld, TestMap, TestPersona};
    let world = TestWorld::minimal();
    let map = TestMap::single_room("start");
    storage.seed_world(&world, &map).unwrap();
    let player = TestPersona::standard();
    storage.seed_persona(&player.key, &player).unwrap();
    let game_id = storage
        .create_game(
            &world.name,
            &world.key,
            &player.key,
            &player.sheet.name,
            "Test Game",
        )
        .unwrap();
    storage.set_game_id(game_id);

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

    let found_input = story_log_contains(&app, "hello test").await;

    assert!(
        found_input,
        "Async action should add input entry to story log within 10s"
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

#[tokio::test]
async fn test_check_text_handler_failure_returns_internal_server_error() {
    let mut state = TestAppBuilder::default_test()
        .settings(text_check_settings(TextCheckMode::Spell))
        .build_app_state();
    state.text_check_service = Arc::new(TextCheckService::new(Arc::new(FailingTextChecker)));

    let app = Router::new()
        .route("/check-text", post(check_text_handler))
        .with_state(state);
    let req = Request::builder()
        .uri("/check-text")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=anything"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
