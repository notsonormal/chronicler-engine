//! HTTP E2E tests for the connection form: the context window and the reasoning effort the Add, Edit and Test actions take.

use std::sync::{Arc, Mutex};

use axum::http::StatusCode;

use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driving::http::builders::router::build_router;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::application::connection_test_service::ProviderFactory;
use chronicler_engine::application::ports::llm_provider::LlmProvider;
use chronicler_engine::domain::model::llm_backend::LlmBackendType;
use chronicler_engine::domain::model::settings::{AppSettings, LlmProviderConfig, ReasoningEffort};
use chronicler_engine::test_support::body_text;
use chronicler_engine::TestAppBuilder;

use crate::SettingsTestGuard;
use crate::support::app_wiring::long_story_app;
use crate::support::http_fixtures::mock_connection;
use crate::support::http_requests::{fetch_body, post_action, post_form, wait_settled};

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

// [docs/specs/settings.md] SCENARIO: 20.25
#[tokio::test]
async fn test_connection_context_window_applies_from_the_next_narration() {
    let _guard = SettingsTestGuard::new();
    // An empty window on a mock connection resolves the provider default of
    // 4096 tokens; the edit sets a window wide enough for the whole story log.
    // The padding story makes the window's size show in the prompt.
    let (app, state, _storage, _game_id) = long_story_app(None);

    assert!(post_action(&app, "first look").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the first narration should settle"
    );

    let edited = post_form(
        &app,
        "/connections/narration-conn/edit",
        "conn_name=Wide&conn_provider=mock&conn_model=mock-model&conn_api_key=&conn_base_url=&conn_max_context_tokens=131072",
    )
    .await;
    assert_eq!(
        edited.status(),
        StatusCode::OK,
        "the connection edit should save"
    );

    assert!(post_action(&app, "second look").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the second narration should settle"
    );

    let prompts: Vec<String> = state
        .game_view_query
        .list_latest_llm_messages(20)
        .expect("the recorded calls read back")
        .into_iter()
        .filter(|message| message.agent_name == "narrator")
        .map(|message| message.user_prompt)
        .collect();
    assert_eq!(prompts.len(), 2, "each Action records one narrator call");
    let tight = prompts[0].matches("Story entry ").count();
    let wide = prompts[1].matches("Story entry ").count();
    assert!(
        tight < 150,
        "the default window must not carry the whole story log: {tight} entries"
    );
    assert!(
        wide > tight,
        "the window the Connection holds at that moment decides how much of the story \
         log reaches the prompt: {tight} entries before the edit, {wide} after"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.26
#[tokio::test]
async fn test_empty_context_window_field_uses_the_provider_default() {
    let _guard = SettingsTestGuard::new();
    let (app, state, _storage, _game_id) = long_story_app(Some(131_072));

    assert!(post_action(&app, "first look").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the first narration should settle"
    );

    let edited = post_form(
        &app,
        "/connections/narration-conn/edit",
        "conn_name=Default&conn_provider=mock&conn_model=mock-model&conn_api_key=&conn_base_url=&conn_max_context_tokens=",
    )
    .await;
    assert_eq!(
        edited.status(),
        StatusCode::OK,
        "an empty window field should save"
    );

    assert!(post_action(&app, "second look").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the second narration should settle"
    );

    let prompts: Vec<String> = state
        .game_view_query
        .list_latest_llm_messages(20)
        .expect("the recorded calls read back")
        .into_iter()
        .filter(|message| message.agent_name == "narrator")
        .map(|message| message.user_prompt)
        .collect();
    assert_eq!(prompts.len(), 2, "each Action records one narrator call");
    let wide = prompts[0].matches("Story entry ").count();
    let narrow = prompts[1].matches("Story entry ").count();
    assert!(
        narrow < wide,
        "an empty window field must fall back to the provider default: {wide} entries \
         with the explicit window, {narrow} after it was cleared"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.27
#[tokio::test]
async fn test_edit_that_changes_the_provider_keeps_an_explicit_context_window() {
    let _guard = SettingsTestGuard::new();
    let mut connection = mock_connection("window-conn", "mock-model");
    connection.max_context_tokens = Some(65_536);
    let settings = AppSettings {
        connections: vec![connection],
        narration_connection_id: "window-conn".into(),
        quantifier_connection_id: "window-conn".into(),
        ..AppSettings::default()
    };
    let (state, _storage) = TestAppBuilder::default_test()
        .settings(settings)
        .build_service_with_storage();
    let app = build_router(state.clone());

    let response = post_form(
        &app,
        "/connections/window-conn/edit",
        "conn_name=Wide&conn_provider=ollama&conn_model=llama3&conn_api_key=&conn_base_url=&conn_max_context_tokens=65536",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    let stored = state
        .settings_service
        .get_settings()
        .expect("settings read should succeed");
    let edited = stored
        .find_connection("window-conn")
        .expect("the edited connection must still be stored");
    assert_eq!(edited.provider, LlmBackendType::Ollama);
    assert_eq!(
        edited.max_context_tokens,
        Some(65_536),
        "the explicit window must survive the provider change"
    );

    let form = fetch_body(&app, "/fragment/connections/window-conn/edit").await;
    assert!(
        form.contains(r#"name="conn_max_context_tokens" value="65536""#),
        "the edit form must show the explicit window: {form}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.28
#[tokio::test]
async fn test_context_window_that_is_not_a_positive_whole_number_is_refused() {
    let _guard = SettingsTestGuard::new();
    let mut connection = mock_connection("alpha-conn", "mock-model");
    connection.name = "Alpha".into();
    let settings = AppSettings {
        connections: vec![connection],
        narration_connection_id: "alpha-conn".into(),
        quantifier_connection_id: "alpha-conn".into(),
        ..AppSettings::default()
    };
    let (state, _storage) = TestAppBuilder::default_test()
        .settings(settings)
        .build_service_with_storage();
    let app = build_router(state.clone());

    for typed in ["wide", "0", "-5", "4294967296"] {
        let response = post_form(
            &app,
            "/connections/alpha-conn/edit",
            &format!(
                "conn_name=Alpha&conn_provider=mock&conn_model=mock-model&conn_api_key=&conn_base_url=&conn_max_context_tokens={typed}"
            ),
        )
        .await;
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "the window '{typed}' must be refused"
        );
        let body = body_text(response).await;
        assert!(
            body.contains("Context window must be a positive whole number"),
            "the refusal must name the window rule for '{typed}': {body}"
        );
    }

    let panel = fetch_body(&app, "/fragment/settings").await;
    assert!(
        panel.contains(r#"<span class="connection-name">Alpha</span>"#),
        "the refused edit must leave the connection list intact: {panel}"
    );
    let stored = state
        .settings_service
        .get_settings()
        .expect("settings read should succeed");
    let connection_count = stored.connections.len();

    let added = post_form(
        &app,
        "/connections/add",
        "conn_name=Refused&conn_provider=mock&conn_model=mock-model&conn_api_key=&conn_base_url=&conn_max_context_tokens=wide",
    )
    .await;
    assert_eq!(
        added.status(),
        StatusCode::BAD_REQUEST,
        "a refused window must refuse the add too"
    );
    let after_add = state
        .settings_service
        .get_settings()
        .expect("settings read should succeed");
    assert_eq!(
        after_add.connections.len(),
        connection_count,
        "a refused window must save no connection"
    );

    let untouched = after_add
        .connections
        .iter()
        .find(|conn| conn.id == "alpha-conn")
        .expect("the edited connection must still be stored");
    assert_eq!(
        untouched.max_context_tokens, None,
        "a refused window must store nothing"
    );
}

fn app_with_one_connection_at_effort(
    reasoning_effort: Option<ReasoningEffort>,
) -> (axum::Router, AppState) {
    let mut connection = mock_connection("effort-conn", "mock-model");
    connection.name = "Effort".into();
    connection.reasoning_effort = reasoning_effort;
    let settings = AppSettings {
        connections: vec![connection],
        narration_connection_id: "effort-conn".into(),
        quantifier_connection_id: "effort-conn".into(),
        ..AppSettings::default()
    };
    TestAppBuilder::default_test()
        .settings(settings)
        .build_with_state()
}

fn stored_reasoning_effort(state: &AppState) -> Option<ReasoningEffort> {
    state
        .settings_service
        .get_settings()
        .expect("settings read should succeed")
        .find_connection("effort-conn")
        .expect("the connection must still be stored")
        .reasoning_effort
}

fn edit_form_body(reasoning_effort: &str) -> String {
    format!(
        "conn_name=Effort&conn_provider=mock&conn_model=mock-model&conn_api_key=&conn_base_url=&conn_reasoning_effort={reasoning_effort}"
    )
}

// [docs/specs/settings.md] SCENARIO: 20.29
#[tokio::test]
async fn test_reasoning_effort_field_stores_the_chosen_effort() {
    let _guard = SettingsTestGuard::new();
    let (app, state) = app_with_one_connection_at_effort(None);

    let response = post_form(
        &app,
        "/connections/effort-conn/edit",
        &edit_form_body("high"),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(stored_reasoning_effort(&state), Some(ReasoningEffort::High));
    let form = fetch_body(&app, "/fragment/connections/effort-conn/edit").await;
    assert!(
        form.contains(r#"<option value="high" selected>High</option>"#),
        "the edit form must select the stored effort: {form}"
    );
    assert!(
        !form.contains(r#"<option value="" selected>"#),
        "the provider default must not stay selected: {form}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.30
#[tokio::test]
async fn test_empty_reasoning_effort_field_stores_no_effort() {
    let _guard = SettingsTestGuard::new();
    let (app, state) = app_with_one_connection_at_effort(Some(ReasoningEffort::High));

    let response = post_form(&app, "/connections/effort-conn/edit", &edit_form_body("")).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(stored_reasoning_effort(&state), None);
    let form = fetch_body(&app, "/fragment/connections/effort-conn/edit").await;
    assert!(
        form.contains(r#"<option value="" selected>Provider default</option>"#),
        "the edit form must select the provider default: {form}"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.31
#[tokio::test]
async fn test_unknown_reasoning_effort_is_refused() {
    let _guard = SettingsTestGuard::new();
    let (app, state) = app_with_one_connection_at_effort(Some(ReasoningEffort::High));

    let response = post_form(
        &app,
        "/connections/effort-conn/edit",
        &edit_form_body("turbo"),
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_text(response).await;
    assert!(
        body.contains(
            "Reasoning effort must be one of the listed values, or empty for the provider default"
        ),
        "the refusal must name the reasoning effort rule: {body}"
    );
    assert_eq!(
        stored_reasoning_effort(&state),
        Some(ReasoningEffort::High),
        "a refused edit must store nothing"
    );

    let connection_count = state
        .settings_service
        .get_settings()
        .expect("settings read should succeed")
        .connections
        .len();
    let added = post_form(
        &app,
        "/connections/add",
        "conn_name=Refused&conn_provider=mock&conn_model=mock-model&conn_api_key=&conn_base_url=&conn_reasoning_effort=turbo",
    )
    .await;
    assert_eq!(added.status(), StatusCode::BAD_REQUEST);
    let added_body = body_text(added).await;
    assert!(
        added_body.contains(
            "Reasoning effort must be one of the listed values, or empty for the provider default"
        ),
        "the refused add must name the reasoning effort rule: {added_body}"
    );
    let after_add = state
        .settings_service
        .get_settings()
        .expect("settings read should succeed");
    assert_eq!(
        after_add.connections.len(),
        connection_count,
        "a refused reasoning effort must save no connection"
    );
}

// [docs/specs/settings.md] SCENARIO: 20.32
#[tokio::test]
async fn test_adding_a_connection_stores_the_chosen_reasoning_effort() {
    let _guard = SettingsTestGuard::new();
    let (app, state) = app_with_one_connection_at_effort(None);

    let response = post_form(
        &app,
        "/connections/add",
        "conn_name=Added&conn_provider=mock&conn_model=mock-model&conn_api_key=&conn_base_url=&conn_reasoning_effort=low",
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let stored = state
        .settings_service
        .get_settings()
        .expect("settings read should succeed");
    let added = stored
        .connections
        .iter()
        .find(|connection| connection.name == "Added")
        .expect("the added connection must be stored");
    assert_eq!(added.reasoning_effort, Some(ReasoningEffort::Low));
}

// [docs/specs/settings.md] SCENARIO: 20.33
#[tokio::test]
async fn test_form_connection_test_hands_the_typed_reasoning_effort_to_the_provider() {
    let _guard = SettingsTestGuard::new();
    let (state, _storage) = TestAppBuilder::default_test()
        .settings(single_mock_settings("saved-conn", "saved-model"))
        .build_service_with_storage();
    let tested: Arc<Mutex<Vec<LlmProviderConfig>>> = Arc::default();
    let capture = Arc::clone(&tested);
    let factory: ProviderFactory = Arc::new(move |config: &LlmProviderConfig| {
        capture.lock().unwrap().push(config.clone());
        Arc::new(MockBackend::new()) as Arc<dyn LlmProvider>
    });
    let app = build_router(state.clone().with_connection_test_provider(factory));

    let response = post_form(
        &app,
        "/connections/test",
        "conn_name=Typed&conn_provider=mock&conn_model=typed-model&conn_api_key=&conn_base_url=&conn_reasoning_effort=low",
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let tested = tested.lock().unwrap();
    assert_eq!(tested.len(), 1, "the test builds one provider");
    assert_eq!(tested[0].model, "typed-model");
    assert_eq!(tested[0].reasoning_effort, Some(ReasoningEffort::Low));
    let stored = state
        .settings_service
        .get_settings()
        .expect("settings read should succeed");
    assert_eq!(
        stored.connections.len(),
        1,
        "a form test must not save the typed connection"
    );
}
