//! HTTP E2E tests for the settings endpoints: sub-tabs, role rows, and the text-check auto-save.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;

use chronicler_engine::adapters::driven::storage::{StorageCall, TestOverride};
use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driving::http::builders::router::build_router;
use chronicler_engine::application::connection_test_service::ProviderFactory;
use chronicler_engine::application::ports::llm_provider::LlmProvider;
use chronicler_engine::domain::model::settings::{AppSettings, LlmConnection};
use chronicler_engine::test_support::{assert_answers_nothing, body_text};
use chronicler_engine::test_support::fixtures::llm_message;
use chronicler_engine::TestAppBuilder;
use chronicler_engine::test_support::empty_sqlite_storage;

use crate::SettingsTestGuard;
use crate::support::app_wiring::app_with_production_graph;
use crate::support::http_fixtures::mock_connection;
use crate::support::http_requests::post_form;
use crate::test_utils::card_html_slice;

async fn get_body(app: &axum::Router, uri: &str) -> String {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    body_text(response).await
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

    assert!(
        !body.contains("settings-panel"),
        "the shell's .settings-panel is the part: the fragment repeats no root: {body}"
    );
    assert!(
        !body.contains("connection-editor") && !body.contains("editor-slot"),
        "the connection forms open in the dialog, so the panel renders no editor slot: {body}"
    );
    assert!(body.contains(r#"role="tablist""#));
    assert!(body.contains(
        r#"id="subtab-connections" aria-controls="settings-connections" aria-selected="true" data-keep="aria-selected""#
    ));
    assert!(
        body.contains(r#"id="settings-text-check" role="tabpanel" aria-labelledby="subtab-text-check" hidden data-keep="hidden""#),
        "the Text Check panel starts hidden, and its state is kept through a load: {body}"
    );
    assert!(body.contains(r#"id="role-row-narrator""#));
    assert!(body.contains(r#"id="role-row-quantifier""#));
    assert!(body.contains(r#"id="subtab-text-check""#));
    assert!(body.contains(r#"id="settings-connections""#));
    assert!(body.contains(r#"id="settings-text-check""#));
    assert!(body.contains(r#"id="role-select-narrator""#));
    assert!(body.contains(r#"id="role-select-quantifier""#));
    assert!(body.contains(r#"<div class="connection-row" id="connection-"#));
    assert!(body.contains(r#"<span class="connection-name">openrouter-gpt-4o-mini</span>"#));
    assert!(body.contains("OpenRouter - openai/gpt-4o-mini"));
    assert!(
        body.contains(">Test</button>"),
        "each connection row must offer a Test control: {body}"
    );
    assert!(
        !body.contains("connection-test-slot"),
        "the result of a row Test opens in the dialog, so a row renders no result slot: {body}"
    );
    assert!(
        !body.contains("settings-subtab active") && !body.contains("settings-subtab-panel active"),
        "the active sub-tab is no class: {body}"
    );
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
    assert!(
        body.contains(
            r#"class="text-check-status saved-flag" id="text-check-status" data-keep="data-saved""#
        ),
        "the text-check status must be a saved-flag that a load keeps, so the browser can show Saved: {body}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.34
#[tokio::test]
async fn test_role_rows_and_connection_rows_report_failures_in_their_own_error_slots() {
    let app = TestAppBuilder::default_app();

    let body = get_body(&app, "/fragment/settings").await;

    for slot in [
        "role-narrator",
        "role-quantifier",
        "connection-openrouter-gpt-4o-mini",
    ] {
        assert!(
            body.contains(&format!(r#"data-error-slot="{slot}""#)),
            "error slot {slot}: {body}"
        );
    }
    assert!(
        body.contains(r#"data-error-slot="text-check-card""#),
        "the text-check card has its own error slot: {body}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.35
#[tokio::test]
async fn test_panel_controls_swap_nothing_into_the_panel() {
    let app = TestAppBuilder::default_app();

    let body = get_body(&app, "/fragment/settings").await;

    assert!(
        body.contains(r#"hx-trigger="change" hx-swap="none""#),
        "a role select saves on change and swaps nothing: {body}"
    );
    assert!(
        body.contains(r##"hx-get="/fragment/connections/new" hx-target="#edit-dialog-body""##),
        "Add Connection opens the dialog: {body}"
    );
    assert!(
        body.contains(r##"hx-get="/fragment/connections/openrouter-gpt-4o-mini/edit" hx-target="#edit-dialog-body""##),
        "Edit opens the dialog: {body}"
    );
    assert!(
        body.contains(
            r##"hx-post="/connections/openrouter-gpt-4o-mini/test" hx-target="#edit-dialog-body""##
        ),
        "Test shows its result in the dialog: {body}"
    );
    assert!(
        !body.contains(r#"hx-target=".settings-panel""#),
        "no control targets the panel: {body}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.36
#[tokio::test]
async fn test_connection_forms_answer_dialog_fragments() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let add = get_body(&app, "/fragment/connections/new").await;
    assert!(
        add.contains("data-dialog-form")
            && add.contains(r#"<h2 id="edit-dialog-title">Add Connection</h2>"#),
        "the Add form is a dialog fragment with its own title: {add}"
    );
    assert!(add.contains(r#"hx-post="/connections/add""#), "{add}");

    let edit = get_body(&app, "/fragment/connections/openrouter-gpt-4o-mini/edit").await;
    assert!(
        edit.contains("data-dialog-form")
            && edit.contains(r#"<h2 id="edit-dialog-title">Edit openrouter-gpt-4o-mini</h2>"#),
        "the Edit form is a dialog fragment with its own title: {edit}"
    );
    assert!(
        edit.contains(r#"hx-post="/connections/openrouter-gpt-4o-mini/edit""#),
        "{edit}"
    );
    for form in [&add, &edit] {
        assert!(
            form.contains("data-close-dialog"),
            "a Cancel control: {form}"
        );
        assert!(
            form.contains(r#"hx-post="/connections/test""#)
                && form.contains(r#"hx-include="closest form""#),
            "a Test control for the typed values: {form}"
        );
        assert!(
            form.contains(r#"data-error-slot="connection-form""#),
            "the form reports a refusal in its own slot: {form}"
        );
        assert!(
            form.contains("data-refusal-what="),
            "the form names what a busy refusal leaves unsaved: {form}"
        );
        assert!(
            !form.contains("data-close-editor") && !form.contains("data-work"),
            "no editor-slot marker: {form}"
        );
    }
}

// [docs/specs/settings.md] SCENARIO: 20.12
#[tokio::test]
async fn test_text_check_autosave_stores_mode_and_check_before_sending() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let response = post_form(
        &app,
        "/settings/text-check",
        "check_mode=spell&enable_auto_check=true",
    )
    .await;

    assert_answers_nothing(response).await;

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

    let response = post_form(
        &app,
        "/settings/text-check",
        "check_mode=disabled&enable_auto_check=true",
    )
    .await;
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

// [docs/specs/settings.md] SCENARIO: 20.22
#[tokio::test]
async fn test_failed_text_check_save_answers_non_2xx_and_swaps_no_card() {
    let _guard = SettingsTestGuard::new();
    let storage = Arc::new(empty_sqlite_storage().with_failure(
        StorageCall::UpdateSettings,
        TestOverride::internal("text check save failure"),
    ));
    let app = TestAppBuilder::default_test().storage(storage).build();

    let response = post_form(
        &app,
        "/settings/text-check",
        "check_mode=spell&enable_auto_check=true",
    )
    .await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = body_text(response).await;
    assert!(
        body.contains("text check save failure"),
        "the failure must name the storage error: {body}"
    );
    assert!(
        !body.contains(r#"id="text-check-card"#),
        "a failed save must not answer a card, which the client would swap in: {body}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.23
#[tokio::test]
async fn test_failed_settings_panel_load_reports_inside_the_panel() {
    let _guard = SettingsTestGuard::new();
    let storage = Arc::new(empty_sqlite_storage());
    let app = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build();
    storage.add_failure(
        StorageCall::GetSettings,
        TestOverride::internal("settings read failure"),
    );

    let request = Request::builder()
        .uri("/fragment/settings")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "a failed load answers non-2xx, so the screen load that reads it fails"
    );
    assert!(response.headers().get("HX-Reswap").is_none());
    let body = body_text(response).await;

    assert!(
        body.contains("settings read failure"),
        "a panel that cannot read settings must render the failure: {body}"
    );
    assert!(
        !body.contains(r#"id="subtab-connections""#),
        "the panel must not render a settings panel it could not read: {body}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.24
#[tokio::test]
async fn test_unknown_connection_edit_form_is_refused_and_opens_no_dialog() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let request = Request::builder()
        .uri("/fragment/connections/does-not-exist/edit")
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "a refusal answers non-2xx, so no dialog opens"
    );
    let body = body_text(response).await;

    assert!(
        body.contains("Connection not found"),
        "the panel must report the unknown connection: {body}"
    );
    assert!(
        !body.contains(r#"id="conn_name"#),
        "an unknown connection must not render an edit form: {body}"
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

    let response = post_form(&app, "/connections/set-narrator", "connection_id=mock-b").await;
    assert_answers_nothing(response).await;
    let panel = get_body(&app, "/fragment/settings").await;
    let narrator_row = panel
        .split(r#"id="role-row-quantifier""#)
        .next()
        .expect("the Narrator row comes first");
    assert!(
        narrator_row.contains(r#"<option value="mock-b" selected>"#),
        "the role row must render the new selection: {panel}"
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

    let response = post_form(&app, "/connections/set-quantifier", "connection_id=mock-b").await;
    assert_answers_nothing(response).await;
    let panel = get_body(&app, "/fragment/settings").await;
    let quantifier_row = panel
        .split(r#"id="role-row-quantifier""#)
        .nth(1)
        .expect("the Quantifier row follows the Narrator row");
    assert!(
        quantifier_row.contains(r#"<option value="mock-b" selected>"#),
        "the role row must render the new selection: {panel}"
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

    let response = post_form(&app, "/connections/mock-a/delete", "").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_text(response).await;
    assert!(
        body.contains("Narrator uses this connection"),
        "the refusal must name the role: {body}"
    );
    assert!(
        body.contains("role above"),
        "the refusal must point at the role rows: {body}"
    );
    let panel = get_body(&app, "/fragment/settings").await;
    assert!(
        panel.contains(r#"hx-get="/fragment/connections/mock-a/edit"#),
        "the refused connection must stay in the list: {panel}"
    );

    let response = post_form(&app, "/connections/mock-b/delete", "").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_text(response).await;
    assert!(
        body.contains("Quantifier uses this connection"),
        "the refusal must name the Quantifier: {body}"
    );

    let response = post_form(&app, "/connections/mock-c/delete", "").await;
    assert_answers_nothing(response).await;
    let panel = get_body(&app, "/fragment/settings").await;
    assert!(
        !panel.contains(r#"hx-get="/fragment/connections/mock-c/edit"#),
        "an unused connection must still be deletable: {panel}"
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
        .save_llm_message(&llm_message(
            "narrator",
            "mock",
            Some("connection refused"),
            -10,
        ))
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
        body.contains(r#"title="A role is degraded (engine-wide role health)""#),
        "the marker's tooltip must state that role health covers every game: {body}"
    );
    assert!(
        body.contains(r#"<pre class="error-detail-raw">connection refused</pre>"#),
        "the raw failure text must be reachable in the disclosure: {body}"
    );

    storage
        .save_llm_message(&llm_message("narrator", "mock", None, 0))
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

    let first = post_form(
        &app,
        "/connections/add",
        "conn_name=Duplicate+Probe&conn_provider=mock&conn_model=mock-model&conn_api_key=&conn_base_url=",
    )
    .await;
    assert_eq!(first.status(), StatusCode::OK);

    let second = post_form(
        &app,
        "/connections/add",
        "conn_name=Duplicate+Probe&conn_provider=mock&conn_model=mock-model&conn_api_key=&conn_base_url=",
    )
    .await;
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

    let first = post_form(
        &app,
        "/connections/add",
        "conn_name=Alpha&conn_provider=mock&conn_model=mock-model&conn_api_key=&conn_base_url=",
    )
    .await;
    assert_eq!(first.status(), StatusCode::OK);

    let second = post_form(
        &app,
        "/connections/add",
        "conn_name=++alpha++&conn_provider=mock&conn_model=mock-model&conn_api_key=&conn_base_url=",
    )
    .await;
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

    let added = post_form(
        &app,
        "/connections/add",
        "conn_name=Alpha&conn_provider=mock&conn_model=alpha-model&conn_api_key=&conn_base_url=",
    )
    .await;
    assert_eq!(added.status(), StatusCode::OK);
    let panel = get_body(&app, "/fragment/settings").await;
    let id = added_connection_id(&panel);

    let response = post_form(
        &app,
        &format!("/connections/{id}/edit"),
        "conn_name=Alpha&conn_provider=mock&conn_model=alpha-model-2&conn_api_key=&conn_base_url=",
    )
    .await;
    assert_answers_nothing(response).await;
    let panel = get_body(&app, "/fragment/settings").await;
    let row = connection_row(&panel, &id);
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

    let response = post_form(
        &app,
        "/connections/add",
        "conn_name=Bogus&conn_provider=bogus_provider&conn_model=m&conn_api_key=&conn_base_url=",
    )
    .await;
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
        panel.contains(r#"<span class="connection-name">"#),
        "the connection rows must survive the refused add: {panel}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.18
#[tokio::test]
async fn test_connection_edit_unknown_provider_answers_non_2xx() {
    let _guard = SettingsTestGuard::new();
    let app = TestAppBuilder::default_app();

    let added = post_form(
        &app,
        "/connections/add",
        "conn_name=Alpha&conn_provider=mock&conn_model=alpha-model&conn_api_key=&conn_base_url=",
    )
    .await;
    assert_eq!(added.status(), StatusCode::OK);
    let panel = get_body(&app, "/fragment/settings").await;
    let id = added_connection_id(&panel);

    let response = post_form(
        &app,
        &format!("/connections/{id}/edit"),
        "conn_name=Alpha&conn_provider=bogus_provider&conn_model=alpha-model&conn_api_key=&conn_base_url=",
    )
    .await;
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
    Arc::new(|_config: &LlmConnection| {
        Arc::new(MockBackend::new().with_fail()) as Arc<dyn LlmProvider>
    })
}

// [docs/specs/settings.md] SCENARIO: 20.19
#[tokio::test]
async fn test_saved_connection_test_reports_success_and_records_nothing() {
    let _guard = SettingsTestGuard::new();
    let (app, state, _storage) =
        app_with_production_graph(single_mock_settings("mock-a", "mock-model-a"));

    let response = post_form(&app, "/connections/mock-a/test", "").await;

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
        body.contains("data-dialog-form")
            && body.contains(r#"<h2 id="edit-dialog-title">Test "#)
            && body.contains("data-close-dialog"),
        "the answer is a dialog fragment with a title and a Close control: {body}"
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

    let response = post_form(&app, "/connections/mock-a/test", "").await;

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
        body.contains("data-dialog-form") && body.contains("data-close-dialog"),
        "the failure shows in the dialog too: {body}"
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

    let response = post_form(
        &app,
        "/connections/test",
        "conn_name=Typed&conn_provider=mock&conn_model=typed-model&conn_api_key=&conn_base_url=",
    )
    .await;

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
