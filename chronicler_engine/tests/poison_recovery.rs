//! Tests for poison-recovery behaviour: confirms that a poisoned `RwLock` inside the settings layer and `CancellationToken` machinery does not crash subsequent operations.

use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use chronicler_engine::application::text_check_service::TextCheckService;
use chronicler_engine::application::ports::text_checker::TextChecker;
use chronicler_engine::domain::model::settings::AppSettings;
use chronicler_engine::domain::model::settings::TextCheckMode;
use chronicler_engine::adapters::driving::http::{AppState, write_lock_or_recover};
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::error::EngineError;
use chronicler_engine::application::ports::text_checker::CheckResult;

fn test_game_service() -> Arc<chronicler_engine::application::game_service::GameService> {
    chronicler_engine::bootstrap::wiring::build_app_graph_for_tests(
        Arc::new(std::sync::RwLock::new(AppSettings::default())),
        Arc::new(Storage::new_in_memory()),
        Arc::new(Storage::new_in_memory()),
        None,
    )
    .expect("build_app_graph_for_tests should succeed")
    .game_service
}

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

    let game_service = test_game_service();
    let wired = chronicler_engine::bootstrap::wiring::build_app_graph_for_tests(
        Arc::new(std::sync::RwLock::new(AppSettings::default())),
        Arc::new(Storage::new_in_memory()),
        Arc::new(Storage::new_in_memory()),
        Some(Arc::clone(&game_service)),
    )
    .expect("build_app_graph_for_tests should succeed");
    let app_state = AppState {
        storage: Arc::new(Storage::new_in_memory()),
        preset_storage: Arc::new(Storage::new_in_memory()),
        game_service: Arc::clone(&game_service),
        application_service: wired.application_service,
        persistence_gate: wired.persistence_gate,
        text_check_service: Arc::new(TextCheckService::new(Arc::new(NoopTextChecker))),
        settings,
        shutdown_token: Arc::new(std::sync::RwLock::new(CancellationToken::new())),
    };

    let recovered = app_state.settings();
    assert_eq!(
        recovered.narration_connection_id, "poison-test",
        "settings() should recover actual settings from poisoned RwLock"
    );
}

#[test]
fn test_cancel_token_recover_from_poisoned_rwlock() {
    let token = CancellationToken::new();
    let shutdown_token = Arc::new(std::sync::RwLock::new(token.clone()));

    let cancel_clone = Arc::clone(&shutdown_token);
    let _ = std::thread::spawn(move || {
        let _guard = cancel_clone.write().unwrap();
        panic!("intentional panic to poison lock");
    })
    .join();

    let game_service = test_game_service();
    let wired = chronicler_engine::bootstrap::wiring::build_app_graph_for_tests(
        Arc::new(std::sync::RwLock::new(AppSettings::default())),
        Arc::new(Storage::new_in_memory()),
        Arc::new(Storage::new_in_memory()),
        Some(Arc::clone(&game_service)),
    )
    .expect("build_app_graph_for_tests should succeed");
    let app_state = AppState {
        storage: Arc::new(Storage::new_in_memory()),
        preset_storage: Arc::new(Storage::new_in_memory()),
        game_service: Arc::clone(&game_service),
        application_service: wired.application_service,
        persistence_gate: wired.persistence_gate,
        text_check_service: Arc::new(TextCheckService::new(Arc::new(NoopTextChecker))),
        settings: Arc::new(std::sync::RwLock::new(AppSettings::default())),
        shutdown_token,
    };

    let recovered = app_state.current_shutdown_token();
    assert!(
        !recovered.is_cancelled(),
        "current_shutdown_token() should recover the actual token from poisoned RwLock"
    );
}

#[test]
fn test_write_lock_recover_from_poisoned_rwlock() {
    let lock = Arc::new(std::sync::RwLock::new(0));

    let lock_clone = Arc::clone(&lock);
    let _ = std::thread::spawn(move || {
        let mut guard = lock_clone.write().unwrap();
        *guard = 42;
        panic!("intentional panic to poison lock");
    })
    .join();

    let recovered = write_lock_or_recover(&lock, "test");
    assert_eq!(*recovered, 42);
}
