//! HTTP E2E tests for options-autogeneration: on-demand /options, the always-on turn-end hook, and response-shape parsing.

use std::sync::Arc;

use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::application::agents::options::OptionsAgent;
use chronicler_engine::application::agents::registry::AgentRegistry;
use chronicler_engine::application::ports::llm_provider::LlmProvider;
use chronicler_engine::domain::model::settings::AppSettings;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::test_support::TestMap;

use crate::test_helpers::{
    app_with_narrator_and_registry, fetch_body, if_world, post_action, post_form, wait_idle,
};

const SET_A: &str = "<suggestion>Search the desk</suggestion>\
<suggestion>Question the guard</suggestion><suggestion>Leave the hall</suggestion>";
const SET_B: &str = "<suggestion>Open the gate</suggestion>\
<suggestion>Feed the horse</suggestion><suggestion>Climb the tower</suggestion>";
const NUMBERED: &str = "1. Search the desk\n2. Question the guard\n3. Leave the hall";

/// App wired with an options agent whose backend is the given mock (the
/// narrator gets a default mock). Built on the default test data.
fn options_app(options: Arc<MockBackend>) -> (axum::Router, AppState, Arc<Storage>) {
    let mut registry = AgentRegistry::default();
    registry.add_agent(Box::new(OptionsAgent::with_provider(
        "options".to_string(),
        options as Arc<dyn LlmProvider>,
    )));
    app_with_narrator_and_registry(
        Arc::new(MockBackend::default()),
        registry,
        AppSettings::default(),
    )
}

fn tagged_provider(responses: Vec<String>) -> Arc<MockBackend> {
    Arc::new(MockBackend::new().with_prompt_responses(responses))
}

fn always_on_world() -> WorldCard {
    WorldCard {
        key: "always_on_world".to_string(),
        name: "Always On World".to_string(),
        description: "A world with options always on.".to_string(),
        options_always_on: true,
        ..Default::default()
    }
}

/// POST /games for the given world key, asserting success.
async fn create_game(app: &axum::Router, world_key: &str) {
    let body = format!("world_key={world_key}&persona_key=test_player");
    let resp = post_form(app, "/games", &body).await;
    assert!(resp.status().is_success(), "game creation should succeed");
}

fn current_options(state: &AppState) -> Vec<String> {
    state
        .message_service
        .load_or_fresh()
        .narrative
        .current_options
}

fn message_count(state: &AppState) -> usize {
    state.message_service.load_messages().unwrap().len()
}

fn system_messages(state: &AppState) -> Vec<String> {
    state
        .message_service
        .load_messages()
        .unwrap()
        .into_iter()
        .filter(|m| m.message_type == MessageType::System)
        .map(|m| m.text().to_string())
        .collect()
}

// [docs/specs/options.md] SCENARIO: 24.1
#[tokio::test]
async fn test_on_demand_options_render_in_dock_http() {
    let (app, state, _storage) = options_app(tagged_provider(vec![SET_A.to_string()]));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "setup turn should complete");

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "/options should complete");

    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        dock.contains("options-strip"),
        "the dock must render: {dock}"
    );
    assert!(dock.contains("Search the desk"), "{dock}");
    assert!(dock.contains("Question the guard"), "{dock}");
    assert!(dock.contains("Leave the hall"), "{dock}");
    assert_eq!(current_options(&state).len(), 3);
}

// [docs/specs/options.md] SCENARIO: 24.2
#[tokio::test]
async fn test_options_without_scene_history_errors_http() {
    let (app, state, _storage) = options_app(tagged_provider(vec![SET_A.to_string()]));

    let resp = post_action(&app, "/options").await;
    assert_eq!(resp.status(), axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("No scene to generate options for yet"),
        "the validation failure must be named: {body_str}"
    );
    assert!(wait_idle(&state, 1000).await);

    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        !dock.contains("options-strip"),
        "a rejected /options must leave the dock empty: {dock}"
    );
    assert!(current_options(&state).is_empty());
}

// [docs/specs/options.md] SCENARIO: 24.4
#[tokio::test]
async fn test_regenerate_replaces_set_without_history_http() {
    let (app, state, _storage) =
        options_app(tagged_provider(vec![SET_A.to_string(), SET_B.to_string()]));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);
    let count_before = message_count(&state);

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(dock.contains("Search the desk"), "set A first: {dock}");

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(dock.contains("Open the gate"), "set B second: {dock}");
    assert!(
        !dock.contains("Search the desk"),
        "set A must be gone after regeneration: {dock}"
    );
    assert_eq!(
        message_count(&state),
        count_before,
        "/options calls must not add history entries"
    );
}

// [docs/specs/options.md] SCENARIO: 24.5
#[tokio::test]
async fn test_failed_regeneration_keeps_prior_set_http() {
    let (app, state, _storage) = options_app(tagged_provider(vec![
        SET_A.to_string(),
        "no parseable options in this response".to_string(),
        "still nothing parseable here".to_string(),
    ]));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);
    assert_eq!(current_options(&state).len(), 3, "set A installed");

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let options = current_options(&state);
    assert_eq!(
        options.len(),
        3,
        "the failed regeneration must keep the prior set"
    );
    assert!(options.contains(&"Search the desk".to_string()));

    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        dock.contains("Search the desk"),
        "the dock must still render the prior set: {dock}"
    );
    let systems = system_messages(&state);
    assert!(
        systems
            .iter()
            .any(|m| m.contains("Options generation failed")),
        "the failure must surface a system message: {systems:?}"
    );
}

// [docs/specs/options.md] SCENARIO: 24.6
#[tokio::test]
async fn test_always_on_generates_after_narration_turn_http() {
    let (app, state, storage) = options_app(Arc::new(MockBackend::default()));
    storage
        .seed_world(&always_on_world(), &TestMap::single_room("start"))
        .expect("seed always-on world");

    create_game(&app, "always_on_world").await;

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_idle(&state, 1000).await,
        "narration turn should complete"
    );

    let options = current_options(&state);
    assert_eq!(
        options.len(),
        3,
        "the always-on toggle must generate options at turn end"
    );

    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        dock.contains("options-strip"),
        "the dock must render: {dock}"
    );
    assert!(
        dock.contains("Option one from the mock."),
        "the mock's canned options must render: {dock}"
    );
}

// [docs/specs/options.md] SCENARIO: 24.7
#[tokio::test]
async fn test_always_on_skips_impersonate_turn_http() {
    let (app, state, storage) = options_app(Arc::new(MockBackend::default()));
    storage
        .seed_world(&always_on_world(), &TestMap::single_room("start"))
        .expect("seed always-on world");
    create_game(&app, "always_on_world").await;

    let resp = post_action(&app, "/impersonate hello").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "impersonate should complete");

    assert!(
        current_options(&state).is_empty(),
        "an impersonate turn must not trigger always-on options"
    );
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        !dock.contains("options-strip"),
        "the dock must stay empty after an impersonate turn: {dock}"
    );
}

// [docs/specs/options.md] SCENARIO: 24.12
#[tokio::test]
async fn test_always_on_disabled_does_not_generate_after_narration_turn_http() {
    let (app, state, _storage) = options_app(Arc::new(MockBackend::default()));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_idle(&state, 1000).await,
        "narration turn should complete"
    );

    assert!(
        current_options(&state).is_empty(),
        "the toggle-off game must not generate options at turn end"
    );
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        !dock.contains("options-strip"),
        "the dock must stay empty when the toggle is off: {dock}"
    );
}

// [docs/specs/options.md] SCENARIO: 24.10
#[tokio::test]
async fn test_numbered_list_response_parses_http() {
    let (app, state, _storage) = options_app(tagged_provider(vec![NUMBERED.to_string()]));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(dock.contains("Search the desk"), "{dock}");
    assert!(dock.contains("Question the guard"), "{dock}");
    assert!(
        !dock.contains("1. Search the desk"),
        "the list numbering must be stripped: {dock}"
    );
}

// [docs/specs/options.md] SCENARIO: 24.11
#[tokio::test]
async fn test_options_work_in_if_mode_http() {
    let (app, state, storage) = options_app(tagged_provider(vec![SET_A.to_string()]));
    storage
        .seed_world(&if_world(), &TestMap::single_room("start"))
        .expect("seed IF world");
    create_game(&app, "if_world").await;

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        dock.contains("options-strip"),
        "the dock must render: {dock}"
    );
    assert!(dock.contains("Search the desk"), "{dock}");
}
