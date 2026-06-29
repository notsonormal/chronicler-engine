use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use chronicler_engine::application::game_service::GameService;
use chronicler_engine::application::DefaultApplicationService;
use chronicler_engine::domain::model::settings::AppSettings;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::adapters::driven::storage::Storage;

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

    let game_service = Arc::new(GameService::with_storage(
        None,
        None,
        Arc::new(std::sync::RwLock::new(AppSettings::default())),
    ));
    let app_state = AppState {
        storage: Arc::new(Storage::new_in_memory()),
        preset_storage: Arc::new(Storage::new_in_memory()),
        game_service: Arc::clone(&game_service),
        application_service: Arc::new(DefaultApplicationService::new(Arc::clone(&game_service))),
        settings,
        cancel_token: Arc::new(std::sync::RwLock::new(CancellationToken::new())),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
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
    let cancel_token = Arc::new(std::sync::RwLock::new(token.clone()));

    let cancel_clone = Arc::clone(&cancel_token);
    let _ = std::thread::spawn(move || {
        let _guard = cancel_clone.write().unwrap();
        panic!("intentional panic to poison lock");
    })
    .join();

    let game_service = Arc::new(GameService::with_storage(
        None,
        None,
        Arc::new(std::sync::RwLock::new(AppSettings::default())),
    ));
    let app_state = AppState {
        storage: Arc::new(Storage::new_in_memory()),
        preset_storage: Arc::new(Storage::new_in_memory()),
        game_service: Arc::clone(&game_service),
        application_service: Arc::new(DefaultApplicationService::new(Arc::clone(&game_service))),
        settings: Arc::new(std::sync::RwLock::new(AppSettings::default())),
        cancel_token,
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    };

    let recovered = app_state.current_cancel_token();
    assert!(
        !recovered.is_cancelled(),
        "current_cancel_token() should recover the actual token from poisoned RwLock"
    );
}
