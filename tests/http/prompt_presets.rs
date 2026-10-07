//! HTTP E2E tests for the prompt-presets endpoints.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{self, Request, StatusCode};
use tower::util::ServiceExt;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driven::storage::TestOverride;
use chronicler_engine::adapters::driving::http::builders::router::build_router;
use chronicler_engine::application::prompt_preset_service::PromptPresetService;
use chronicler_engine::test_support::body_text;
use chronicler_engine::test_support::TestPromptPreset;
use chronicler_engine::TestAppBuilder;

use crate::test_utils::preset_card_html_slice;
use crate::test_utils::panel_section_html_slice;

use crate::SettingsTestGuard;

fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .method(http::Method::GET)
        .body(Body::empty())
        .unwrap()
}

fn post_form_request(uri: &str, body: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn empty_post_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap()
}

/// The id of the single preset with `name`, read from storage rather than
/// scraped from rendered HTML — the panel is a text blob and a name match
/// against it can hit a truncated or escaped form.
fn preset_id_by_name(storage: &Storage, name: &str) -> String {
    use chronicler_engine::domain::model::prompt_preset::PresetType;
    for preset_type in [
        PresetType::System,
        PresetType::Quantifier,
        PresetType::Impersonate,
    ] {
        for preset in storage.list_presets(preset_type).unwrap() {
            if preset.name == name {
                return preset.id;
            }
        }
    }
    panic!("no preset named {name:?} in storage");
}

fn extract_first_preset_id(body: &str) -> String {
    let delete_marker = "/delete\"";
    let delete_pos = body
        .find(delete_marker)
        .expect("preset delete button in response");
    let url_prefix = "/prompt-presets/";
    let search_area = &body[..delete_pos];
    let url_pos = search_area
        .rfind(url_prefix)
        .expect("preset URL before delete");
    let id_start = url_pos + url_prefix.len();
    body[id_start..delete_pos].to_string()
}

/// HTML slice covering one preset's card. Cards carry no data-id, so the
/// slice is located via the always-rendered duplicate URL and bounded by the
/// next card (or the end of the body).
fn preset_card_slice<'a>(body: &'a str, preset_id: &str) -> &'a str {
    let anchor = format!(r#"hx-post="/prompt-presets/{preset_id}/duplicate""#);
    preset_card_html_slice(body, &anchor)
        .unwrap_or_else(|| panic!("preset card for {preset_id} missing: no duplicate button"))
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.1
#[tokio::test]
async fn test_prompt_presets_panel_renders_full_surface() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(get_request("/fragment/prompt-presets"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;

    assert!(body.contains(r#"<div class="prompt-presets-panel">"#));
    assert!(body.contains("<h2>System Prompts</h2>"));
    assert!(body.contains("<h2>Quantifier Prompts</h2>"));
    assert!(body.contains("preset-add-toggle"));
    assert!(body.contains("Add System Prompt Preset"));
    assert!(body.contains("Add Quantifier Prompt Preset"));
    // Default test fixture seeds one default system preset; the spec assumes a
    // default quantifier preset is also present, but the fixture does not seed one.
    assert!(body.contains("preset-card"));
    assert!(body.contains("Default"));
    assert!(body.contains(r#"<input type="hidden" name="preset_type" value="system" />"#));
    assert!(body.contains(r#"<input type="hidden" name="preset_type" value="quantifier" />"#));
    assert!(body.contains("<h2>Impersonate Prompts</h2>"));
    assert!(body.contains("Add Impersonate Prompt Preset"));
    let impersonate_marker = r#"<input type="hidden" name="preset_type" value="impersonate" />"#;
    let impersonate_start = body
        .find(impersonate_marker)
        .expect("impersonate add-form must render");
    let impersonate_form = &body[impersonate_start..];
    let impersonate_form = &impersonate_form[..impersonate_form
        .find("</form>")
        .expect("impersonate add-form must close")];
    for field in [
        "name",
        "role",
        "instructions",
        "writing_style",
        "output_format",
    ] {
        assert!(
            impersonate_form.contains(&format!(r#"name="{field}""#)),
            "impersonate add-form must contain input `{field}`: {impersonate_form}"
        );
    }
    assert!(
        body.contains(r#"data-error-slot="preset-add-system""#)
            && body.contains(r#"data-error-slot="preset-add-options""#),
        "each Add form must carry the inline error slot the client renders into"
    );
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.28
#[tokio::test]
async fn test_prompt_presets_add_forms_collapsed_by_default() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(get_request("/fragment/prompt-presets"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;

    assert_eq!(
        body.matches(r#"<details class="preset-add">"#).count(),
        4,
        "each category's add form must be a closed disclosure"
    );
    assert!(
        !body.contains(r#"<details class="preset-add" open>"#),
        "the add forms must be closed by default"
    );
    assert!(body.contains("Add System Prompt Preset"));
    assert!(body.contains("Add Quantifier Prompt Preset"));
    assert!(body.contains("Add Impersonate Prompt Preset"));
    assert!(body.contains("Add Options Prompt Preset"));
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.2
#[tokio::test]
async fn test_prompt_preset_single_card_renders() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let create_response = app
        .clone()
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=My+System+Prompt&instructions=You+are+a+test+narrator.&preset_type=system",
        ))
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);
    let panel = body_text(create_response).await;
    let preset_id = extract_first_preset_id(&panel);

    let response = app
        .oneshot(get_request(&format!(
            "/fragment/prompt-presets/{preset_id}"
        )))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains(r#"<div class="preset-card""#));
    assert!(body.contains("My System Prompt"));
    assert!(body.contains("Set Active"));
    assert!(body.contains("Edit"));
    assert!(body.contains("Delete"));
    assert!(body.contains("Duplicate"));
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.3
#[tokio::test]
async fn test_prompt_preset_single_card_missing_returns_error() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(get_request("/fragment/prompt-presets/does-not-exist"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert_eq!(body, "<div class=\"error-message\">Preset not found</div>");
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.4
#[tokio::test]
async fn test_prompt_preset_edit_form_renders_for_non_default() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let create_response = app
        .clone()
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=Editable&instructions=Edit+me.&preset_type=system",
        ))
        .await
        .unwrap();
    let panel = body_text(create_response).await;
    let preset_id = extract_first_preset_id(&panel);

    let response = app
        .oneshot(get_request(&format!(
            "/fragment/prompt-presets/{preset_id}/edit"
        )))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains(r#"<div class="preset-card edit-form">"#));
    assert!(body.contains(&format!(r#"hx-post="/prompt-presets/{preset_id}""#)));
    assert!(body.contains(r#"<input type="hidden" name="preset_type" value="system" />"#));
    assert!(body.contains(r#"value="Editable""#));
    assert!(body.contains(r#"name="role""#));
    assert!(body.contains(r#"name="instructions""#));
    assert!(body.contains(r#"name="writing_style""#));
    assert!(body.contains(r#"name="output_format""#));
    assert!(body.contains("Save"));
    assert!(body.contains("Cancel"));
    assert!(
        body.contains(&format!(r#"data-error-slot="preset-edit-{preset_id}""#)),
        "the edit form must carry the inline error slot the client renders into"
    );
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.5
#[tokio::test]
async fn test_prompt_preset_edit_form_missing_returns_error() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(get_request("/fragment/prompt-presets/does-not-exist/edit"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert_eq!(body, "<div class=\"error-message\">Preset not found</div>");
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.6
#[tokio::test]
async fn test_prompt_preset_edit_form_default_returns_error() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(get_request("/fragment/prompt-presets/system_default/edit"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert_eq!(
        body,
        "<div class=\"error-message\">Cannot edit default presets</div>"
    );
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.7
#[tokio::test]
async fn test_prompt_preset_view_form_renders_for_default() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(get_request("/fragment/prompt-presets/system_default/view"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains(r#"<div class="preset-card view-form">"#));
    assert!(body.contains("Default Test System"));
    assert!(body.contains("Role"));
    assert!(body.contains("Instructions"));
    assert!(body.contains("Writing Style"));
    assert!(body.contains("Output Format"));
    assert!(body.contains("Close"));
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.8
#[tokio::test]
async fn test_prompt_preset_view_form_missing_returns_error() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(get_request("/fragment/prompt-presets/does-not-exist/view"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert_eq!(body, "<div class=\"error-message\">Preset not found</div>");
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.9
#[tokio::test]
async fn test_create_system_preset_renders_panel() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=My+System+Prompt&instructions=You+are+a+test+narrator.&preset_type=system",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains(r#"<div class="prompt-presets-panel">"#));
    assert!(body.contains("My System Prompt"));
    assert!(body.contains("You are a test narrator."));
    assert!(body.contains("Set Active"));
    assert!(body.contains("Edit"));
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.10
#[tokio::test]
async fn test_create_quantifier_preset_renders_panel() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=My+Quantifier+Prompt&instructions=Quantify+this+scene.&preset_type=quantifier",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains(r#"<div class="prompt-presets-panel">"#));
    assert!(body.contains("My Quantifier Prompt"));
    assert!(body.contains("Quantify this scene."));
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.11
#[tokio::test]
async fn test_create_preset_invalid_type_returns_error() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=Bad+Type&instructions=Test.&preset_type=invalid",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_text(response).await;
    assert_eq!(
        body,
        "<div class=\"error-message\">Invalid preset type</div>"
    );
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.12
#[tokio::test]
async fn test_create_preset_missing_type_returns_422() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=Missing+Type&instructions=Test.",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.13
#[tokio::test]
async fn test_create_preset_reports_save_failure() {
    let _guard = SettingsTestGuard::new();
    let mut app_state = TestAppBuilder::default_test().build_service();
    app_state.prompt_preset_service = PromptPresetService::new(Arc::new(
        Storage::new_in_memory()
            .with_failure("save_preset", TestOverride::internal("preset save failure")),
    ));
    let app = build_router(app_state);

    let response = app
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=Fail+Preset&instructions=Will+fail.&preset_type=system",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = body_text(response).await;
    assert!(body.contains(r#"<div class="error-message">Save failed:"#));
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.14
#[tokio::test]
async fn test_update_preset_renders_card_with_new_name() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let create_response = app
        .clone()
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=Before&instructions=Original.&preset_type=system",
        ))
        .await
        .unwrap();
    let panel = body_text(create_response).await;
    let preset_id = extract_first_preset_id(&panel);

    let response = app
        .oneshot(post_form_request(
            &format!("/prompt-presets/{preset_id}"),
            "name=After&instructions=Updated.&preset_type=system",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains(r#"<div class="preset-card""#));
    assert!(body.contains("After"));
    assert!(!body.contains("Before"));
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.15
#[tokio::test]
async fn test_update_missing_preset_returns_error() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(post_form_request(
            "/prompt-presets/does-not-exist",
            "name=Updated&instructions=Updated.&preset_type=system",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_text(response).await;
    assert_eq!(body, "<div class=\"error-message\">Preset not found</div>");
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.16
#[tokio::test]
async fn test_update_preset_ignores_form_preset_type() {
    let _guard = SettingsTestGuard::new();
    let app_state = TestAppBuilder::default_test().build_service();
    let app = build_router(app_state.clone());

    let create_response = app
        .clone()
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=Update+Type&instructions=Test.&preset_type=system",
        ))
        .await
        .unwrap();
    let panel = body_text(create_response).await;
    let preset_id = extract_first_preset_id(&panel);

    let response = app
        .oneshot(post_form_request(
            &format!("/prompt-presets/{preset_id}"),
            "name=Updated&instructions=Updated.&preset_type=quantifier",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(
        !body.contains("error-message"),
        "update must succeed: {body}"
    );
    assert!(body.contains("preset-card"));

    let stored = app_state
        .prompt_preset_service
        .get_preset(&preset_id)
        .unwrap()
        .expect("preset still present");
    assert_eq!(stored.preset_type.as_str(), "system");
    assert_eq!(stored.name, "Updated");
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.17
#[tokio::test]
async fn test_update_default_preset_returns_error() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(post_form_request(
            "/prompt-presets/system_default",
            "name=Changed&instructions=Changed.&preset_type=system",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_text(response).await;
    assert_eq!(
        body,
        "<div class=\"error-message\">Cannot edit default presets</div>"
    );
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.18
#[tokio::test]
async fn test_delete_non_default_preset_returns_empty_body() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let create_response = app
        .clone()
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=Delete+Me&instructions=Delete.&preset_type=system",
        ))
        .await
        .unwrap();
    let panel = body_text(create_response).await;
    let preset_id = extract_first_preset_id(&panel);

    let response = app
        .oneshot(empty_post_request(&format!(
            "/prompt-presets/{preset_id}/delete"
        )))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.is_empty());
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.19
#[tokio::test]
async fn test_delete_missing_preset_returns_error() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(empty_post_request("/prompt-presets/does-not-exist/delete"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_text(response).await;
    assert_eq!(body, "<div class=\"error-message\">Preset not found</div>");
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.20
#[tokio::test]
async fn test_delete_default_preset_returns_error() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(empty_post_request("/prompt-presets/system_default/delete"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_text(response).await;
    assert_eq!(
        body,
        "<div class=\"error-message\">Cannot delete default presets</div>"
    );
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.37
#[tokio::test]
async fn test_delete_refuses_a_preset_referenced_as_a_mode_default() {
    use chronicler_engine::domain::model::settings::NarratorMode;

    let _guard = SettingsTestGuard::new();
    let storage = Arc::new(Storage::new_in_memory());
    storage
        .save_preset(&TestPromptPreset::system("custom_ref", "Custom Ref"))
        .expect("seed a system preset");
    let app_state = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_service();

    app_state
        .settings_service
        .update_settings(|settings| {
            let mut bundle = settings
                .mode_preset_registry
                .bundle_for(NarratorMode::InteractiveFiction);
            bundle.system_prompt_preset_id = "custom_ref".to_string();
            settings.mode_preset_registry.set_bundle(bundle);
            Ok(())
        })
        .expect("update_settings should succeed");
    let app = build_router(app_state.clone());

    let response = app
        .oneshot(empty_post_request("/prompt-presets/custom_ref/delete"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_text(response).await;
    assert!(
        body.contains("Preset is a mode default; change the default before deleting"),
        "the refusal must name the mode default: {body}"
    );
    assert!(
        storage.get_preset("custom_ref").unwrap().is_some(),
        "a referenced preset must not be deleted"
    );
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.38
#[tokio::test]
async fn test_delete_refuses_the_active_options_default() {
    let _guard = SettingsTestGuard::new();
    let storage = Arc::new(Storage::new_in_memory());
    storage
        .save_preset(&TestPromptPreset::options(
            "options_custom",
            "Custom Options",
        ))
        .expect("seed an options preset");
    let app_state = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_service();
    let app = build_router(app_state.clone());

    let activate = app
        .clone()
        .oneshot(empty_post_request(
            "/prompt-presets/options_custom/activate",
        ))
        .await
        .unwrap();
    assert_eq!(activate.status(), StatusCode::OK);

    let response = app
        .oneshot(empty_post_request("/prompt-presets/options_custom/delete"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_text(response).await;
    assert!(
        body.contains("Preset is the default Options preset; change the default before deleting"),
        "the refusal must name the Options default: {body}"
    );
    assert!(
        storage.get_preset("options_custom").unwrap().is_some(),
        "the active Options preset must not be deleted"
    );
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.21
#[tokio::test]
async fn test_duplicate_preset_renders_panel_with_copy() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let create_response = app
        .clone()
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=Original&instructions=Original.&preset_type=system",
        ))
        .await
        .unwrap();
    let panel = body_text(create_response).await;
    let preset_id = extract_first_preset_id(&panel);

    let response = app
        .oneshot(empty_post_request(&format!(
            "/prompt-presets/{preset_id}/duplicate"
        )))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains(r#"<div class="prompt-presets-panel">"#));
    assert!(body.contains("Original (Copy)"));
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.22
#[tokio::test]
async fn test_duplicate_missing_preset_returns_error() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(empty_post_request(
            "/prompt-presets/does-not-exist/duplicate",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert_eq!(body, "<div class=\"error-message\">Preset not found</div>");
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.23
#[tokio::test]
async fn test_activate_system_preset_renders_active_badge() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let create_response = app
        .clone()
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=Activate+Me&instructions=Activate.&preset_type=system",
        ))
        .await
        .unwrap();
    let panel = body_text(create_response).await;
    let preset_id = extract_first_preset_id(&panel);

    let response = app
        .oneshot(empty_post_request(&format!(
            "/prompt-presets/{preset_id}/activate"
        )))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(body.contains(r#"<div class="prompt-presets-panel">"#));
    // Scope both assertions to the activated preset's card: the panel holds
    // several presets, and a panel-wide check can pass or fail on an
    // unrelated preset's badge or buttons.
    let card = preset_card_slice(&body, &preset_id);
    assert!(
        card.contains("Active · Novel"),
        "activated preset's card must carry the Active · Novel badge"
    );
    assert!(
        !card.contains("Set Active (Novel)"),
        "activated preset's card must hide its Set Active (Novel) button once active"
    );
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.24
#[tokio::test]
async fn test_activate_missing_preset_returns_error() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .oneshot(empty_post_request(
            "/prompt-presets/does-not-exist/activate",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert_eq!(body, "<div class=\"error-message\">Preset not found</div>");
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.26
#[tokio::test]
async fn test_activate_refuses_preset_not_allowed_for_mode() {
    use chronicler_engine::domain::model::prompt_preset::{PresetType, PromptPreset};
    use chronicler_engine::domain::model::settings::NarratorMode;

    let _guard = SettingsTestGuard::new();
    let storage = Arc::new(Storage::new_in_memory());
    let preset = PromptPreset {
        id: "if-only".to_string(),
        name: "IF Only".to_string(),
        instructions: Some("IF.".to_string()),
        allowed_modes: vec![NarratorMode::InteractiveFiction],
        is_default: false,
        preset_type: PresetType::System,
        ..Default::default()
    };
    storage.save_preset(&preset).unwrap();

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    // No mode param resolves to Novel, which the preset does not allow.
    let response = app
        .oneshot(empty_post_request("/prompt-presets/if-only/activate"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert_eq!(
        body,
        "<div class=\"error-message\">Preset not allowed for novel mode</div>"
    );
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.25
#[tokio::test]
async fn test_activate_if_only_preset_via_if_mode_populates_if_bundle() {
    use chronicler_engine::domain::model::prompt_preset::{PresetType, PromptPreset};
    use chronicler_engine::domain::model::settings::NarratorMode;

    let _guard = SettingsTestGuard::new();
    let storage = Arc::new(Storage::new_in_memory());
    let preset = PromptPreset {
        id: "if-only".to_string(),
        name: "IF Only".to_string(),
        instructions: Some("IF.".to_string()),
        allowed_modes: vec![NarratorMode::InteractiveFiction],
        is_default: false,
        preset_type: PresetType::System,
        ..Default::default()
    };
    storage.save_preset(&preset).unwrap();

    let app_state = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_service();
    let app = build_router(app_state.clone());

    // Captured before activation: equality (not inequality) proves that IF
    // activation leaves the Novel bundle's system slot untouched.
    let novel_before = app_state
        .settings_service
        .get_settings()
        .unwrap()
        .mode_preset_registry
        .bundle_for(NarratorMode::Novel)
        .system_prompt_preset_id
        .clone();

    let response = app
        .clone()
        .oneshot(empty_post_request(
            "/prompt-presets/if-only/activate?mode=interactive_fiction",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(
        body.contains("Active · Interactive Fiction"),
        "IF activation must badge the preset Active · Interactive Fiction"
    );

    let settings = app_state.settings_service.get_settings().unwrap();
    let if_bundle = settings
        .mode_preset_registry
        .bundle_for(NarratorMode::InteractiveFiction);
    assert_eq!(
        if_bundle.system_prompt_preset_id, "if-only",
        "IF activation must write the IF bundle's system slot"
    );
    let novel_bundle = settings
        .mode_preset_registry
        .bundle_for(NarratorMode::Novel);
    assert_eq!(
        novel_bundle.system_prompt_preset_id, novel_before,
        "IF activation must not touch the Novel bundle"
    );
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.26
#[tokio::test]
async fn test_panel_gates_activation_buttons_by_allowed_modes() {
    use chronicler_engine::domain::model::prompt_preset::{PresetType, PromptPreset};
    use chronicler_engine::domain::model::settings::NarratorMode;

    let _guard = SettingsTestGuard::new();
    let storage = Arc::new(Storage::new_in_memory());
    let preset = PromptPreset {
        id: "if-only".to_string(),
        name: "IF Only".to_string(),
        instructions: Some("IF.".to_string()),
        allowed_modes: vec![NarratorMode::InteractiveFiction],
        is_default: false,
        preset_type: PresetType::System,
        ..Default::default()
    };
    storage.save_preset(&preset).unwrap();

    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let response = app
        .oneshot(get_request("/fragment/prompt-presets"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(
        body.contains(r#"hx-post="/prompt-presets/if-only/activate?mode=interactive_fiction""#),
        "IF-only preset must render an IF activation button"
    );
    assert!(
        !body.contains(r#"hx-post="/prompt-presets/if-only/activate?mode=novel""#),
        "IF-only preset must not render a Novel activation button"
    );
}

// The server-observable chain: create a preset, fetch its edit form (checking
// the rendered allowed-modes flags), then POST the update with the Interactive
// Fiction flag and assert the returned card exposes the IF activation button.
// The click handling between those hops is wiring, which no server response can
// observe.
// [docs/specs/prompt_presets.md] SCENARIO: 21.27
#[tokio::test]
async fn test_allowed_modes_duplicate_edit_save_chain_http() {
    let _guard = SettingsTestGuard::new();
    let storage = Arc::new(Storage::new_in_memory());
    let app_state = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_service();
    let app = build_router(app_state.clone());

    // 1. Seed the source with a fixed id. The chain under test is
    //    duplicate -> edit-form -> save, so the source's provenance is
    //    irrelevant.
    use chronicler_engine::domain::model::prompt_preset::{PresetType, PromptPreset};
    use chronicler_engine::domain::model::settings::NarratorMode;
    storage
        .save_preset(&PromptPreset {
            id: "chain-source".to_string(),
            name: "Chain".to_string(),
            instructions: Some("Chain.".to_string()),
            allowed_modes: vec![NarratorMode::Novel, NarratorMode::InteractiveFiction],
            is_default: false,
            preset_type: PresetType::System,
            ..Default::default()
        })
        .unwrap();
    let preset_id = "chain-source".to_string();

    // The copy carries the source's flags and is the only non-default card
    // with an edit form.
    let duplicated = app
        .clone()
        .oneshot(empty_post_request(&format!(
            "/prompt-presets/{preset_id}/duplicate"
        )))
        .await
        .unwrap();
    assert_eq!(duplicated.status(), StatusCode::OK);
    let copy_id = preset_id_by_name(&storage, "Chain (Copy)");
    assert_ne!(copy_id, preset_id, "the duplicate must be a new preset");

    // 2. The Edit hop: the form must render the stored allowed-modes flags.
    let edit_form = app
        .clone()
        .oneshot(get_request(&format!(
            "/fragment/prompt-presets/{copy_id}/edit"
        )))
        .await
        .unwrap();
    assert_eq!(edit_form.status(), StatusCode::OK);
    let form = body_text(edit_form).await;
    assert!(
        form.contains(r#"class="preset-card edit-form""#),
        "the edit form must render for the copy: {form}"
    );
    assert!(
        form.contains(r#"name="allowed_mode_novel" value="true" checked"#),
        "the stored Novel flag must render checked: {form}"
    );
    assert!(
        form.contains(r#"name="allowed_mode_if" value="true" checked"#),
        "the stored IF flag must render checked: {form}"
    );

    // 3. The save hop with the IF flag checked: the returned card must offer
    //    per-mode activation for both allowed modes.
    let saved = app
        .oneshot(post_form_request(
            &format!("/prompt-presets/{copy_id}"),
            "name=Chain+Copy&instructions=Chain.&preset_type=system\
             &allowed_mode_novel=true&allowed_mode_if=true",
        ))
        .await
        .unwrap();
    assert_eq!(saved.status(), StatusCode::OK);
    let card = body_text(saved).await;
    assert!(
        card.contains("Set Active (Interactive Fiction)"),
        "the saved card must offer Set Active (Interactive Fiction): {card}"
    );
    assert!(
        card.contains("Set Active (Novel)"),
        "the saved card must still offer Set Active (Novel): {card}"
    );

    let stored = app_state
        .prompt_preset_service
        .get_preset(&copy_id)
        .unwrap()
        .expect("the updated copy must persist");
    assert!(
        stored.allowed_modes.contains(&NarratorMode::Novel)
            && stored
                .allowed_modes
                .contains(&NarratorMode::InteractiveFiction),
        "both flags must persist: {:?}",
        stored.allowed_modes
    );
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.29
#[tokio::test]
async fn test_create_preset_duplicate_name_in_category_is_refused() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let first = app
        .clone()
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=Alpha&instructions=First.&preset_type=system",
        ))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);

    let second = app
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=Alpha&instructions=Second.&preset_type=system",
        ))
        .await
        .unwrap();
    assert_eq!(
        second.status(),
        StatusCode::BAD_REQUEST,
        "a duplicate preset name in the same category must be refused"
    );
    let body = body_text(second).await;
    assert!(body.contains("Alpha"), "body: {body}");
    assert!(body.contains("already exists"), "body: {body}");
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.30
#[tokio::test]
async fn test_create_preset_case_and_space_variant_is_refused() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let first = app
        .clone()
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=Alpha&instructions=First.&preset_type=system",
        ))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);

    let second = app
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=++alpha++&instructions=Second.&preset_type=system",
        ))
        .await
        .unwrap();
    assert_eq!(
        second.status(),
        StatusCode::BAD_REQUEST,
        "a case-and-space variant must be refused"
    );
    let body = body_text(second).await;
    assert!(body.contains("already exists"), "body: {body}");
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.31
#[tokio::test]
async fn test_create_preset_same_name_in_another_category_is_allowed() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let first = app
        .clone()
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=Alpha&instructions=First.&preset_type=system",
        ))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);

    let second = app
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=Alpha&instructions=Second.&preset_type=quantifier",
        ))
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::OK);
    let body = body_text(second).await;
    let quantifier_section = panel_section_html_slice(&body, "Quantifier Prompts")
        .expect("the panel must render a Quantifier section");
    assert!(
        quantifier_section.contains("<span class=\"card-title\">Alpha</span>"),
        "the Quantifier section must list the same-named preset: {quantifier_section}"
    );
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.32
#[tokio::test]
async fn test_update_preset_keeps_its_own_name() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let created = app
        .clone()
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=Alpha&instructions=First.&preset_type=system",
        ))
        .await
        .unwrap();
    let panel = body_text(created).await;
    let preset_id = extract_first_preset_id(&panel);

    let response = app
        .oneshot(post_form_request(
            &format!("/prompt-presets/{preset_id}"),
            "name=Alpha&instructions=Changed.&preset_type=system",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    let card = preset_card_html_slice(&body, "Alpha").expect("the updated card must render");
    assert!(
        card.contains("<span class=\"card-title\">Alpha</span>"),
        "the card must keep the preset's own name: {card}"
    );
    assert!(
        card.contains("Changed."),
        "the card must show the posted instructions: {card}"
    );
    assert!(
        !body.contains("error-message"),
        "keeping the own name must succeed: {body}"
    );
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.33
#[tokio::test]
async fn test_duplicate_same_preset_twice_yields_distinct_copies() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let created = app
        .clone()
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=Original&instructions=Original.&preset_type=system",
        ))
        .await
        .unwrap();
    let panel = body_text(created).await;
    let source_id = extract_first_preset_id(&panel);

    let first = app
        .clone()
        .oneshot(empty_post_request(&format!(
            "/prompt-presets/{source_id}/duplicate"
        )))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    let first_panel = body_text(first).await;
    assert!(
        first_panel.contains("Original (Copy)"),
        "first copy: {first_panel}"
    );

    let second = app
        .oneshot(empty_post_request(&format!(
            "/prompt-presets/{source_id}/duplicate"
        )))
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::OK);
    let second_panel = body_text(second).await;
    assert!(
        second_panel.contains("Original (Copy 2)"),
        "second copy must get the next free number: {second_panel}"
    );
    assert!(
        second_panel.contains("Original (Copy)"),
        "the first copy must remain: {second_panel}"
    );
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.34
#[tokio::test]
async fn test_update_preset_onto_sibling_name_is_refused() {
    let _guard = SettingsTestGuard::new();
    let storage = Arc::new(Storage::new_in_memory());
    let app_state = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_service();
    let app = build_router(app_state.clone());

    for name in ["Alpha", "Beta"] {
        let response = app
            .clone()
            .oneshot(post_form_request(
                "/prompt-presets",
                &format!("name={name}&instructions=First.&preset_type=system"),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    let beta_id = preset_id_by_name(&storage, "Beta");
    let response = app
        .oneshot(post_form_request(
            &format!("/prompt-presets/{beta_id}"),
            "name=Alpha&instructions=Changed.&preset_type=system",
        ))
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "renaming onto a sibling's name must be refused"
    );
    let body = body_text(response).await;
    assert!(body.contains("Alpha"), "body: {body}");
    assert!(body.contains("already exists"), "body: {body}");

    let stored = app_state
        .prompt_preset_service
        .get_preset(&beta_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.name, "Beta");
    assert_eq!(stored.instructions.as_deref(), Some("First."));
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.35
#[tokio::test]
async fn test_prompt_presets_panel_renders_options_section() {
    let _guard = SettingsTestGuard::new();
    let storage = Arc::new(Storage::new_in_memory());
    storage
        .save_preset(&TestPromptPreset::options(
            "options_custom",
            "Custom Options",
        ))
        .expect("seed an options preset");
    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();

    let response = app
        .oneshot(get_request("/fragment/prompt-presets"))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;

    assert!(body.contains("<h2>Options Prompts</h2>"));
    assert!(body.contains("Add Options Prompt Preset"));
    assert!(body.contains(r#"name="preset_type" value="options""#));
    assert!(body.contains("Custom Options"));

    let card = preset_card_slice(&body, "options_custom");
    assert!(
        card.contains("Set Active</button>"),
        "an Options card must offer a single Set Active button: {card}"
    );
    assert!(
        !card.contains("Set Active (Novel)") && !card.contains("Set Active (Interactive Fiction)"),
        "an Options card must not offer mode-specific activation: {card}"
    );
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.36
#[tokio::test]
async fn test_activate_options_preset_sets_the_settings_default() {
    let _guard = SettingsTestGuard::new();
    let storage = Arc::new(Storage::new_in_memory());
    storage
        .save_preset(&TestPromptPreset::options(
            "options_custom",
            "Custom Options",
        ))
        .expect("seed an options preset");
    let app_state = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_service();
    let app = build_router(app_state.clone());

    let response = app
        .oneshot(empty_post_request(
            "/prompt-presets/options_custom/activate",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;

    let card = preset_card_slice(&body, "options_custom");
    assert!(
        card.contains(r#"badge primary">Active"#),
        "the activated Options card must carry the Active badge: {card}"
    );
    assert!(
        !card.contains("Set Active</button>"),
        "an active Options preset must hide its Set Active button: {card}"
    );

    let settings = app_state.settings().expect("settings read should succeed");
    assert_eq!(settings.active_options_prompt_preset_id, "options_custom");
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.39
#[tokio::test]
async fn test_update_preset_storage_failure_answers_non_2xx_and_keeps_the_preset() {
    let _guard = SettingsTestGuard::new();
    let storage = Storage::new_in_memory();
    let (storage, failures) = storage.with_test_failures();
    storage
        .save_preset(&TestPromptPreset::system("original_ref", "Original"))
        .expect("seed a system preset");
    let app = TestAppBuilder::default_test()
        .storage(Arc::new(storage))
        .build_service();
    let app = build_router(app);

    failures.set("save_preset", TestOverride::internal("preset save failure"));

    let response = app
        .clone()
        .oneshot(post_form_request(
            "/prompt-presets/original_ref",
            "name=Changed&instructions=Updated.&preset_type=system",
        ))
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "a failed update must answer non-2xx, not a 200 card replacement"
    );
    let body = body_text(response).await;
    assert!(
        body.contains(r#"<div class="error-message">Update failed:"#),
        "the failure must reach the client for its inline slot: {body}"
    );
    assert!(
        !body.contains("preset-card"),
        "a failed update must not swap the card into the response: {body}"
    );

    failures.clear("save_preset");

    let panel = app
        .oneshot(get_request("/fragment/prompt-presets"))
        .await
        .unwrap();
    assert_eq!(panel.status(), StatusCode::OK);
    let panel_body = body_text(panel).await;
    assert!(
        panel_body.contains("Original"),
        "the stored preset must keep its original name: {panel_body}"
    );
}
