use std::sync::{Arc, RwLock};

use crate::model::prompt_preset::{PresetType, PromptPreset};
use crate::server::prompt_presets_fragment::handlers::{
    PresetForm, activate_preset_handler, delete_preset_handler, duplicate_preset_handler,
    edit_preset_form_handler, panel_handler, preset_card_handler, save_preset_handler,
    update_preset_handler, view_preset_form_handler,
};
use crate::storage::{Operation, Storage, TestOverride};

fn make_test_app_state_with_preset(preset: PromptPreset) -> crate::server::AppState {
    make_test_app_state_with_storage(Arc::new(Storage::new_in_memory()), preset)
}

fn make_test_app_state_with_storage(
    storage: Arc<Storage>,
    preset: PromptPreset,
) -> crate::server::AppState {
    let _ = storage.save_preset(&preset);

    let settings = Arc::new(RwLock::new(crate::model::settings::AppSettings::default()));
    let game_service = Arc::new(crate::application::game_service::GameService::with_storage(
        Some(Arc::new(Storage::new_in_memory())),
        None,
        Arc::clone(&settings),
    ));
    crate::server::AppState {
        storage: Arc::new(Storage::new_in_memory()),
        preset_storage: storage,
        world: Arc::new(crate::test_support::TestWorld::minimal()),
        map: Arc::new(crate::test_support::TestMap::single_room("start")),
        player: Arc::new(crate::test_support::TestPlayer::standard()),
        npcs: Arc::new(std::collections::HashMap::new()),
        game_service: Arc::clone(&game_service),
        application_service: Arc::new(
            crate::application::application_service::DefaultApplicationService::new(Arc::clone(
                &game_service,
            )),
        ),
        settings,
        cancel_token: Arc::new(RwLock::new(tokio_util::sync::CancellationToken::new())),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    }
}

#[tokio::test]
async fn test_preset_card_handler_returns_card() {
    let preset = PromptPreset {
        id: "card-test".into(),
        name: "Card Test".into(),
        instructions: Some("Test.".into()),
        preset_type: PresetType::System,
        ..Default::default()
    };
    let app_state = make_test_app_state_with_preset(preset.clone());
    let response = preset_card_handler(
        axum::extract::State(app_state),
        axum::extract::Path("card-test".to_string()),
    )
    .await;
    assert!(response.0.contains("Card Test"));
    assert!(response.0.contains("Set Active</button>"));
}

#[tokio::test]
async fn test_preset_card_handler_not_found() {
    let app_state =
        make_test_app_state_with_preset(crate::test_support::TestPromptPreset::system("x", "X"));
    let response = preset_card_handler(
        axum::extract::State(app_state),
        axum::extract::Path("missing".to_string()),
    )
    .await;
    assert!(response.0.contains("Preset not found"));
}

#[tokio::test]
async fn test_view_preset_form_handler_default_preset() {
    let preset = PromptPreset {
        id: "default".into(),
        name: "Default".into(),
        instructions: Some("System prompt.".into()),
        is_default: true,
        preset_type: PresetType::System,
        ..Default::default()
    };
    let app_state = make_test_app_state_with_preset(preset.clone());
    let response = view_preset_form_handler(
        axum::extract::State(app_state),
        axum::extract::Path("default".to_string()),
    )
    .await;
    assert!(response.0.contains("View Default"));
    assert!(response.0.contains("System prompt."));
    assert!(response.0.contains("disabled"));
}

#[tokio::test]
async fn test_view_preset_form_handler_not_found() {
    let app_state =
        make_test_app_state_with_preset(crate::test_support::TestPromptPreset::system("x", "X"));
    let response = view_preset_form_handler(
        axum::extract::State(app_state),
        axum::extract::Path("missing".to_string()),
    )
    .await;
    assert!(response.0.contains("Preset not found"));
}

#[tokio::test]
async fn test_duplicate_preset_handler() {
    let preset = PromptPreset {
        id: "orig".into(),
        name: "Original".into(),
        instructions: Some("Original prompt.".into()),
        preset_type: PresetType::System,
        ..Default::default()
    };
    let app_state = make_test_app_state_with_preset(preset.clone());
    let response = duplicate_preset_handler(
        axum::extract::State(app_state),
        axum::extract::Path("orig".to_string()),
    )
    .await;
    assert!(response.0.contains("Original (Copy)"));
    assert!(response.0.contains("System Prompts"));
}

#[tokio::test]
async fn test_duplicate_preset_handler_not_found() {
    let app_state =
        make_test_app_state_with_preset(crate::test_support::TestPromptPreset::system("x", "X"));
    let response = duplicate_preset_handler(
        axum::extract::State(app_state),
        axum::extract::Path("missing".to_string()),
    )
    .await;
    assert!(response.0.contains("Preset not found"));
}

#[tokio::test]
async fn test_duplicate_preset_storage_error_returns_error() {
    let app_state = make_test_app_state_with_failing_storage(
        PromptPreset {
            id: "orig".into(),
            name: "Original".into(),
            instructions: Some("Original prompt.".into()),
            preset_type: PresetType::System,
            ..Default::default()
        },
        |h| {
            h.set(
                Operation::SavePreset,
                TestOverride::config("injected save failure"),
            )
        },
    );
    let response = duplicate_preset_handler(
        axum::extract::State(app_state),
        axum::extract::Path("orig".to_string()),
    )
    .await;
    assert!(response.0.contains("Duplicate failed"));
}

#[tokio::test]
async fn test_edit_default_preset_returns_error() {
    let preset = PromptPreset {
        id: "default".into(),
        name: "Default".into(),
        instructions: Some("System prompt.".into()),
        is_default: true,
        preset_type: PresetType::System,
        ..Default::default()
    };
    let app_state = make_test_app_state_with_preset(preset.clone());
    let response = edit_preset_form_handler(
        axum::extract::State(app_state),
        axum::extract::Path("default".to_string()),
    )
    .await;
    assert!(response.0.contains("Cannot edit default presets"));
}

#[tokio::test]
async fn test_update_default_preset_returns_error() {
    let preset = PromptPreset {
        id: "default".into(),
        name: "Default".into(),
        instructions: Some("System prompt.".into()),
        is_default: true,
        preset_type: PresetType::System,
        ..Default::default()
    };
    let app_state = make_test_app_state_with_preset(preset.clone());
    let response = update_preset_handler(
        axum::extract::State(app_state),
        axum::extract::Path("default".to_string()),
        axum::extract::Form(PresetForm {
            name: "Updated".into(),
            instructions: Some("Updated.".into()),
            preset_type: "system".into(),
            ..Default::default()
        }),
    )
    .await;
    assert!(response.0.contains("Cannot edit default presets"));
}

#[tokio::test]
async fn test_delete_default_preset_returns_error() {
    let preset = PromptPreset {
        id: "default".into(),
        name: "Default".into(),
        instructions: Some("System prompt.".into()),
        is_default: true,
        preset_type: PresetType::System,
        ..Default::default()
    };
    let app_state = make_test_app_state_with_preset(preset.clone());
    let response = delete_preset_handler(
        axum::extract::State(app_state),
        axum::extract::Path("default".to_string()),
    )
    .await;
    assert!(response.0.contains("Cannot delete default presets"));
}

#[tokio::test]
async fn test_save_preset_invalid_type_returns_error() {
    let app_state =
        make_test_app_state_with_preset(crate::test_support::TestPromptPreset::system("x", "X"));
    let response = save_preset_handler(
        axum::extract::State(app_state),
        axum::extract::Form(PresetForm {
            name: "Test".into(),
            instructions: Some("Test.".into()),
            preset_type: "invalid".into(),
            ..Default::default()
        }),
    )
    .await;
    assert!(response.0.contains("Invalid preset type"));
}

#[tokio::test]
async fn test_update_preset_invalid_type_returns_error() {
    let preset = crate::test_support::TestPromptPreset::system("custom", "Custom");
    let app_state = make_test_app_state_with_preset(preset.clone());
    let response = update_preset_handler(
        axum::extract::State(app_state),
        axum::extract::Path("custom".to_string()),
        axum::extract::Form(PresetForm {
            name: "Updated".into(),
            instructions: Some("Updated.".into()),
            preset_type: "invalid".into(),
            ..Default::default()
        }),
    )
    .await;
    assert!(response.0.contains("Invalid preset type"));
}

#[tokio::test]
async fn test_activate_nonexistent_preset_returns_error() {
    let app_state =
        make_test_app_state_with_preset(crate::test_support::TestPromptPreset::system("x", "X"));
    let response = activate_preset_handler(
        axum::extract::State(app_state),
        axum::extract::Path("missing".to_string()),
    )
    .await;
    assert!(response.0.contains("Preset not found"));
}

#[tokio::test]
async fn test_activate_preset_settings_save_error_returns_error() {
    // Set an invalid settings path so settings.save() fails.
    let invalid_path = format!(
        "{}\\chronicler_test_invalid_{}\\settings.json",
        std::env::temp_dir().display(),
        std::process::id()
    );
    unsafe { std::env::set_var("CHRONICLER_SETTINGS_PATH", &invalid_path) };

    let app_state = make_test_app_state_with_preset(PromptPreset {
        id: "active-test".into(),
        name: "Active Test".into(),
        instructions: Some("Active prompt.".into()),
        ..Default::default()
    });
    let response = activate_preset_handler(
        axum::extract::State(app_state),
        axum::extract::Path("active-test".to_string()),
    )
    .await;

    // Clean up env var
    unsafe { std::env::remove_var("CHRONICLER_SETTINGS_PATH") };

    assert!(response.0.contains("Save failed"));
}

#[tokio::test]
async fn test_panel_handler_with_poisoned_settings_lock() {
    let app_state =
        make_test_app_state_with_preset(crate::test_support::TestPromptPreset::system("x", "X"));

    // Poison the settings lock by panicking while holding a write guard.
    // [DOC: docs/reference/testing.md#poisoned-lock-testing]
    let settings_clone = Arc::clone(&app_state.settings);
    let handle = tokio::task::spawn_blocking(move || {
        let _guard = settings_clone.write().unwrap();
        panic!("intentional panic to poison lock");
    });
    let _ = handle.await;

    // panel_handler should recover from the poisoned lock via try_lock!.
    let response = panel_handler(axum::extract::State(app_state)).await;
    assert!(
        response.0.contains("System Prompts"),
        "Panel should render even with poisoned lock: {}",
        response.0
    );
}

fn make_test_app_state_with_failing_storage(
    preset: PromptPreset,
    fail_after_setup: impl FnOnce(&crate::storage::TestFailureHandle),
) -> crate::server::AppState {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    let _ = storage.save_preset(&preset);
    fail_after_setup(&handle);

    let settings = Arc::new(RwLock::new(crate::model::settings::AppSettings::default()));
    let game_service = Arc::new(crate::application::game_service::GameService::with_storage(
        Some(Arc::new(Storage::new_in_memory())),
        None,
        Arc::clone(&settings),
    ));
    crate::server::AppState {
        storage: Arc::new(Storage::new_in_memory()),
        preset_storage: Arc::new(storage),
        world: Arc::new(crate::test_support::TestWorld::minimal()),
        map: Arc::new(crate::test_support::TestMap::single_room("start")),
        player: Arc::new(crate::test_support::TestPlayer::standard()),
        npcs: Arc::new(std::collections::HashMap::new()),
        game_service: Arc::clone(&game_service),
        application_service: Arc::new(
            crate::application::application_service::DefaultApplicationService::new(Arc::clone(
                &game_service,
            )),
        ),
        settings,
        cancel_token: Arc::new(RwLock::new(tokio_util::sync::CancellationToken::new())),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    }
}

#[tokio::test]
async fn test_save_preset_storage_error_returns_error() {
    let app_state = make_test_app_state_with_failing_storage(
        crate::test_support::TestPromptPreset::system("x", "X"),
        |h| {
            h.set(
                Operation::SavePreset,
                TestOverride::config("injected save failure"),
            )
        },
    );
    let response = save_preset_handler(
        axum::extract::State(app_state),
        axum::extract::Form(PresetForm {
            name: "Test".into(),
            instructions: Some("Test.".into()),
            preset_type: "system".into(),
            ..Default::default()
        }),
    )
    .await;
    assert!(response.0.contains("Save failed"));
}

#[tokio::test]
async fn test_edit_preset_storage_error_returns_error() {
    let app_state = make_test_app_state_with_failing_storage(
        PromptPreset {
            id: "custom".into(),
            name: "Custom".into(),
            instructions: Some("Custom.".into()),
            ..Default::default()
        },
        |h| {
            h.set(
                Operation::GetPreset,
                TestOverride::config("injected get failure"),
            )
        },
    );
    let response = edit_preset_form_handler(
        axum::extract::State(app_state),
        axum::extract::Path("custom".to_string()),
    )
    .await;
    assert!(response.0.contains("Load failed"));
}

#[tokio::test]
async fn test_update_preset_storage_error_returns_error() {
    let app_state = make_test_app_state_with_failing_storage(
        PromptPreset {
            id: "custom".into(),
            name: "Custom".into(),
            instructions: Some("Custom.".into()),
            ..Default::default()
        },
        |h| {
            h.set(
                Operation::SavePreset,
                TestOverride::config("injected save failure"),
            )
        },
    );
    let response = update_preset_handler(
        axum::extract::State(app_state),
        axum::extract::Path("custom".to_string()),
        axum::extract::Form(PresetForm {
            name: "Updated".into(),
            instructions: Some("Updated.".into()),
            preset_type: "system".into(),
            ..Default::default()
        }),
    )
    .await;
    assert!(response.0.contains("Update failed"));
}

#[tokio::test]
async fn test_delete_preset_get_storage_error_returns_error() {
    let app_state = make_test_app_state_with_failing_storage(
        PromptPreset {
            id: "custom".into(),
            name: "Custom".into(),
            instructions: Some("Custom.".into()),
            ..Default::default()
        },
        |h| {
            h.set(
                Operation::GetPreset,
                TestOverride::config("injected get failure"),
            )
        },
    );
    let response = delete_preset_handler(
        axum::extract::State(app_state),
        axum::extract::Path("custom".to_string()),
    )
    .await;
    assert!(response.0.contains("Load failed"));
}

#[tokio::test]
async fn test_delete_preset_delete_storage_error_returns_error() {
    let app_state = make_test_app_state_with_failing_storage(
        PromptPreset {
            id: "custom".into(),
            name: "Custom".into(),
            instructions: Some("Custom.".into()),
            ..Default::default()
        },
        |h| {
            h.set(
                Operation::DeletePreset,
                TestOverride::config("injected delete failure"),
            )
        },
    );
    let response = delete_preset_handler(
        axum::extract::State(app_state),
        axum::extract::Path("custom".to_string()),
    )
    .await;
    assert!(response.0.contains("Delete failed"));
}
