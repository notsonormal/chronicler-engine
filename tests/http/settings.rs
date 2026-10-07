//! HTTP E2E tests for the settings endpoints: sub-tabs, role rows, and the text-check auto-save.

use axum::body::Body;
use axum::http::{self, Request, StatusCode};
use chrono::Utc;
use tower::util::ServiceExt;

use chronicler_engine::adapters::driving::http::builders::router::build_router;
use chronicler_engine::domain::model::llm_backend::LlmBackendType;
use chronicler_engine::domain::model::llm_message::LlmMessage;
use chronicler_engine::domain::model::settings::{AppSettings, LlmProviderConfig};
use chronicler_engine::test_support::body_text;
use chronicler_engine::TestAppBuilder;

use crate::SettingsTestGuard;
use crate::support::app_wiring::app_with_production_graph;
use crate::test_utils::card_html_slice;

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
    body_text(response).await
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

fn llm_message(agent: &str, error: Option<&str>, created_offset_secs: i64) -> LlmMessage {
    LlmMessage {
        id: 0,
        agent_name: agent.to_string(),
        backend_name: "Mock".to_string(),
        model_name: "mock".to_string(),
        system_prompt: String::new(),
        user_prompt: String::new(),
        raw_request_json: String::new(),
        raw_response_json: String::new(),
        parsed_response: String::new(),
        error_message: error.map(str::to_string),
        created_at: Utc::now() + chrono::Duration::seconds(created_offset_secs),
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

/// The `<div class="connection-row">…</div>` slice for connection `id` in the
/// rendered settings panel. Rows carry no data-id, so the anchor is the
/// connection's own edit link and the slice ends at the next row.
fn connection_row<'a>(panel: &'a str, id: &str) -> &'a str {
    let anchor = format!(r#"hx-get="/fragment/connections/{id}/edit"#);
    card_html_slice(panel, "connection-row", &anchor)
        .unwrap_or_else(|| panic!("no connection row for '{id}' in panel: {panel}"))
}

// [docs/specs/settings.md] SCENARIO: 20.1
#[tokio::test]
async fn test_settings_panel_renders_the_sub_tabs_and_role_rows() {
    let app = TestAppBuilder::default_app();

    let body = get_body(&app, "/fragment/settings").await;

    assert!(body.contains(r#"<div class="settings-panel">"#));
    assert!(body.contains(r#"role="tablist""#));
    assert!(body.contains(
        r#"id="subtab-connections" aria-controls="settings-connections" aria-selected="true""#
    ));
    assert!(body.contains(r#"id="subtab-text-check""#));
    assert!(body.contains(r#"id="settings-connections""#));
    assert!(body.contains(r#"id="settings-text-check""#));
    assert!(body.contains(r#"id="role-select-narrator""#));
    assert!(body.contains(r#"id="role-select-quantifier""#));
    assert!(body.contains(r#"<div class="connection-row">"#));
    assert!(body.contains(r#"<span class="connection-name">openrouter-gpt-4o-mini</span>"#));
    assert!(body.contains("OpenRouter - openai/gpt-4o-mini"));
    assert!(body.contains("Add Connection"));
    assert!(!body.contains("Set as Narrator"));
    assert!(!body.contains("Set as Quantifier"));
    assert!(
        !body.contains(r#"id="conn_name""#),
        "the always-open Add form must be gone: {body}"
    );
    assert!(body.contains(r#"id="text-check-card""#));
    assert!(body.contains(r#"id="check_mode""#));
    assert!(body.contains(r#"name="enable_auto_check""#));
}

// [docs/specs/settings.md] SCENARIO: 20.12
#[tokio::test]
async fn test_text_check_autosave_stores_mode_and_check_before_sending() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = post_form_request(
        "/settings/text-check",
        "check_mode=spell&enable_auto_check=true",
    );
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(
        body.contains(r#"id="text-check-card"#),
        "the auto-save must return the re-rendered card: {body}"
    );
    assert!(
        body.contains("Saved"),
        "the card must carry its own save feedback: {body}"
    );

    let panel = get_body(&app, "/fragment/settings").await;
    assert!(
        panel.contains(r#"<option value="spell" selected>"#),
        "the stored mode must render selected: {panel}"
    );
    assert!(
        panel.contains(r#"name="enable_auto_check" value="true" checked"#),
        "the stored check-before-sending must render checked: {panel}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.13
#[tokio::test]
async fn test_text_check_disabled_clears_check_before_sending() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let req = post_form_request(
        "/settings/text-check",
        "check_mode=disabled&enable_auto_check=true",
    );
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let panel = get_body(&app, "/fragment/settings").await;
    assert!(
        panel.contains(r#"<option value="disabled" selected>"#),
        "the stored mode must render Disabled selected: {panel}"
    );
    assert!(
        !panel.contains(r#"name="enable_auto_check" value="true" checked"#),
        "a Disabled mode must clear check-before-sending: {panel}"
    );

    let start = panel
        .find(r#"name="enable_auto_check""#)
        .expect("the check box must render");
    let end = start + panel[start..].find('>').unwrap_or(0);
    let tag = &panel[start..end];
    assert!(
        tag.contains("disabled"),
        "the check box must be disabled: {tag}"
    );
    assert!(
        !tag.contains("checked"),
        "the check box must be cleared: {tag}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.8
#[tokio::test]
async fn test_narrator_role_select_applies_at_once() {
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

    let backend = get_body(&app, "/debug/backend").await;
    assert!(
        backend.contains("mock-model-a"),
        "the seeded narrator should serve the first resolution: {backend}"
    );

    let response = app
        .clone()
        .oneshot(post_form_request(
            "/connections/set-narrator",
            "connection_id=mock-b",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(
        body.contains(r#"<option value="mock-b" selected>"#),
        "the role row must render the new selection: {body}"
    );

    let backend = get_body(&app, "/debug/backend").await;
    assert!(
        backend.contains("mock-model-b"),
        "the switch should take effect on the next resolution: {backend}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.14
#[tokio::test]
async fn test_quantifier_role_select_applies_at_once() {
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
    let (app, state, _storage) = app_with_production_graph(settings);

    let response = app
        .oneshot(post_form_request(
            "/connections/set-quantifier",
            "connection_id=mock-b",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(
        body.contains(r#"<option value="mock-b" selected>"#),
        "the role row must render the new selection: {body}"
    );

    let stored = state
        .settings_service
        .get_settings()
        .expect("settings read should succeed");
    assert_eq!(stored.quantifier_connection_id, "mock-b");
}

// [docs/specs/settings.md] SCENARIO: 20.15
#[tokio::test]
async fn test_delete_connection_used_by_a_role_is_refused() {
    let _guard = SettingsTestGuard::new();

    let settings = AppSettings {
        connections: vec![
            mock_connection("mock-a", "mock-model-a"),
            mock_connection("mock-b", "mock-model-b"),
            mock_connection("mock-c", "mock-model-c"),
        ],
        narration_connection_id: "mock-a".into(),
        quantifier_connection_id: "mock-b".into(),
        ..AppSettings::default()
    };
    let (app, _state, _storage) = app_with_production_graph(settings);

    let response = app
        .clone()
        .oneshot(post_form_request("/connections/mock-a/delete", ""))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(
        body.contains("Narrator uses this connection"),
        "the refusal must name the role: {body}"
    );
    assert!(
        body.contains("role above"),
        "the refusal must point at the role rows: {body}"
    );
    assert!(
        body.contains(r#"hx-get="/fragment/connections/mock-a/edit"#),
        "the refused connection must stay in the list: {body}"
    );

    let response = app
        .clone()
        .oneshot(post_form_request("/connections/mock-b/delete", ""))
        .await
        .unwrap();
    let body = body_text(response).await;
    assert!(
        body.contains("Quantifier uses this connection"),
        "the refusal must name the Quantifier: {body}"
    );

    let response = app
        .oneshot(post_form_request("/connections/mock-c/delete", ""))
        .await
        .unwrap();
    let body = body_text(response).await;
    assert!(
        !body.contains(r#"hx-get="/fragment/connections/mock-c/edit"#),
        "an unused connection must still be deletable: {body}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.16
#[tokio::test]
async fn test_role_rows_show_each_roles_health() {
    let (state, storage) = TestAppBuilder::default_test().build_service_with_storage();
    let app = build_router(state);

    let body = get_body(&app, "/fragment/settings").await;
    assert_eq!(
        body.matches("No calls yet").count(),
        2,
        "both role rows must start with no recorded calls: {body}"
    );
    assert!(!body.contains("subtab-degraded-dot"));

    storage
        .save_llm_message(&llm_message("narrator", Some("connection refused"), -10))
        .unwrap();
    let body = get_body(&app, "/fragment/settings").await;
    assert!(
        body.contains("Degraded"),
        "the failed role must degrade: {body}"
    );
    assert!(
        body.contains("subtab-degraded-dot"),
        "the Connections sub-tab must mark the degraded role: {body}"
    );
    assert!(
        body.contains(r#"<pre class="error-detail-raw">connection refused</pre>"#),
        "the raw failure text must be reachable in the disclosure: {body}"
    );

    storage
        .save_llm_message(&llm_message("narrator", None, 0))
        .unwrap();
    let body = get_body(&app, "/fragment/settings").await;
    assert!(
        body.contains("Healthy"),
        "a later success must report Healthy: {body}"
    );
    assert!(!body.contains("subtab-degraded-dot"));
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
    let body = body_text(second).await;
    assert!(body.contains("Duplicate Probe"), "body: {body}");
    assert!(body.contains("already exists"), "body: {body}");

    let panel = get_body(&app, "/fragment/settings").await;
    assert_eq!(
        panel
            .matches(r#"<span class="connection-name">Duplicate Probe</span>"#)
            .count(),
        1,
        "the refused connection must not be stored: {panel}"
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
    let body = body_text(second).await;
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
    let panel = body_text(added).await;
    let id = added_connection_id(&panel);

    let response = app
        .oneshot(post_form_request(
            &format!("/connections/{id}/edit"),
            "conn_name=Alpha&conn_provider=mock&conn_model=alpha-model-2&conn_api_key=&conn_base_url=",
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    let row = connection_row(&body, &id);
    assert!(
        row.contains(r#"<span class="connection-name">Alpha</span>"#),
        "the row must keep the connection's own name: {row}"
    );
    assert!(
        row.contains("alpha-model-2"),
        "the row must show the changed model: {row}"
    );
}
