use std::sync::{Arc, RwLock};

use crate::error::EngineError;
use crate::model::prompt_preset::{PresetType, PromptPreset};
use crate::server::prompt_presets_fragment::handlers::{
    PresetForm, activate_preset_handler, delete_preset_handler, duplicate_preset_handler,
    edit_preset_form_handler, panel_handler, preset_card_handler, save_preset_handler,
    update_preset_handler, view_preset_form_handler,
};
use crate::storage::prompt_preset_storage::{InMemoryPromptPresetStorage, PromptPresetStorage};

/// Wrapper that delegates to an inner storage but can be configured to fail
/// on specific operations, enabling error-branch coverage.
struct FailingPromptPresetStorage {
    inner: InMemoryPromptPresetStorage,
    fail_save: bool,
    fail_get: bool,
    fail_delete: bool,
}

impl FailingPromptPresetStorage {
    fn new() -> Self {
        Self {
            inner: InMemoryPromptPresetStorage::new(),
            fail_save: false,
            fail_get: false,
            fail_delete: false,
        }
    }

    fn set_fail_save(&mut self) {
        self.fail_save = true;
    }

    fn set_fail_get(&mut self) {
        self.fail_get = true;
    }

    fn set_fail_delete(&mut self) {
        self.fail_delete = true;
    }
}

impl PromptPresetStorage for FailingPromptPresetStorage {
    fn list(&self, preset_type: PresetType) -> Result<Vec<PromptPreset>, EngineError> {
        self.inner.list(preset_type)
    }

    fn get(&self, id: &str) -> Result<Option<PromptPreset>, EngineError> {
        if self.fail_get {
            return Err(EngineError::Config("injected get failure".into()));
        }
        self.inner.get(id)
    }

    fn save(&self, preset: &PromptPreset) -> Result<(), EngineError> {
        if self.fail_save {
            return Err(EngineError::Config("injected save failure".into()));
        }
        self.inner.save(preset)
    }

    fn delete(&self, id: &str) -> Result<(), EngineError> {
        if self.fail_delete {
            return Err(EngineError::Config("injected delete failure".into()));
        }
        self.inner.delete(id)
    }
}

fn make_test_app_state_with_preset(preset: PromptPreset) -> crate::server::AppState {
    make_test_app_state_with_storage(Arc::new(InMemoryPromptPresetStorage::new()), preset)
}

fn make_test_app_state_with_storage(
    storage: Arc<dyn PromptPresetStorage>,
    preset: PromptPreset,
) -> crate::server::AppState {
    let _ = storage.save(&preset);

    let settings = Arc::new(RwLock::new(crate::model::settings::AppSettings::default()));
    let game_service: Arc<dyn crate::application::game_service::GameService> = Arc::new(
        crate::application::game_service::DefaultGameService::with_storage(
            Some(Arc::new(
                crate::storage::llm_message_storage::InMemoryLlmMessageStorage::new(),
            )),
            None,
            Arc::clone(&settings),
        ),
    );
    crate::server::AppState {
        game_storage: Arc::new(crate::test_support::InMemoryGameRepository::new()),
        snapshot_storage: Arc::new(crate::test_support::InMemorySnapshotRepository::new()),
        message_storage: Arc::new(crate::test_support::InMemoryMessageRepository::new()),
        message_swipe_storage: Arc::new(
            crate::test_support::in_memory_storage::InMemoryMessageSwipeStorage::new(),
        ),
        llm_message_storage: Arc::new(
            crate::storage::llm_message_storage::InMemoryLlmMessageStorage::new(),
        ),
        world: Arc::new(crate::test_support::TestWorld::minimal()),
        map: Arc::new(crate::test_support::TestMap::single_room("start")),
        player: Arc::new(crate::test_support::TestPlayer::standard()),
        npcs: Arc::new(std::collections::HashMap::new()),
        game_service: Arc::clone(&game_service),
        application_service: Arc::new(
            crate::application::application_service::DefaultApplicationService::new(game_service),
        ),
        prompt_preset_storage: storage,
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
    let app_state = make_test_app_state_with_preset(PromptPreset {
        id: "x".into(),
        name: "X".into(),
        instructions: Some("X.".into()),
        ..Default::default()
    });
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
    let app_state = make_test_app_state_with_preset(PromptPreset {
        id: "x".into(),
        name: "X".into(),
        instructions: Some("X.".into()),
        ..Default::default()
    });
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
    let app_state = make_test_app_state_with_preset(PromptPreset {
        id: "x".into(),
        name: "X".into(),
        instructions: Some("X.".into()),
        ..Default::default()
    });
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
        FailingPromptPresetStorage::new(),
        PromptPreset {
            id: "orig".into(),
            name: "Original".into(),
            instructions: Some("Original prompt.".into()),
            preset_type: PresetType::System,
            ..Default::default()
        },
        |s| s.set_fail_save(),
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
    let app_state = make_test_app_state_with_preset(PromptPreset {
        id: "x".into(),
        name: "X".into(),
        instructions: Some("X.".into()),
        ..Default::default()
    });
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
    let preset = PromptPreset {
        id: "custom".into(),
        name: "Custom".into(),
        instructions: Some("Custom.".into()),
        ..Default::default()
    };
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
    let app_state = make_test_app_state_with_preset(PromptPreset {
        id: "x".into(),
        name: "X".into(),
        instructions: Some("X.".into()),
        ..Default::default()
    });
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
    let app_state = make_test_app_state_with_preset(PromptPreset {
        id: "x".into(),
        name: "X".into(),
        instructions: Some("X.".into()),
        ..Default::default()
    });

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
    mut storage: FailingPromptPresetStorage,
    preset: PromptPreset,
    fail_after_setup: impl FnOnce(&mut FailingPromptPresetStorage),
) -> crate::server::AppState {
    let _ = storage.save(&preset);
    fail_after_setup(&mut storage);

    let settings = Arc::new(RwLock::new(crate::model::settings::AppSettings::default()));
    let game_service: Arc<dyn crate::application::game_service::GameService> = Arc::new(
        crate::application::game_service::DefaultGameService::with_storage(
            Some(Arc::new(
                crate::storage::llm_message_storage::InMemoryLlmMessageStorage::new(),
            )),
            None,
            Arc::clone(&settings),
        ),
    );
    crate::server::AppState {
        game_storage: Arc::new(crate::test_support::InMemoryGameRepository::new()),
        snapshot_storage: Arc::new(crate::test_support::InMemorySnapshotRepository::new()),
        message_storage: Arc::new(crate::test_support::InMemoryMessageRepository::new()),
        message_swipe_storage: Arc::new(
            crate::test_support::in_memory_storage::InMemoryMessageSwipeStorage::new(),
        ),
        llm_message_storage: Arc::new(
            crate::storage::llm_message_storage::InMemoryLlmMessageStorage::new(),
        ),
        world: Arc::new(crate::test_support::TestWorld::minimal()),
        map: Arc::new(crate::test_support::TestMap::single_room("start")),
        player: Arc::new(crate::test_support::TestPlayer::standard()),
        npcs: Arc::new(std::collections::HashMap::new()),
        game_service: Arc::clone(&game_service),
        application_service: Arc::new(
            crate::application::application_service::DefaultApplicationService::new(game_service),
        ),
        prompt_preset_storage: Arc::new(storage),
        settings,
        cancel_token: Arc::new(RwLock::new(tokio_util::sync::CancellationToken::new())),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    }
}

#[tokio::test]
async fn test_save_preset_storage_error_returns_error() {
    let app_state = make_test_app_state_with_failing_storage(
        FailingPromptPresetStorage::new(),
        PromptPreset {
            id: "x".into(),
            name: "X".into(),
            instructions: Some("X.".into()),
            ..Default::default()
        },
        |s| s.set_fail_save(),
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
        FailingPromptPresetStorage::new(),
        PromptPreset {
            id: "custom".into(),
            name: "Custom".into(),
            instructions: Some("Custom.".into()),
            ..Default::default()
        },
        |s| s.set_fail_get(),
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
        FailingPromptPresetStorage::new(),
        PromptPreset {
            id: "custom".into(),
            name: "Custom".into(),
            instructions: Some("Custom.".into()),
            ..Default::default()
        },
        |s| s.set_fail_save(),
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
        FailingPromptPresetStorage::new(),
        PromptPreset {
            id: "custom".into(),
            name: "Custom".into(),
            instructions: Some("Custom.".into()),
            ..Default::default()
        },
        |s| s.set_fail_get(),
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
        FailingPromptPresetStorage::new(),
        PromptPreset {
            id: "custom".into(),
            name: "Custom".into(),
            instructions: Some("Custom.".into()),
            ..Default::default()
        },
        |s| s.set_fail_delete(),
    );
    let response = delete_preset_handler(
        axum::extract::State(app_state),
        axum::extract::Path("custom".to_string()),
    )
    .await;
    assert!(response.0.contains("Delete failed"));
}
