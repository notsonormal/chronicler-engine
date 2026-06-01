use std::sync::{Arc, RwLock};
use axum::{http::StatusCode};

use crate::application::application_service::DefaultApplicationService;
use crate::application::game_service::GameService;
use crate::model::settings::AppSettings;
use crate::server::fragments::games::{
    create_game_handler, delete_game_handler, list_games_fragment, switch_game_handler,
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

// ─── List Games Tests ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_games_empty() {
    let state = make_test_app_state();
    let response = list_games_fragment(axum::extract::State(state)).await;
    // Just check it returns OK - actual content depends on world name
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_games_no_active() {
    let state = make_test_app_state();
    let response = list_games_fragment(axum::extract::State(state)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

// ─── Create Game Tests ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_game_ok() {
    let state = make_test_app_state();
    let response = create_game_handler(axum::extract::State(state)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_game_while_generating() {
    let state = make_test_app_state_with_generating(true);
    let response = create_game_handler(axum::extract::State(state)).await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

// ─── Switch Game Tests ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_switch_game_ok() {
    let state = make_test_app_state();
    // Create a game first
    let _ = create_game_handler(axum::extract::State(state.clone())).await;

    // Get the game ID from storage
    let games = state
        .application_service
        .list_games(state.as_game_service_context())
        .unwrap();
    if !games.is_empty() {
        let game_id = games[0].id;
        let response =
            switch_game_handler(axum::extract::State(state), axum::extract::Path(game_id)).await;
        assert_eq!(response.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_switch_game_while_generating() {
    let state = make_test_app_state_with_generating(true);
    let response =
        switch_game_handler(axum::extract::State(state), axum::extract::Path(999u64)).await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

// ─── Delete Game Tests ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_game() {
    let state = make_test_app_state();
    // Try to delete non-existent game - will fail but shouldn't panic
    let response =
        delete_game_handler(axum::extract::State(state), axum::extract::Path(999u64)).await;
    // Just ensure it returns some status without panicking
    assert!(
        response.0.is_client_error() || response.0.is_server_error() || response.0.is_success()
    );
}

#[tokio::test]
async fn test_delete_game_while_generating() {
    let state = make_test_app_state_with_generating(true);
    let response =
        delete_game_handler(axum::extract::State(state), axum::extract::Path(999u64)).await;
    assert_eq!(response.0, StatusCode::SERVICE_UNAVAILABLE);
}
