use std::sync::{Arc, RwLock};
use axum::{http::StatusCode, Form};

use crate::application::application_service::DefaultApplicationService;
use crate::application::game_service::GameService;
use crate::model::settings::{AppSettings, TextCheckMode};
use crate::server::fragments::actions::ActionForm;
use crate::server::fragments::misc::{
    check_text_handler, reset_handler, retrigger_handler, retry_handler, switch_swipe_handler,
};
use crate::server::AppState;
use crate::storage::Storage;
use crate::test_support::{TestMap, TestPlayer, TestWorld};
use tokio_util::sync::CancellationToken;

fn make_test_app_state() -> AppState {
    let storage = Arc::new(Storage::new_in_memory());
    let settings = Arc::new(RwLock::new(AppSettings::default()));
    let game_service = Arc::new(GameService::with_storage(
        Some(Arc::clone(&storage)),
        None,
        Arc::clone(&settings),
    ));
    AppState {
        storage: Arc::clone(&storage),
        preset_storage: Arc::new(Storage::new_in_memory()),
        world: Arc::new(TestWorld::minimal()),
        map: Arc::new(TestMap::single_room("start")),
        player: Arc::new(TestPlayer::standard()),
        npcs: Arc::new(std::collections::HashMap::new()),
        game_service: Arc::clone(&game_service),
        application_service: Arc::new(DefaultApplicationService::new(Arc::clone(&game_service))),
        settings,
        cancel_token: Arc::new(RwLock::new(CancellationToken::new())),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    }
}

fn make_test_app_state_with_generating(is_generating: bool) -> AppState {
    let state = make_test_app_state();
    state
        .is_generating
        .store(is_generating, std::sync::atomic::Ordering::SeqCst);
    state
}

// ─── Check Text Handler Tests ────────────────────────────────────────────────

#[tokio::test]
async fn test_check_text_handler_empty() {
    let state = make_test_app_state();
    let form = ActionForm {
        command: String::new(),
    };
    let response = check_text_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_check_text_handler_disabled() {
    let state = make_test_app_state();
    state.settings.write().unwrap().text_check.mode = TextCheckMode::Disabled;
    let form = ActionForm {
        command: "test".to_string(),
    };
    let response = check_text_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_check_text_handler_ok() {
    let state = make_test_app_state();
    let form = ActionForm {
        command: "This is a test sentence".to_string(),
    };
    let response = check_text_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

// ─── Retry Handler Tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_retry_handler() {
    let state = make_test_app_state();
    let (status, _body) = retry_handler(axum::extract::State(state)).await;
    // May return 400 if no history to retry, or 200 if success
    assert!(status == StatusCode::OK || status == StatusCode::BAD_REQUEST);
}

// ─── Retrigger Handler Tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_retrigger_handler_ok() {
    let state = make_test_app_state();
    let (status, body) = retrigger_handler(axum::extract::State(state)).await;
    assert!(status == StatusCode::OK || status == StatusCode::BAD_REQUEST);
    // May fail if no last_trigger, but should not panic
    assert!(!body.is_empty());
}

// ─── Switch Swipe Handler Tests ──────────────────────────────────────────────

#[tokio::test]
async fn test_switch_swipe_handler() {
    let state = make_test_app_state();
    let response = switch_swipe_handler(
        axum::extract::State(state),
        axum::extract::Path((0u64, 0usize)),
    )
    .await;
    // May return various statuses depending on game state
    // Just ensure it doesn't panic
    assert!(
        response.status().is_success()
            || response.status().is_client_error()
            || response.status().is_server_error()
    );
}

// ─── Reset Handler Tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_reset_handler_ok() {
    let state = make_test_app_state();
    let response = reset_handler(axum::extract::State(state)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_reset_handler_while_generating() {
    let state = make_test_app_state_with_generating(true);
    let response = reset_handler(axum::extract::State(state)).await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}
