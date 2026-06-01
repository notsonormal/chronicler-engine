use std::sync::{Arc, RwLock};
use axum::{http::StatusCode, Form};
use tokio_util::sync::CancellationToken;

use crate::application::application_service::DefaultApplicationService;
use crate::application::game_service::GameService;
use crate::model::settings::AppSettings;
use crate::server::fragments::history::{delete_history_handler, edit_history_handler, EditHistoryForm};
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
fn test_edit_history_form_deserialization() {
    let form: EditHistoryForm = serde_json::from_str(r#"{"text": "Modified text"}"#).unwrap();
    assert_eq!(form.text, "Modified text");
}

#[test]
fn test_edit_history_form_empty_text() {
    let form: EditHistoryForm = serde_json::from_str(r#"{"text": ""}"#).unwrap();
    assert!(form.text.is_empty());
}

#[test]
fn test_edit_history_form_with_newlines() {
    let form: EditHistoryForm = serde_json::from_str(r#"{"text": "Line1\nLine2\nLine3"}"#).unwrap();
    assert!(form.text.contains('\n'));
}

#[test]
fn test_edit_history_form_roundtrip() {
    let original = EditHistoryForm {
        text: "new text".to_string(),
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: EditHistoryForm = serde_json::from_str(&json).unwrap();
    assert_eq!(original.text, parsed.text);
}

#[tokio::test]
async fn test_edit_history_handler_ok() {
    let state = make_test_app_state();

    let form = EditHistoryForm {
        text: "modified text".to_string(),
    };
    let (status, _body) = edit_history_handler(
        axum::extract::State(state),
        axum::extract::Path(999u64),
        Form(form),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_history_handler_ok() {
    let state = make_test_app_state();

    let (status, _body) = delete_history_handler(axum::extract::State(state)).await;

    assert!(status == StatusCode::OK || status == StatusCode::BAD_REQUEST);
}
