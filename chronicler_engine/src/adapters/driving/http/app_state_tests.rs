//! Unit tests for AppState helpers and shutdown-token wiring.

use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::adapters::driven::storage::Storage;
use crate::adapters::driving::http::AppState;
use crate::application::ports::text_checker::{CheckResult, TextChecker};
use crate::application::text_check_service::TextCheckService;
use crate::bootstrap::wiring::build_app_graph_for_tests;
use crate::domain::model::settings::{AppSettings, TextCheckMode};
use crate::error::EngineError;

struct NoopTextChecker;

impl TextChecker for NoopTextChecker {
    fn check(
        &self,
        _text: &str,
        _mode: TextCheckMode,
        _ignored_words: &[String],
    ) -> Result<Option<CheckResult>, EngineError> {
        Ok(None)
    }
}

fn build_app_state(settings: Arc<std::sync::RwLock<AppSettings>>) -> AppState {
    let wired = build_app_graph_for_tests(
        Arc::new(std::sync::RwLock::new(AppSettings::default())),
        Arc::new(Storage::new_in_memory()),
        Arc::new(Storage::new_in_memory()),
        None,
    )
    .expect("build_app_graph_for_tests should succeed");

    AppState {
        storage: Arc::new(Storage::new_in_memory()),
        preset_storage: Arc::new(Storage::new_in_memory()),
        persistence_gate: wired.persistence_gate,
        text_check_service: Arc::new(TextCheckService::new(Arc::new(NoopTextChecker))),
        settings,
        shutdown_token: wired.shutdown_token.clone(),
        pipeline: Arc::new(wired.pipeline),
        generation_gate: wired.generation_gate.clone(),
        game_catalogue: wired.game_catalogue.clone(),
        game_view_query: wired.game_view_query.clone(),
    }
}

#[test]
fn test_settings_recover_from_poisoned_rwlock() {
    let settings = Arc::new(std::sync::RwLock::new(AppSettings {
        narration_connection_id: "poison-test".to_string(),
        ..AppSettings::default()
    }));

    let settings_clone = Arc::clone(&settings);
    let _ = std::thread::spawn(move || {
        let _guard = settings_clone.write().unwrap();
        panic!("intentional panic to poison lock");
    })
    .join();

    let app_state = build_app_state(settings);

    let recovered = app_state.settings();
    assert_eq!(
        recovered.narration_connection_id, "poison-test",
        "settings() should recover actual settings from poisoned RwLock"
    );
}

#[test]
fn test_current_shutdown_token_returns_configured_token() {
    let token = CancellationToken::new();

    let wired = build_app_graph_for_tests(
        Arc::new(std::sync::RwLock::new(AppSettings::default())),
        Arc::new(Storage::new_in_memory()),
        Arc::new(Storage::new_in_memory()),
        None,
    )
    .expect("build_app_graph_for_tests should succeed");

    let app_state = AppState {
        storage: Arc::new(Storage::new_in_memory()),
        preset_storage: Arc::new(Storage::new_in_memory()),
        persistence_gate: wired.persistence_gate,
        text_check_service: Arc::new(TextCheckService::new(Arc::new(NoopTextChecker))),
        settings: Arc::new(std::sync::RwLock::new(AppSettings::default())),
        shutdown_token: token.clone(),
        pipeline: Arc::new(wired.pipeline),
        generation_gate: wired.generation_gate.clone(),
        game_catalogue: wired.game_catalogue.clone(),
        game_view_query: wired.game_view_query.clone(),
    };

    let recovered = app_state.current_shutdown_token();
    assert!(
        !recovered.is_cancelled(),
        "current_shutdown_token() should return the configured token"
    );
}
