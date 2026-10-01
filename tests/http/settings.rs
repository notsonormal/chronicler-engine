//! HTTP E2E tests for the settings endpoints: panel rendering and POST /settings.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{self, Request, StatusCode};
use tower::util::ServiceExt;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driven::storage::TestOverride;
use chronicler_engine::domain::model::llm_backend::LlmBackendType;
use chronicler_engine::domain::model::settings::{AppSettings, LlmProviderConfig};
use chronicler_engine::TestAppBuilder;

use crate::SettingsTestGuard;
use crate::support::app_wiring::app_with_production_graph;

async fn body_string(response: axum::response::Response<Body>) -> String {
    let body = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    String::from_utf8_lossy(&body).to_string()
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

async fn get_body(app: &axum::Router, uri: &str) -> String {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    body_string(response).await
}

fn mock_connection(id: &str, model: &str) -> LlmProviderConfig {
    LlmProviderConfig {
        id: id.into(),
        name: id.into(),
        provider: LlmBackendType::Mock,
        model: model.into(),
        api_key: None,
        base_url: None,
        single_user_message: false,
        max_tokens: None,
        max_context_tokens: None,
    }
}

/// The id of the connection just added through the Settings panel. Added ids
/// start with `conn-`; the seeded fixture connections carry author-chosen ids.
fn added_connection_id(panel: &str) -> String {
    let prefix = "hx-get=\"/fragment/connections/";
    let marker = format!("{prefix}conn-");
    let start = panel.find(&marker).expect("added connection edit link") + prefix.len();
    let end = start
        + panel[start..]
            .find('/')
            .expect("connection edit link must close");
    panel[start..end].to_string()
}

/// The `<div class="connection-card">…</div>` slice for connection `id` in the
/// rendered settings panel. Cards carry no data-id, so the anchor is the
/// connection's own edit link and the slice ends at the next card.
fn connection_card<'a>(panel: &'a str, id: &str) -> &'a str {
    let anchor = format!(r#"hx-get="/fragment/connections/{id}/edit"#);
    let anchor_pos = panel
        .find(&anchor)
        .unwrap_or_else(|| panic!("no connection card for '{id}' in panel: {panel}"));
    let start = panel[..anchor_pos]
        .rfind(r#"<div class="connection-card">"#)
        .expect("connection card opening tag before its edit link");
    let end = panel[anchor_pos..]
        .find(r#"<div class="connection-card">"#)
        .map(|offset| anchor_pos + offset)
        .unwrap_or(panel.len());
    &panel[start..end]
}

// [docs/specs/settings.md] SCENARIO: 20.1
#[tokio::test]
async fn test_settings_panel_renders_full_surface() {
    let app = TestAppBuilder::default_app();

    let req = Request::builder()
        .uri("/fragment/settings")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert!(body.contains(r#"<div class="settings-panel">"#));
    assert!(body.contains("<h2>Connections</h2>"));
    assert!(body.contains("connection-card"));
    assert!(body.contains("<h3>Add Connection</h3>"));
    assert!(body.contains(r#"id="conn_name""#));
    assert!(body.contains(r#"name="conn_provider""#));
    assert!(body.contains("OpenRouter"));
    assert!(body.contains("DeepSeek"));
    assert!(body.contains("Ollama"));
    assert!(body.contains(r#"id="conn_model""#));
    assert!(body.contains(r#"id="conn_api_key""#));
    assert!(body.contains(r#"id="conn_base_url""#));
    assert!(body.contains(r#"name="single_user_message""#));
    assert!(body.contains("Single User Message"));
    assert!(body.contains("<h2>Text Check</h2>"));
    assert!(body.contains(r#"id="check_mode""#));
    assert!(body.contains(r#"name="enable_auto_check""#));
}

// [docs/specs/settings.md] SCENARIO: 20.2
#[tokio::test]
async fn test_post_settings_switches_narrator() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = post_form_request(
        "/settings",
        "narration_connection_id=openrouter-euryale&quantifier_connection_id=openrouter-gpt-4o-mini",
    );
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert_eq!(body, "Settings saved!");

    let panel = get_body(&app, "/fragment/settings").await;
    assert!(
        connection_card(&panel, "openrouter-euryale")
            .contains(r#"<span class="badge">Narrator</span>"#),
        "the switched narrator should render its badge: {panel}"
    );
    assert!(
        !connection_card(&panel, "openrouter-gpt-4o-mini")
            .contains(r#"<span class="badge">Narrator</span>"#),
        "the previous narrator should no longer render the Narrator badge: {panel}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.3
#[tokio::test]
async fn test_post_settings_switches_quantifier() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = post_form_request(
        "/settings",
        "narration_connection_id=openrouter-gpt-4o-mini&quantifier_connection_id=ollama-gemma-4-26B",
    );
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert_eq!(body, "Settings saved!");

    let panel = get_body(&app, "/fragment/settings").await;
    assert!(
        !connection_card(&panel, "openrouter-gpt-4o-mini")
            .contains(r#"<span class="badge quantifier">Quantifier</span>"#),
        "the previous quantifier should no longer render the Quantifier badge: {panel}"
    );
    assert!(
        connection_card(&panel, "ollama-gemma-4-26B")
            .contains(r#"<span class="badge quantifier">Quantifier</span>"#),
        "the switched quantifier should render its badge: {panel}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.4
#[tokio::test]
async fn test_post_settings_switches_both_connections() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = post_form_request(
        "/settings",
        "narration_connection_id=openrouter-euryale&quantifier_connection_id=ollama-gemma-4-26B",
    );
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert_eq!(body, "Settings saved!");

    let panel = get_body(&app, "/fragment/settings").await;
    assert!(
        connection_card(&panel, "openrouter-euryale")
            .contains(r#"<span class="badge">Narrator</span>"#),
        "the switched narrator should render its badge: {panel}"
    );
    assert!(
        connection_card(&panel, "ollama-gemma-4-26B")
            .contains(r#"<span class="badge quantifier">Quantifier</span>"#),
        "the switched quantifier should render its badge: {panel}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.5
#[tokio::test]
async fn test_post_settings_rejects_unknown_connection_id() {
    let _guard = SettingsTestGuard::new();
    let (app, app_state) = TestAppBuilder::default_test().build_with_state();

    let req = post_form_request(
        "/settings",
        "narration_connection_id=not-a-connection&quantifier_connection_id=openrouter-gpt-4o-mini",
    );
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert!(body.contains("error-message"));
    assert!(body.contains("is not in the connections list"));

    let stored = app_state.settings().expect("settings read should succeed");
    assert_eq!(stored.narration_connection_id, "openrouter-gpt-4o-mini");
}

// [docs/specs/settings.md] SCENARIO: 20.6
#[tokio::test]
async fn test_post_settings_missing_field_returns_422() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = post_form_request("/settings", "narration_connection_id=openrouter-euryale");
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// [docs/specs/settings.md] SCENARIO: 20.7
#[tokio::test]
async fn test_post_settings_reports_save_failure() {
    let _guard = SettingsTestGuard::new();
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        "update_settings",
        TestOverride::internal("settings save failure"),
    ));
    let app = TestAppBuilder::default_test().storage(storage).build();

    let req = post_form_request(
        "/settings",
        "narration_connection_id=openrouter-euryale&quantifier_connection_id=openrouter-gpt-4o-mini",
    );
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert!(body.contains("error-message"));
    assert!(body.contains("settings save failure"));
}

// [docs/specs/settings.md] SCENARIO: 20.8
#[tokio::test]
async fn test_narrator_switch_takes_effect_without_a_restart() {
    let _guard = SettingsTestGuard::new();

    let settings = AppSettings {
        connections: vec![
            mock_connection("mock-a", "mock-model-a"),
            mock_connection("mock-b", "mock-model-b"),
        ],
        narration_connection_id: "mock-a".into(),
        quantifier_connection_id: "mock-a".into(),
        ..AppSettings::default()
    };
    let (app, _state, _storage) = app_with_production_graph(settings);

    let body = get_body(&app, "/debug/backend").await;
    assert!(
        body.contains("mock-model-a"),
        "the seeded narrator should serve the first resolution: {body}"
    );

    let response = app
        .clone()
        .oneshot(post_form_request("/connections/mock-b/set-narrator", ""))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = get_body(&app, "/debug/backend").await;
    assert!(
        body.contains("mock-model-b"),
        "the switch should take effect on the next resolution: {body}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.9
#[tokio::test]
async fn test_connection_add_duplicate_name_is_refused() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let first = app
        .clone()
        .oneshot(post_form_request(
            "/connections/add",
            "conn_name=Duplicate+Probe&conn_provider=mock&conn_model=mock-model&conn_api_key=&conn_base_url=",
        ))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);

    let second = app
        .clone()
        .oneshot(post_form_request(
            "/connections/add",
            "conn_name=Duplicate+Probe&conn_provider=mock&conn_model=mock-model&conn_api_key=&conn_base_url=",
        ))
        .await
        .unwrap();
    assert_eq!(
        second.status(),
        StatusCode::BAD_REQUEST,
        "a duplicate connection name must be refused"
    );
    let body = body_string(second).await;
    assert!(body.contains("Duplicate Probe"), "body: {body}");
    assert!(body.contains("already exists"), "body: {body}");

    let panel = get_body(&app, "/fragment/settings").await;
    assert_eq!(
        panel.matches("Duplicate Probe").count(),
        1,
        "the refused connection must not be stored"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.10
#[tokio::test]
async fn test_connection_add_case_and_space_variant_is_refused() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let first = app
        .clone()
        .oneshot(post_form_request(
            "/connections/add",
            "conn_name=Alpha&conn_provider=mock&conn_model=mock-model&conn_api_key=&conn_base_url=",
        ))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);

    let second = app
        .clone()
        .oneshot(post_form_request(
            "/connections/add",
            "conn_name=++alpha++&conn_provider=mock&conn_model=mock-model&conn_api_key=&conn_base_url=",
        ))
        .await
        .unwrap();
    assert_eq!(
        second.status(),
        StatusCode::BAD_REQUEST,
        "a case-and-space variant must be refused"
    );
    let body = body_string(second).await;
    assert!(body.contains("already exists"), "body: {body}");
}

// [docs/specs/settings.md] SCENARIO: 20.11
#[tokio::test]
async fn test_connection_edit_keeps_its_own_name() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let added = app
        .clone()
        .oneshot(post_form_request(
            "/connections/add",
            "conn_name=Alpha&conn_provider=mock&conn_model=alpha-model&conn_api_key=&conn_base_url=",
        ))
        .await
        .unwrap();
    assert_eq!(added.status(), StatusCode::OK);
    let panel = body_string(added).await;
    let id = added_connection_id(&panel);

    let response = app
        .oneshot(post_form_request(
            &format!("/connections/{id}/edit"),
            "conn_name=Alpha&conn_provider=mock&conn_model=alpha-model-2&conn_api_key=&conn_base_url=",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_string(response).await;
    assert!(body.contains("Alpha"), "body: {body}");
    assert!(body.contains("alpha-model-2"), "body: {body}");
}
