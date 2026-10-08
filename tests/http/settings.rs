//! HTTP E2E tests for the settings endpoints: sub-tabs, role rows, and the text-check auto-save.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{self, Request, StatusCode};
use chrono::Utc;
use tower::util::ServiceExt;

use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driving::http::builders::router::build_router;
use chronicler_engine::application::connection_test_service::ProviderFactory;
use chronicler_engine::application::ports::llm_provider::LlmProvider;
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

/// Added ids start with `conn-`; the seeded fixture connections carry
/// author-chosen ids.
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

/// Rows carry no data-id, so the slice is anchored on the connection's own edit
/// link and ends at the next row.
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
    assert!(
        body.contains(">Test</button>"),
        "each connection row must offer a Test control: {body}"
    );
    assert!(body.contains("connection-test-slot"));
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
    assert!(!body.contains("subtab-degraded-marker"));

    storage
        .save_llm_message(&llm_message("narrator", Some("connection refused"), -10))
        .unwrap();
    let body = get_body(&app, "/fragment/settings").await;
    assert!(
        body.contains("Degraded"),
        "the failed role must degrade: {body}"
    );
    assert!(
        body.contains("subtab-degraded-marker"),
        "the Connections sub-tab must mark the degraded role: {body}"
    );
    assert!(
        body.contains(r##"<use href="#i-triangle-alert""##),
        "the marker must carry an icon, or a colour-blind reader loses the cue: {body}"
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
    assert!(!body.contains("subtab-degraded-marker"));
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

// [docs/specs/settings.md] SCENARIO: 20.17
#[tokio::test]
async fn test_connection_add_unknown_provider_answers_non_2xx() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = app
        .clone()
        .oneshot(post_form_request(
            "/connections/add",
            "conn_name=Bogus&conn_provider=bogus_provider&conn_model=m&conn_api_key=&conn_base_url=",
        ))
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "an unknown provider must be refused, not answered as a 200 panel replacement"
    );
    let body = body_text(response).await;
    assert!(
        body.contains("bogus_provider"),
        "the failure must name the rejected provider: {body}"
    );

    let panel = get_body(&app, "/fragment/settings").await;
    assert!(
        panel.contains(r#"<div class="settings-panel">"#)
            && panel.contains(r#"<span class="connection-name">"#),
        "the panel and its connection rows must survive the refused add: {panel}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.18
#[tokio::test]
async fn test_connection_edit_unknown_provider_answers_non_2xx() {
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
        .clone()
        .oneshot(post_form_request(
            &format!("/connections/{id}/edit"),
            "conn_name=Alpha&conn_provider=bogus_provider&conn_model=alpha-model&conn_api_key=&conn_base_url=",
        ))
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "an unknown provider on edit must be refused"
    );
    let body = body_text(response).await;
    assert!(body.contains("bogus_provider"), "body: {body}");

    let panel = get_body(&app, "/fragment/settings").await;
    assert!(
        panel.contains(r#"<span class="connection-name">Alpha</span>"#),
        "the connection must survive the refused edit: {panel}"
    );
}

/// One Mock connection serving both roles, so the production graph can resolve
/// its eager boot check.
fn single_mock_settings(id: &str, model: &str) -> AppSettings {
    AppSettings {
        connections: vec![mock_connection(id, model)],
        narration_connection_id: id.into(),
        quantifier_connection_id: id.into(),
        ..AppSettings::default()
    }
}

/// A provider that fails every call, so a connection test can exercise the
/// failure shape without a network dependency.
fn failing_connection_test_factory() -> ProviderFactory {
    Arc::new(|_config: &LlmProviderConfig| {
        Arc::new(MockBackend::new().with_fail()) as Arc<dyn LlmProvider>
    })
}

// [docs/specs/settings.md] SCENARIO: 20.19
#[tokio::test]
async fn test_saved_connection_test_reports_success_and_records_nothing() {
    let _guard = SettingsTestGuard::new();
    let (app, state, _storage) =
        app_with_production_graph(single_mock_settings("mock-a", "mock-model-a"));

    let response = app
        .oneshot(post_form_request("/connections/mock-a/test", ""))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(
        body.contains("connection-test-result success"),
        "a passing test must report success: {body}"
    );
    assert!(
        body.contains("mock-model-a"),
        "the reply must name the connection's model: {body}"
    );
    assert!(
        body.contains(" ms"),
        "the reply must carry the reply time: {body}"
    );

    assert!(
        state
            .game_view_query
            .list_latest_llm_messages(50)
            .unwrap()
            .is_empty(),
        "a connection test must not record a forensic row"
    );
    assert!(
        state
            .game_view_query
            .role_health()
            .unwrap()
            .iter()
            .all(|health| health.backend_model.is_none() && health.last_error.is_none()),
        "a connection test must not move role health"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.20
#[tokio::test]
async fn test_failed_saved_connection_test_renders_the_error_disclosure() {
    let _guard = SettingsTestGuard::new();
    let (state, _storage) = TestAppBuilder::default_test()
        .settings(single_mock_settings("mock-a", "mock-model-a"))
        .build_service_with_storage();
    let app = build_router(
        state
            .clone()
            .with_connection_test_provider(failing_connection_test_factory()),
    );

    let response = app
        .oneshot(post_form_request("/connections/mock-a/test", ""))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(
        body.contains(r#"<div class="error-disclosure">"#),
        "a failed test must render the shared disclosure: {body}"
    );
    assert!(
        body.contains(r#"id="connection-test-mock-a-popover""#),
        "the disclosure must be anchored: {body}"
    );
    assert!(
        body.contains(r#"<pre class="error-detail-raw">"#),
        "the raw failure text must be reachable: {body}"
    );
    assert!(
        body.contains("configured_failure"),
        "the raw failure text must name the provider error: {body}"
    );
    assert!(
        state
            .game_view_query
            .list_latest_llm_messages(50)
            .unwrap()
            .is_empty(),
        "a failed connection test must not record a forensic row"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.21
#[tokio::test]
async fn test_form_connection_test_uses_the_typed_values_before_save() {
    let _guard = SettingsTestGuard::new();
    let (state, _storage) = TestAppBuilder::default_test()
        .settings(single_mock_settings("saved-conn", "saved-model"))
        .build_service_with_storage();
    let app = build_router(state.clone());

    let response = app
        .oneshot(post_form_request(
            "/connections/test",
            "conn_name=Typed&conn_provider=mock&conn_model=typed-model&conn_api_key=&conn_base_url=",
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_text(response).await;
    assert!(
        body.contains("connection-test-result success"),
        "a passing form test must report success: {body}"
    );
    assert!(
        body.contains("typed-model"),
        "the form test must use the model typed into the form: {body}"
    );

    let stored = state
        .settings_service
        .get_settings()
        .expect("settings read should succeed");
    assert_eq!(
        stored.connections.len(),
        1,
        "a form test must not save the typed connection"
    );
    assert_eq!(stored.connections[0].id, "saved-conn");
}
