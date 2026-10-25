use askama::Template;

use crate::application::games::view_query::RoleHealth;
use crate::domain::model::agent::Role;
use crate::domain::model::llm_backend::LlmBackendType;
use crate::domain::model::settings::{AppSettings, LlmConnection, TextCheckMode, TextCheckSettings};
use crate::adapters::driving::http::settings::templates::{
    ConnectionFormTemplate, SettingsTemplate, TextCheckCardTemplate,
};
use crate::error::EngineError;
use crate::test_support::test_connection;

fn two_connection_settings() -> AppSettings {
    AppSettings {
        connections: vec![
            LlmConnection {
                name: "Test Narrator".into(),
                api_key: Some("sk-test".into()),
                ..test_connection("conn-1", LlmBackendType::OpenRouter)
            },
            LlmConnection {
                name: "Test Quantifier".into(),
                model: "llama3".into(),
                base_url: Some("http://localhost:11434".into()),
                ..test_connection("conn-2", LlmBackendType::Ollama)
            },
        ],
        narration_connection_id: "conn-1".into(),
        quantifier_connection_id: "conn-2".into(),
        response_length: "flexible".into(),
        ..Default::default()
    }
}

fn health(role: Role, backend: Option<&str>, error: Option<&str>) -> RoleHealth {
    RoleHealth {
        role,
        backend_model: backend.map(str::to_string),
        last_error: error.map(str::to_string),
    }
}

#[test]
fn test_settings_template_renders_the_sub_tabs_and_role_rows() {
    let settings = two_connection_settings();
    let html = SettingsTemplate::from_settings(&settings, &[], None)
        .render()
        .unwrap();

    assert!(html.contains(r#"role="tablist""#));
    assert!(html.contains(r#"id="subtab-connections""#));
    assert!(html.contains(r#"id="subtab-text-check""#));
    assert!(html.contains(r#"id="settings-connections""#));
    assert!(html.contains(r#"id="settings-text-check""#));
    assert!(html.contains(r#"id="role-select-narrator""#));
    assert!(html.contains(r#"id="role-select-quantifier""#));
    assert!(html.contains(r#"<option value="conn-1" selected>"#));
    assert!(html.contains(r#"<option value="conn-2" selected>"#));
    assert!(html.contains("Add Connection"));
    assert!(html.contains(r#"<div class="connection-row">"#));
    assert!(!html.contains("Set as Narrator"));
    assert!(!html.contains("Set as Quantifier"));
}

#[test]
fn test_narrator_badge_renders() {
    let settings = two_connection_settings();
    let html = SettingsTemplate::from_settings(&settings, &[], None)
        .render()
        .unwrap();

    assert!(html.contains(r#"<span class="badge">Narrator</span>"#));
}

#[test]
fn test_quantifier_badge_renders() {
    let settings = two_connection_settings();
    let html = SettingsTemplate::from_settings(&settings, &[], None)
        .render()
        .unwrap();

    assert!(html.contains(r#"<span class="badge quantifier">Quantifier</span>"#));
}

#[test]
fn test_no_calls_yet_health_renders_for_each_role() {
    let settings = two_connection_settings();
    let html = SettingsTemplate::from_settings(&settings, &[], None)
        .render()
        .unwrap();

    assert_eq!(
        html.matches("No calls yet").count(),
        2,
        "both role rows must report no recorded calls: {html}"
    );
    assert!(!html.contains("subtab-degraded-marker"));
}

#[test]
fn test_degraded_role_renders_the_details_disclosure_and_marks_the_subtab() {
    let settings = two_connection_settings();
    let roles = vec![health(
        Role::Narrator,
        Some("Mock mock"),
        Some("connection refused"),
    )];
    let html = SettingsTemplate::from_settings(&settings, &roles, None)
        .render()
        .unwrap();

    assert!(html.contains("subtab-degraded-marker"));
    assert!(
        html.contains(r##"<use href="#i-triangle-alert""##),
        "the degraded marker must carry an icon, or the cue is the colour alone: {html}"
    );
    assert!(
        html.contains(r#"class="error-disclosure""#),
        "the row must carry the shared disclosure: {html}"
    );
    assert!(html.contains("error-details-toggle"));
    assert!(
        html.contains("Narrator failed — no narration was written"),
        "the row's message must name the role's failure once, without the header's engine-wide suffix: {html}"
    );
    assert!(
        html.contains(r#"<pre class="error-detail-raw">connection refused</pre>"#),
        "the raw failure text must be reachable in the disclosure: {html}"
    );
}

#[test]
fn test_healthy_role_renders_healthy() {
    let settings = two_connection_settings();
    let roles = vec![health(Role::Quantifier, Some("Mock mock"), None)];
    let html = SettingsTemplate::from_settings(&settings, &roles, None)
        .render()
        .unwrap();

    assert!(html.contains("Healthy"));
    assert!(!html.contains("subtab-degraded-marker"));
}

#[test]
fn test_refusal_message_rides_inside_the_connections_panel() {
    let settings = two_connection_settings();
    let refusal = EngineError::Validation("Narrator uses this connection".to_string());
    let roles = vec![health(
        Role::Narrator,
        Some("Mock mock"),
        Some("connection refused"),
    )];
    let html = SettingsTemplate::from_settings(&settings, &roles, Some(&refusal))
        .render()
        .unwrap();

    assert!(
        html.contains(
            r#"<span class="error-disclosure-message"><span class="error-disclosure-text">Narrator uses this connection</span></span>"#
        ),
        "the refusal's short message must be the disclosure's visible text: {html}"
    );
    assert!(
        html.contains(r#"class="error-details-toggle""#),
        "the refusal must carry the Details toggle: {html}"
    );
    assert!(
        html.contains(
            r#"<pre class="error-detail-raw">Validation error: Narrator uses this connection</pre>"#
        ),
        "the popover must carry the raw failure text: {html}"
    );
    for popover_id in ["settings-panel-error", "role-health-narrator-popover"] {
        assert_eq!(
            html.matches(&format!(r#"id="{popover_id}""#)).count(),
            1,
            "the panel refusal and a role-health disclosure must not share a popover id: {html}"
        );
    }
}

#[test]
fn test_connection_form_template_add_and_edit() {
    let settings = two_connection_settings();
    let add = ConnectionFormTemplate::new(None).render().unwrap();
    assert!(add.contains(r#"<h2>Add Connection</h2>"#));
    assert!(add.contains(r#"hx-post="/connections/add""#));
    assert!(add.contains("</svg> Connections</button>"));
    assert!(add.contains(r#"name="conn_name" value=""#));

    let edit = ConnectionFormTemplate::new(Some(&settings.connections[0]))
        .render()
        .unwrap();
    assert!(edit.contains(r#"<h2>Edit Test Narrator</h2>"#));
    assert!(edit.contains(r#"hx-post="/connections/conn-1/edit""#));
    assert!(edit.contains(r#"name="conn_model" value="openai/gpt-4o-mini""#));
    assert!(edit.contains(r#"name="conn_api_key" value="sk-test""#));
}

#[test]
fn test_connection_form_template_escapes_the_name() {
    let mut settings = two_connection_settings();
    settings.connections[0].name = "Test <script>alert('xss')</script>".into();
    let html = ConnectionFormTemplate::new(Some(&settings.connections[0]))
        .render()
        .unwrap();

    assert!(!html.contains("<script>"));
    assert!(
        html.contains("&#60;script&#62;"),
        "the name must be HTML-escaped: {html}"
    );
}

#[test]
fn test_connection_form_template_shows_the_stored_context_window() {
    let mut settings = two_connection_settings();
    settings.connections[0].max_context_tokens = Some(65536);

    let edit = ConnectionFormTemplate::new(Some(&settings.connections[0]))
        .render()
        .unwrap();
    assert!(
        edit.contains(r#"name="conn_max_context_tokens" value="65536" min="1""#),
        "the edit form must show the stored window: {edit}"
    );

    let add = ConnectionFormTemplate::new(None).render().unwrap();
    assert!(
        add.contains(r#"name="conn_max_context_tokens" value="""#),
        "the add form must leave the window empty: {add}"
    );
    assert!(
        add.contains("provider default"),
        "the empty field must say that it follows the provider default: {add}"
    );
}

#[test]
fn test_text_check_card_disabled_mode_renders_the_checkbox_unchecked() {
    let settings = AppSettings {
        text_check: TextCheckSettings {
            mode: TextCheckMode::Disabled,
            enable_auto_check: true,
            ignored_words: Vec::new(),
        },
        ..AppSettings::default()
    };

    let html = TextCheckCardTemplate::from_settings(&settings, "")
        .render()
        .unwrap();

    assert!(
        !html.contains(r#"name="enable_auto_check" value="true" checked"#),
        "Disabled mode must render the check box unchecked even when the stored pair is inconsistent: {html}"
    );
    assert!(
        html.contains("disabled"),
        "Disabled mode must render the check box disabled: {html}"
    );
}
