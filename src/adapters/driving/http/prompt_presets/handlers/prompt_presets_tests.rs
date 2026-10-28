//! Prompt preset HTTP handler tests.

use std::sync::Arc;

use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::NarratorMode;
use crate::adapters::driving::http::prompt_presets::handlers::{
    ActivateQuery, PresetForm, activate_preset_handler, delete_preset_handler,
    duplicate_preset_handler, edit_preset_form_handler, save_preset_handler, update_preset_handler,
    view_preset_form_handler,
};
use crate::adapters::driven::storage::{Storage, StorageCall, TestOverride};
use crate::test_support::TestPromptPreset;
use crate::test_support::{assert_answers_nothing, body_text};

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

    let wired =
        crate::bootstrap::wiring::build_app_graph_for_tests(Arc::clone(&storage), None, None)
            .expect("build_app_graph_for_tests should succeed");
    crate::adapters::driving::http::AppState::from_wired(wired)
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
    let body = body_text(response).await;
    assert!(body.contains("View Default"));
    assert!(body.contains("System prompt."));
    assert!(body.contains("disabled"));
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
    let body = body_text(response).await;
    assert!(body.contains("Preset not found"));
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
        axum::extract::State(app_state.clone()),
        axum::extract::Path("orig".to_string()),
    )
    .await;
    assert_answers_nothing(response).await;
    let names: Vec<String> = app_state
        .prompt_preset_service
        .list_presets(PresetType::System)
        .expect("presets list")
        .into_iter()
        .map(|preset| preset.name)
        .collect();
    assert!(
        names.contains(&"Original (Copy)".to_string()),
        "the copy is stored: {names:?}"
    );
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
    let body = body_text(response).await;
    assert!(body.contains("Preset not found"));
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
                StorageCall::SavePreset,
                TestOverride::config("injected save failure"),
            )
        },
    );
    let response = duplicate_preset_handler(
        axum::extract::State(app_state),
        axum::extract::Path("orig".to_string()),
    )
    .await;
    let body = body_text(response).await;
    assert!(body.contains("injected save failure"), "{body}");
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
    let body = body_text(response).await;
    assert!(body.contains("Cannot edit default presets"));
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
    let body = body_text(response).await;
    assert!(body.contains("Cannot edit default presets"));
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
    let body = body_text(response).await;
    assert!(body.contains("Cannot delete default presets"));
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
    let body = body_text(response).await;
    assert!(body.contains("Invalid preset type"));
}

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
    let body = body_text(response).await;
    assert!(
        !body.contains("error-message"),
        "update must succeed: {body}"
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
                StorageCall::UpdateSettings,
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
    let body = body_text(response).await;
    assert!(body.contains("injected save failure"), "{body}");

    let active_id = app_state
        .settings()
        .expect("settings read should succeed")
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
    let body = body_text(response).await;
    assert!(body.contains("Preset not found"));
}

fn make_test_app_state_with_failing_storage(
    preset: PromptPreset,
    fail_after_setup: impl FnOnce(&crate::adapters::driven::storage::TestFailureHandle),
) -> crate::adapters::driving::http::AppState {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    let _ = storage.save_preset(&preset);
    fail_after_setup(&handle);

    let storage = Arc::new(storage);
    let wired =
        crate::bootstrap::wiring::build_app_graph_for_tests(Arc::clone(&storage), None, None)
            .expect("build_app_graph_for_tests should succeed");
    crate::adapters::driving::http::AppState::from_wired(wired)
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
                StorageCall::GetPreset,
                TestOverride::config("injected get failure"),
            )
        },
    );
    let response = edit_preset_form_handler(
        axum::extract::State(app_state),
        axum::extract::Path("custom".to_string()),
    )
    .await;
    let body = body_text(response).await;
    assert!(body.contains("Load failed"));
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
    let body = body_text(response).await;
    assert!(
        !body.contains("error-message"),
        "activation should succeed: {body}"
    );

    let settings = app_state.settings().expect("settings read should succeed");
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

    let response = activate_preset_handler(
        axum::extract::State(app_state.clone()),
        axum::extract::Path("if-only".to_string()),
        axum::extract::Query(ActivateQuery::default()),
    )
    .await;
    let body = body_text(response).await;
    assert!(body.contains("not allowed"));

    let settings = app_state.settings().expect("settings read should succeed");
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
    let body = body_text(response).await;
    assert!(
        !body.contains("error-message"),
        "activation should succeed: {body}"
    );

    let settings = app_state.settings().expect("settings read should succeed");
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
async fn test_delete_refuses_preset_referenced_as_any_mode_default() {
    let app_state = make_test_app_state_with_preset(crate::test_support::TestPromptPreset::system(
        "custom-ref",
        "Custom Ref",
    ));

    {
        app_state
            .settings_service
            .update_settings(|settings| {
                let mut bundle = settings
                    .mode_preset_registry
                    .bundle_for(NarratorMode::InteractiveFiction);
                bundle.system_prompt_preset_id = "custom-ref".to_string();
                settings.mode_preset_registry.set_bundle(bundle);
                Ok(())
            })
            .expect("update_settings should succeed");
    }

    let response = delete_preset_handler(
        axum::extract::State(app_state.clone()),
        axum::extract::Path("custom-ref".to_string()),
    )
    .await;
    let body = body_text(response).await;
    assert!(body.contains("mode default"));
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
    let body = body_text(response).await;
    assert!(
        !body.contains("error-message"),
        "update should succeed: {body}"
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
    let body = body_text(response).await;
    assert!(
        !body.contains("error-message"),
        "update should succeed: {body}"
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
    let body = body_text(response).await;
    assert!(
        !body.contains("error-message"),
        "create should succeed: {body}"
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

// A `Vec<String>` checkbox group here surfaced as a browser-only 422, so the
// grammar is pinned at the parse layer.
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
