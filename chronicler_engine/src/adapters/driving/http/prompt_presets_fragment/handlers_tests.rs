use std::sync::{Arc, RwLock};

use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::adapters::driving::http::prompt_presets_fragment::handlers::{
    PresetForm, activate_preset_handler, delete_preset_handler, duplicate_preset_handler,
    edit_preset_form_handler, panel_handler, preset_card_handler, save_preset_handler,
    update_preset_handler, view_preset_form_handler,
};
use crate::adapters::driven::storage::{Storage, TestOverride};
use crate::test_support::TestPromptPreset;

fn make_test_app_state_with_preset(
    preset: PromptPreset,
) -> crate::adapters::driving::http::AppState {
    make_test_app_state_with_storage(Arc::new(Storage::new_in_memory()), preset)
}

fn make_test_app_state_with_storage(
    storage: Arc<Storage>,
    preset: PromptPreset,
) -> crate::adapters::driving::http::AppState {
    let _ = storage.save_preset(&preset);

    let settings = Arc::new(RwLock::new(
        crate::domain::model::settings::AppSettings::default(),
    ));
    let game_service = Arc::new(
        crate::bootstrap::wiring::build_game_service_for_tests(
            Arc::clone(&settings),
            Arc::new(Storage::new_in_memory()),
            Arc::new(Storage::new_in_memory()),
        )
        .expect("build_game_service_for_tests should succeed"),
    );
    let text_check_service = Arc::new(
        crate::bootstrap::text_check_factory::create_text_check_service(&settings.read().unwrap()),
    );
    crate::adapters::driving::http::AppState {
        storage: Arc::clone(&storage),
        preset_storage: Arc::clone(&storage),
        game_service: Arc::clone(&game_service),
        application_service: Arc::new(
            crate::application::application_service::DefaultApplicationService::new(
                Arc::clone(&storage),
                Arc::new(Storage::new_in_memory()),
                Arc::clone(&settings),
                tokio_util::sync::CancellationToken::new(),
                Arc::new(std::sync::atomic::AtomicBool::new(false)),
                Arc::clone(&game_service),
            ),
        ),
        text_check_service,
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
    let preset =
        TestPromptPreset::system_default_with_instructions("default", "Default", "System prompt.");
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
        |h| h.set("save_preset", TestOverride::config("injected save failure")),
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
    let preset =
        TestPromptPreset::system_default_with_instructions("default", "Default", "System prompt.");
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
    let preset =
        TestPromptPreset::system_default_with_instructions("default", "Default", "System prompt.");
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
    let preset =
        TestPromptPreset::system_default_with_instructions("default", "Default", "System prompt.");
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
async fn test_panel_handler_with_poisoned_settings_lock() {
    let app_state =
        make_test_app_state_with_preset(crate::test_support::TestPromptPreset::system("x", "X"));

    // [DOC: docs/reference/testing.md#poisoned-lock-testing]
    let settings_clone = Arc::clone(&app_state.settings);
    let handle = tokio::task::spawn_blocking(move || {
        let _guard = settings_clone.write().unwrap();
        panic!("intentional panic to poison lock");
    });
    let _ = handle.await;

    let response = panel_handler(axum::extract::State(app_state)).await;
    assert!(
        response.0.contains("System Prompts"),
        "Panel should render even with poisoned lock: {}",
        response.0
    );
}

fn make_test_app_state_with_failing_storage(
    preset: PromptPreset,
    fail_after_setup: impl FnOnce(&crate::adapters::driven::storage::TestFailureHandle),
) -> crate::adapters::driving::http::AppState {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    let _ = storage.save_preset(&preset);
    fail_after_setup(&handle);

    let settings = Arc::new(RwLock::new(
        crate::domain::model::settings::AppSettings::default(),
    ));
    let game_service = Arc::new(
        crate::bootstrap::wiring::build_game_service_for_tests(
            Arc::clone(&settings),
            Arc::new(Storage::new_in_memory()),
            Arc::new(Storage::new_in_memory()),
        )
        .expect("build_game_service_for_tests should succeed"),
    );
    let text_check_service = Arc::new(
        crate::bootstrap::text_check_factory::create_text_check_service(&settings.read().unwrap()),
    );
    let preset_storage = Arc::new(storage);
    let application_service = Arc::new(
        crate::application::application_service::DefaultApplicationService::new(
            Arc::new(Storage::new_in_memory()),
            Arc::clone(&preset_storage),
            Arc::clone(&settings),
            tokio_util::sync::CancellationToken::new(),
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
            Arc::clone(&game_service),
        ),
    );
    crate::adapters::driving::http::AppState {
        storage: Arc::new(Storage::new_in_memory()),
        preset_storage,
        game_service: Arc::clone(&game_service),
        application_service,
        text_check_service,
        settings,
        cancel_token: Arc::new(RwLock::new(tokio_util::sync::CancellationToken::new())),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    }
}

#[tokio::test]
async fn test_save_preset_storage_error_returns_error() {
    let app_state = make_test_app_state_with_failing_storage(
        crate::test_support::TestPromptPreset::system("x", "X"),
        |h| h.set("save_preset", TestOverride::config("injected save failure")),
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
        |h| h.set("get_preset", TestOverride::config("injected get failure")),
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
        |h| h.set("save_preset", TestOverride::config("injected save failure")),
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
        |h| h.set("get_preset", TestOverride::config("injected get failure")),
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
                "delete_preset",
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
