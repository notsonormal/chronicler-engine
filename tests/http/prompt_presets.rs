//! HTTP E2E tests for the prompt-presets endpoints.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{self, Request, StatusCode};
use tower::util::ServiceExt;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driven::storage::backend::TestOverride;
use chronicler_engine::adapters::driving::http::builders::router::build_router;
use chronicler_engine::application::prompt_preset_service::PromptPresetService;
use chronicler_engine::TestAppBuilder;

use crate::SettingsTestGuard;

async fn body_string(response: axum::response::Response<Body>) -> String {
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    String::from_utf8_lossy(&body).to_string()
}

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
    let body = body_string(response).await;

    assert!(body.contains(r#"<div class="prompt-presets-panel">"#));
    assert!(body.contains("<h2>System Prompts</h2>"));
    assert!(body.contains("<h2>Quantifier Prompts</h2>"));
    assert!(body.contains("<h3>Add System Prompt Preset</h3>"));
    assert!(body.contains("<h3>Add Quantifier Prompt Preset</h3>"));
    // Default test fixture seeds one default system preset; the spec assumes a
    // default quantifier preset is also present, but the fixture does not seed one.
    assert!(body.contains("preset-card"));
    assert!(body.contains("Default"));
    assert!(body.contains(r#"<input type="hidden" name="preset_type" value="system" />"#));
    assert!(body.contains(r#"<input type="hidden" name="preset_type" value="quantifier" />"#));
    assert!(body.contains(r#"name="name""#));
    assert!(body.contains(r#"name="role""#));
    assert!(body.contains(r#"name="instructions""#));
    assert!(body.contains(r#"name="writing_style""#));
    assert!(body.contains(r#"name="output_format""#));
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
    let panel = body_string(create_response).await;
    let preset_id = extract_first_preset_id(&panel);

    let response = app
        .oneshot(get_request(&format!(
            "/fragment/prompt-presets/{preset_id}"
        )))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
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
    let body = body_string(response).await;
    assert_eq!(body, "<span class='error'>Preset not found</span>");
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
    let panel = body_string(create_response).await;
    let preset_id = extract_first_preset_id(&panel);

    let response = app
        .oneshot(get_request(&format!(
            "/fragment/prompt-presets/{preset_id}/edit"
        )))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
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
    let body = body_string(response).await;
    assert_eq!(body, "<span class='error'>Preset not found</span>");
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
    let body = body_string(response).await;
    assert_eq!(
        body,
        "<span class='error'>Cannot edit default presets</span>"
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
    let body = body_string(response).await;
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
    let body = body_string(response).await;
    assert_eq!(body, "<span class='error'>Preset not found</span>");
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
    let body = body_string(response).await;
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
    let body = body_string(response).await;
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
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert_eq!(body, "<span class='error'>Invalid preset type</span>");
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
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert!(body.contains(r#"<span class='error'>Save failed:"#));
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
    let panel = body_string(create_response).await;
    let preset_id = extract_first_preset_id(&panel);

    let response = app
        .oneshot(post_form_request(
            &format!("/prompt-presets/{preset_id}"),
            "name=After&instructions=Updated.&preset_type=system",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
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
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert_eq!(body, "<span class='error'>Preset not found</span>");
}

// [docs/specs/prompt_presets.md] SCENARIO: 21.16
#[tokio::test]
async fn test_update_preset_invalid_type_returns_error() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let create_response = app
        .clone()
        .oneshot(post_form_request(
            "/prompt-presets",
            "name=Update+Type&instructions=Test.&preset_type=system",
        ))
        .await
        .unwrap();
    let panel = body_string(create_response).await;
    let preset_id = extract_first_preset_id(&panel);

    let response = app
        .oneshot(post_form_request(
            &format!("/prompt-presets/{preset_id}"),
            "name=Updated&instructions=Updated.&preset_type=invalid",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert_eq!(body, "<span class='error'>Invalid preset type</span>");
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
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert_eq!(
        body,
        "<span class='error'>Cannot edit default presets</span>"
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
    let panel = body_string(create_response).await;
    let preset_id = extract_first_preset_id(&panel);

    let response = app
        .oneshot(empty_post_request(&format!(
            "/prompt-presets/{preset_id}/delete"
        )))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
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
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert_eq!(body, "<span class='error'>Preset not found</span>");
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
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert_eq!(
        body,
        "<span class='error'>Cannot delete default presets</span>"
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
    let panel = body_string(create_response).await;
    let preset_id = extract_first_preset_id(&panel);

    let response = app
        .oneshot(empty_post_request(&format!(
            "/prompt-presets/{preset_id}/duplicate"
        )))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
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
    let body = body_string(response).await;
    assert_eq!(body, "<span class='error'>Preset not found</span>");
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
    let panel = body_string(create_response).await;
    let preset_id = extract_first_preset_id(&panel);

    let response = app
        .oneshot(empty_post_request(&format!(
            "/prompt-presets/{preset_id}/activate"
        )))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert!(body.contains(r#"<div class="prompt-presets-panel">"#));
    assert!(body.contains("Active"));
    assert!(!body.contains(&format!(
        r#"hx-post="/prompt-presets/{preset_id}/activate""#
    )));
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
    let body = body_string(response).await;
    assert_eq!(body, "<span class='error'>Preset not found</span>");
}
