use std::sync::{Arc, RwLock};
use axum::{Form, http::StatusCode};
use tokio_util::sync::CancellationToken;

use crate::application::application_service::DefaultApplicationService;
use crate::application::game_service::GameService;
use crate::model::settings::{AppSettings, TextCheckMode};
use crate::server::fragments::actions::{action_check_handler, action_confirm_handler, action_handler};
use crate::server::fragments::ActionForm;
use crate::server::AppState;
use crate::storage::Storage;
use crate::test_support::{TestMap, TestPlayer, TestWorld};

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

#[test]
fn test_action_form_deserialization() {
    let form: ActionForm = serde_json::from_str(r#"{"command": "look"}"#).unwrap();
    assert_eq!(form.command, "look");
}

#[test]
fn test_action_form_empty_command() {
    let form: ActionForm = serde_json::from_str(r#"{"command": ""}"#).unwrap();
    assert!(form.command.is_empty());
}

#[test]
fn test_action_form_with_whitespace_command() {
    let form: ActionForm = serde_json::from_str(r#"{"command": "  look  "}"#).unwrap();
    assert_eq!(form.command, "  look  ");
}

#[test]
fn test_action_form_with_special_characters() {
    let form: ActionForm =
        serde_json::from_str(r#"{"command": "go north & talk to guard"}"#).unwrap();
    assert_eq!(form.command, "go north & talk to guard");
}

#[test]
fn test_action_form_deserialize_unicode() {
    let form: ActionForm = serde_json::from_str(r#"{"command": "こんにちは"}"#).unwrap();
    assert_eq!(form.command, "こんにちは");
}

#[test]
fn test_action_form_roundtrip() {
    let original = ActionForm {
        command: "test command".to_string(),
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: ActionForm = serde_json::from_str(&json).unwrap();
    assert_eq!(original.command, parsed.command);
}

#[tokio::test]
async fn test_action_handler_empty_command() {
    let state = make_test_app_state();
    let form = ActionForm {
        command: String::new(),
    };
    let response = action_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Empty command should trigger continuation"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Thinking"),
        "Expected Thinking status: {body_str}"
    );
}

#[tokio::test]
async fn test_action_handler_whitespace_command() {
    let state = make_test_app_state();
    let form = ActionForm {
        command: "   ".to_string(),
    };
    let response = action_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("Thinking"));
}

#[tokio::test]
async fn test_action_handler_started() {
    let state = make_test_app_state();
    let form = ActionForm {
        command: "look".to_string(),
    };
    let response = action_handler(axum::extract::State(state), Form(form)).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_confirm_handler_empty_command() {
    let state = make_test_app_state();
    let form = ActionForm {
        command: String::new(),
    };
    let response = action_confirm_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Empty command should trigger continuation"
    );
}

#[tokio::test]
async fn test_action_confirm_handler_started() {
    let state = make_test_app_state();
    let form = ActionForm {
        command: "look".to_string(),
    };
    let response = action_confirm_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_check_handler_empty_command() {
    let state = make_test_app_state();
    let form = ActionForm {
        command: String::new(),
    };
    let response = action_check_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("Thinking"));
}

#[tokio::test]
async fn test_action_check_handler_disabled_mode() {
    let state = make_test_app_state();
    state.settings.write().unwrap().text_check.mode = TextCheckMode::Disabled;
    let form = ActionForm {
        command: "test".to_string(),
    };
    let response = action_check_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_check_handler_auto_check_disabled() {
    let state = make_test_app_state();
    state.settings.write().unwrap().text_check.enable_auto_check = false;
    let form = ActionForm {
        command: "test".to_string(),
    };
    let response = action_check_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_check_handler_check_result_none() {
    let state = make_test_app_state();
    let form = ActionForm {
        command: "go north".to_string(),
    };
    let response = action_check_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
}
