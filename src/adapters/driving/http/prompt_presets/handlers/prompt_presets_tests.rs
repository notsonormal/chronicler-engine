//! Prompt preset HTTP handler tests.

use std::sync::{Arc, RwLock};

use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::NarratorMode;
use crate::adapters::driving::http::prompt_presets::handlers::{
    ActivateQuery, PresetForm, activate_preset_handler, delete_preset_handler,
    duplicate_preset_handler, edit_preset_form_handler, panel_handler, preset_card_handler,
    save_preset_handler, update_preset_handler, view_preset_form_handler,
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

    let wired = crate::bootstrap::wiring::build_app_graph_for_tests(
        Arc::new(RwLock::new(
            crate::domain::model::settings::AppSettings::default(),
        )),
        Arc::clone(&storage),
        None,
    )
    .expect("build_app_graph_for_tests should succeed");
    crate::adapters::driving::http::AppState::from_wired(wired)
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
    assert!(response.0.contains("Set Active (Novel)</button>"));
    assert!(response.0.contains("Set Active (IF)</button>"));
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

/// A preset's type is fixed at creation: update takes it from the stored row
/// and ignores the form's hidden `preset_type` input, whatever it carries.
#[tokio::test]
async fn test_update_preset_ignores_form_preset_type() {
    let preset = crate::test_support::TestPromptPreset::system("custom", "Custom");
    let app_state = make_test_app_state_with_preset(preset.clone());
    let response = update_preset_handler(
        axum::extract::State(app_state.clone()),
        axum::extract::Path("custom".to_string()),
        axum::extract::Form(PresetForm {
            name: "Updated".into(),
            instructions: Some("Updated.".into()),
            preset_type: "quantifier".into(),
            ..Default::default()
        }),
    )
    .await;
    assert!(
        !response.0.contains("error"),
        "update must succeed: {}",
        response.0
    );

    let stored = app_state
        .prompt_preset_service
        .get_preset("custom")
        .unwrap()
        .expect("preset still present");
    assert_eq!(stored.preset_type, preset.preset_type);
    assert_eq!(stored.name, "Updated");
}

#[tokio::test]
async fn test_activate_preset_does_not_update_memory_when_save_fails() {
    let app_state = make_test_app_state_with_failing_storage(
        PromptPreset {
            id: "custom-system".into(),
            name: "Custom System".into(),
            instructions: Some("Custom.".into()),
            preset_type: PresetType::System,
            ..Default::default()
        },
        |h| {
            h.set(
                "save_settings",
                TestOverride::config("injected save failure"),
            )
        },
    );

    let response = activate_preset_handler(
        axum::extract::State(app_state.clone()),
        axum::extract::Path("custom-system".to_string()),
        axum::extract::Query(ActivateQuery::default()),
    )
    .await;
    assert!(response.0.contains("Save failed"));

    let active_id = app_state
        .settings
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .mode_preset_registry
        .bundle_for(NarratorMode::Novel)
        .system_prompt_preset_id
        .clone();
    assert_eq!(
        active_id, "system_default",
        "in-memory active preset id must not change when persistence fails"
    );
}

#[tokio::test]
async fn test_activate_nonexistent_preset_returns_error() {
    let app_state =
        make_test_app_state_with_preset(crate::test_support::TestPromptPreset::system("x", "X"));
    let response = activate_preset_handler(
        axum::extract::State(app_state),
        axum::extract::Path("missing".to_string()),
        axum::extract::Query(ActivateQuery::default()),
    )
    .await;
    assert!(response.0.contains("Preset not found"));
}

#[tokio::test]
async fn test_panel_handler_with_poisoned_settings_lock() {
    let app_state =
        make_test_app_state_with_preset(crate::test_support::TestPromptPreset::system("x", "X"));

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

    let wired = crate::bootstrap::wiring::build_app_graph_for_tests(
        Arc::new(RwLock::new(
            crate::domain::model::settings::AppSettings::default(),
        )),
        Arc::new(storage),
        None,
    )
    .expect("build_app_graph_for_tests should succeed");
    crate::adapters::driving::http::AppState::from_wired(wired)
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
            allowed_mode_novel: true,
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

fn preset_with_modes(id: &str, modes: Vec<NarratorMode>) -> PromptPreset {
    let mut preset = crate::test_support::TestPromptPreset::system(id, id);
    preset.allowed_modes = modes;
    preset
}

#[tokio::test]
async fn test_activate_writes_novel_slot_for_allowed_preset() {
    let preset = preset_with_modes("custom-novel", vec![NarratorMode::Novel]);
    let app_state = make_test_app_state_with_preset(preset);

    let response = activate_preset_handler(
        axum::extract::State(app_state.clone()),
        axum::extract::Path("custom-novel".to_string()),
        axum::extract::Query(ActivateQuery::default()),
    )
    .await;
    assert!(
        !response.0.contains("error"),
        "activation should succeed: {}",
        response.0
    );

    let settings = app_state.settings.read().unwrap_or_else(|e| e.into_inner());
    assert_eq!(
        settings
            .mode_preset_registry
            .bundle_for(NarratorMode::Novel)
            .system_prompt_preset_id,
        "custom-novel"
    );
    assert_eq!(
        settings
            .mode_preset_registry
            .bundle_for(NarratorMode::InteractiveFiction)
            .system_prompt_preset_id,
        "system_if_default",
        "IF bundle must stay untouched"
    );
}

#[tokio::test]
async fn test_activate_refuses_preset_not_allowed_for_mode() {
    let preset = preset_with_modes("if-only", vec![NarratorMode::InteractiveFiction]);
    let app_state = make_test_app_state_with_preset(preset);

    // No mode param resolves to Novel, which the preset does not allow.
    let response = activate_preset_handler(
        axum::extract::State(app_state.clone()),
        axum::extract::Path("if-only".to_string()),
        axum::extract::Query(ActivateQuery::default()),
    )
    .await;
    assert!(response.0.contains("not allowed"));

    let settings = app_state.settings.read().unwrap_or_else(|e| e.into_inner());
    assert_eq!(
        settings
            .mode_preset_registry
            .bundle_for(NarratorMode::Novel)
            .system_prompt_preset_id,
        "system_default",
        "refused activation must leave settings untouched"
    );
}

#[tokio::test]
async fn test_activate_with_mode_param_writes_that_modes_slot() {
    let app_state =
        make_test_app_state_with_preset(crate::test_support::TestPromptPreset::system("both", "B"));

    let response = activate_preset_handler(
        axum::extract::State(app_state.clone()),
        axum::extract::Path("both".to_string()),
        axum::extract::Query(ActivateQuery {
            mode: Some("interactive_fiction".to_string()),
        }),
    )
    .await;
    assert!(
        !response.0.contains("error"),
        "activation should succeed: {}",
        response.0
    );

    let settings = app_state.settings.read().unwrap_or_else(|e| e.into_inner());
    assert_eq!(
        settings
            .mode_preset_registry
            .bundle_for(NarratorMode::InteractiveFiction)
            .system_prompt_preset_id,
        "both"
    );
    assert_eq!(
        settings
            .mode_preset_registry
            .bundle_for(NarratorMode::Novel)
            .system_prompt_preset_id,
        "system_default",
        "novel slot must stay untouched"
    );
}

#[tokio::test]
async fn test_card_badges_active_quantifier_default_for_novel() {
    let mut preset = crate::test_support::TestPromptPreset::system("q-active", "Quant");
    preset.preset_type = PresetType::Quantifier;
    let app_state = make_test_app_state_with_preset(preset);

    let response = activate_preset_handler(
        axum::extract::State(app_state.clone()),
        axum::extract::Path("q-active".to_string()),
        axum::extract::Query(ActivateQuery::default()),
    )
    .await;
    assert!(
        !response.0.contains("error"),
        "activation should succeed: {}",
        response.0
    );

    // Refetch the card through the fragment endpoint: the Novel bundle's
    // quantifier slot holds this preset, so the card must badge it and
    // suppress the Set Active (Novel) action.
    let card = preset_card_handler(
        axum::extract::State(app_state),
        axum::extract::Path("q-active".to_string()),
    )
    .await;
    assert!(
        card.0.contains(r#"badge primary">Active · Novel</span>"#),
        "active quantifier default must badge Active · Novel: {}",
        card.0
    );
    assert!(
        !card.0.contains("Set Active (Novel)</button>"),
        "active quantifier default must not offer Set Active (Novel): {}",
        card.0
    );
}

#[tokio::test]
async fn test_card_badges_active_impersonate_default_for_if() {
    let mut preset = crate::test_support::TestPromptPreset::system("imp-active", "Imp");
    preset.preset_type = PresetType::Impersonate;
    let app_state = make_test_app_state_with_preset(preset);

    let response = activate_preset_handler(
        axum::extract::State(app_state.clone()),
        axum::extract::Path("imp-active".to_string()),
        axum::extract::Query(ActivateQuery {
            mode: Some("interactive_fiction".to_string()),
        }),
    )
    .await;
    assert!(
        !response.0.contains("error"),
        "activation should succeed: {}",
        response.0
    );

    let card = preset_card_handler(
        axum::extract::State(app_state),
        axum::extract::Path("imp-active".to_string()),
    )
    .await;
    assert!(
        card.0.contains(r#"badge primary">Active · IF</span>"#),
        "active impersonate default must badge Active · IF: {}",
        card.0
    );
    assert!(
        !card.0.contains(r#"badge primary">Active · Novel</span>"#),
        "IF-only activation must not badge Novel: {}",
        card.0
    );
    assert!(
        !card.0.contains("Set Active (IF)</button>"),
        "active impersonate default must not offer Set Active (IF): {}",
        card.0
    );
}

#[tokio::test]
async fn test_delete_refuses_preset_referenced_as_any_mode_default() {
    let app_state = make_test_app_state_with_preset(crate::test_support::TestPromptPreset::system(
        "custom-ref",
        "Custom Ref",
    ));

    // Reference the custom preset as the IF bundle's system default.
    {
        let mut settings = app_state.settings.write().unwrap();
        let mut bundle = settings
            .mode_preset_registry
            .bundle_for(NarratorMode::InteractiveFiction);
        bundle.system_prompt_preset_id = "custom-ref".to_string();
        settings.mode_preset_registry.set_bundle(bundle);
    }

    let response = delete_preset_handler(
        axum::extract::State(app_state.clone()),
        axum::extract::Path("custom-ref".to_string()),
    )
    .await;
    assert!(response.0.contains("mode default"));
    assert!(
        app_state
            .prompt_preset_service
            .get_preset("custom-ref")
            .unwrap()
            .is_some(),
        "referenced preset must not be deleted"
    );
}

#[tokio::test]
async fn test_update_preset_sets_allowed_modes_from_form() {
    let preset = TestPromptPreset::system("custom", "Custom");
    let app_state = make_test_app_state_with_preset(preset.clone());
    assert!(preset.allows_novel());

    let response = update_preset_handler(
        axum::extract::State(app_state.clone()),
        axum::extract::Path("custom".to_string()),
        axum::extract::Form(PresetForm {
            name: "Custom".into(),
            instructions: Some("Updated.".into()),
            preset_type: "system".into(),
            allowed_mode_if: true,
            ..Default::default()
        }),
    )
    .await;
    assert!(
        !response.0.contains("error"),
        "update should succeed: {}",
        response.0
    );

    let stored = app_state
        .prompt_preset_service
        .get_preset("custom")
        .unwrap()
        .expect("preset still present");
    assert!(!stored.allows_novel(), "novel flag must be cleared");
    assert!(stored.allows_interactive_fiction());
}

#[tokio::test]
async fn test_update_preset_preserves_flags_when_form_omits_them() {
    let preset = TestPromptPreset::system("custom", "Custom");
    let app_state = make_test_app_state_with_preset(preset);

    let response = update_preset_handler(
        axum::extract::State(app_state.clone()),
        axum::extract::Path("custom".to_string()),
        axum::extract::Form(PresetForm {
            name: "Custom".into(),
            preset_type: "system".into(),
            ..Default::default()
        }),
    )
    .await;
    assert!(
        !response.0.contains("error"),
        "update should succeed: {}",
        response.0
    );

    let stored = app_state
        .prompt_preset_service
        .get_preset("custom")
        .unwrap()
        .expect("preset still present");
    assert!(
        stored.allows_novel() && stored.allows_interactive_fiction(),
        "an update without mode flags must preserve the stored flags"
    );
}

#[tokio::test]
async fn test_save_preset_defaults_to_both_modes_when_form_omits_flags() {
    let app_state = make_test_app_state_with_preset(TestPromptPreset::system("x", "X"));

    let response = save_preset_handler(
        axum::extract::State(app_state.clone()),
        axum::extract::Form(PresetForm {
            name: "Created".into(),
            preset_type: "system".into(),
            ..Default::default()
        }),
    )
    .await;
    assert!(
        !response.0.contains("error"),
        "create should succeed: {}",
        response.0
    );

    let created = app_state
        .prompt_preset_service
        .list_presets(PresetType::System)
        .unwrap()
        .into_iter()
        .find(|p| p.name == "Created")
        .expect("created preset present");
    assert!(
        created.allows_novel() && created.allows_interactive_fiction(),
        "panel-created presets default to every mode"
    );
}

// The urlencoded checkbox grammar, pinned at the parse layer: a checked box
// posts `allowed_mode_<mode>=true`, an unchecked box posts nothing and
// serde-defaults to false. A `Vec<String>` checkbox group here surfaced as a
// browser-only 422.
#[test]
fn test_preset_form_urlencoded_parses_checked_flags() {
    let form: PresetForm = serde_urlencoded::from_str(
        "name=N&instructions=I&preset_type=system&allowed_mode_novel=true",
    )
    .expect("the posted body must parse");
    assert!(form.allowed_mode_novel);
    assert!(!form.allowed_mode_if);
}

#[test]
fn test_preset_form_urlencoded_defaults_flags_to_false() {
    let form: PresetForm = serde_urlencoded::from_str("name=N&instructions=I&preset_type=system")
        .expect("a body without flags must parse");
    assert!(!form.allowed_mode_novel);
    assert!(!form.allowed_mode_if);
}
