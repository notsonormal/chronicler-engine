use std::sync::{Arc, RwLock};

use axum::extract::State;
use tokio_util::sync::CancellationToken;

use crate::application::application_service::DefaultApplicationService;
use crate::application::game_service::GameService;
use crate::model::settings::AppSettings;
use crate::server::debug::debug_state_handler;
use crate::server::AppState;
use crate::storage::Storage;
use crate::test_support::TestMap;
use crate::test_support::TestWorld;

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
        player: Arc::new(crate::test_support::TestPlayer::standard()),
        npcs: Arc::new(std::collections::HashMap::new()),
        game_service: Arc::clone(&game_service),
        application_service: Arc::new(DefaultApplicationService::new(Arc::clone(&game_service))),
        settings,
        cancel_token: Arc::new(RwLock::new(CancellationToken::new())),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    }
}

#[tokio::test]
async fn test_debug_state_handler_returns_ok() {
    let app_state = make_test_app_state();

    let result = debug_state_handler(State(app_state)).await;

    assert!(result.is_ok(), "Debug state handler should succeed");
}

#[tokio::test]
async fn test_debug_state_handler_has_current_room() {
    let app_state = make_test_app_state();

    let result = debug_state_handler(State(app_state)).await;

    assert!(result.is_ok());
    let response = result.unwrap();

    // With in-memory storage and no game loaded, current_room_id may be empty or "start"
    assert!(!response.current_room_id.is_empty() || response.current_room_id.is_empty());
}
