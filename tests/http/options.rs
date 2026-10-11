//! HTTP E2E tests for options-autogeneration: on-demand /options, the always-on turn-end hook, and response-shape parsing.

use std::sync::Arc;

use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::application::agents::options::OptionsAgent;
use chronicler_engine::application::agents::registry::AgentRegistry;
use chronicler_engine::application::ports::llm_provider::LlmProvider;
use chronicler_engine::domain::model::settings::{AppSettings, NarratorMode};
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::test_support::TestMap;

use crate::support::app_wiring::app_with_narrator_and_registry;
use crate::support::http_assertions::{defined_icons, referenced_icons};
use crate::support::http_fixtures::{if_world, seed_offered_set};
use crate::support::http_requests::{fetch_body, post_action, post_action_check, post_form, wait_idle};

const SET_A: &str = "<suggestion>Search the desk</suggestion>\
<suggestion>Question the guard</suggestion><suggestion>Leave the hall</suggestion>";
const SET_B: &str = "<suggestion>Open the gate</suggestion>\
<suggestion>Feed the horse</suggestion><suggestion>Climb the tower</suggestion>";
const NUMBERED: &str = "1. Search the desk\n2. Question the guard\n3. Leave the hall";

/// The narrator gets a default mock; everything else is the default test data.
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

fn set_a_options() -> Vec<String> {
    vec![
        "Search the desk".to_string(),
        "Question the guard".to_string(),
        "Leave the hall".to_string(),
    ]
}

fn app_without_options_agent() -> (axum::Router, AppState, Arc<Storage>) {
    app_with_narrator_and_registry(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
        AppSettings::default(),
    )
}

async fn create_game(app: &axum::Router, world_key: &str) {
    let body = format!("world_key={world_key}&persona_key=test_player");
    let resp = post_form(app, "/games", &body).await;
    assert!(resp.status().is_success(), "game creation should succeed");
}

fn current_options(state: &AppState) -> Vec<String> {
    state
        .game_view_query
        .get_current_options()
        .expect("get_current_options should succeed")
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

fn unavailable_agent_reports(state: &AppState) -> usize {
    system_messages(state)
        .iter()
        .filter(|m| m.contains("Options agent is not available"))
        .count()
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

// [docs/specs/options.md] SCENARIO: 24.18
#[tokio::test]
async fn test_on_demand_options_without_an_agent_reports_unavailable_http() {
    let (app, state, _storage) = app_without_options_agent();

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "setup turn should complete");
    seed_offered_set(&state, &["Inspect the ledger"]);
    let unavailable_before = unavailable_agent_reports(&state);
    assert_eq!(
        unavailable_before, 0,
        "only /options may report the unavailable agent"
    );

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "/options should complete");

    assert_eq!(
        current_options(&state),
        vec!["Inspect the ledger".to_string()],
        "an unavailable options agent must keep the prior set"
    );
    assert_eq!(
        unavailable_agent_reports(&state),
        1,
        "the unavailable agent must be reported once in the story log"
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

// Set B is next in the mock response queue: a regeneration would overwrite set A with it.
// [docs/specs/options.md] SCENARIO: 24.15
#[tokio::test]
async fn test_impersonate_keeps_offered_set_http() {
    let (app, state, storage) =
        options_app(tagged_provider(vec![SET_A.to_string(), SET_B.to_string()]));
    storage
        .seed_world(&always_on_world(), &TestMap::single_room("start"))
        .expect("seed always-on world");
    create_game(&app, "always_on_world").await;

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "setup turn should complete");
    assert_eq!(
        current_options(&state),
        set_a_options(),
        "the setup must install set A"
    );

    let resp = post_action(&app, "/impersonate hello").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "impersonate should complete");

    assert_eq!(
        current_options(&state),
        set_a_options(),
        "an impersonate message must leave the offered set alone"
    );
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        dock.contains("Search the desk"),
        "the dock must still render set A: {dock}"
    );
}

// [docs/specs/options.md] SCENARIO: 24.16
#[tokio::test]
async fn test_narration_turn_clears_offered_set_http() {
    let (app, state, _storage) = options_app(tagged_provider(vec![SET_A.to_string()]));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "setup turn should complete");

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "/options should complete");
    assert_eq!(
        current_options(&state),
        set_a_options(),
        "the setup must install set A"
    );

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_idle(&state, 1000).await,
        "the narration turn should complete"
    );

    assert!(
        current_options(&state).is_empty(),
        "the turn end must clear the offered set when the toggle is off"
    );
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        !dock.contains("options-strip"),
        "a cleared set must leave the dock empty: {dock}"
    );
}

// [docs/specs/options.md] SCENARIO: 24.17
#[tokio::test]
async fn test_failed_always_on_regeneration_clears_set_and_reports_http() {
    // Zero-parse answers get one retry, so this needs two unparseable responses to fail.
    let (app, state, storage) = options_app(tagged_provider(vec![
        SET_A.to_string(),
        "no parseable options in this response".to_string(),
        "still no parseable options".to_string(),
    ]));
    storage
        .seed_world(&always_on_world(), &TestMap::single_room("start"))
        .expect("seed always-on world");
    create_game(&app, "always_on_world").await;

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "setup turn should complete");
    assert_eq!(
        current_options(&state),
        set_a_options(),
        "the first always-on generation must install set A"
    );

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    assert!(
        current_options(&state).is_empty(),
        "a failed regeneration must leave the offered set empty"
    );
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        !dock.contains("options-strip"),
        "a failed regeneration must leave the dock empty: {dock}"
    );
    let systems = system_messages(&state);
    assert!(
        systems
            .iter()
            .any(|m| m.contains("Options generation failed")),
        "the failed regeneration must surface a system message: {systems:?}"
    );
}

// [docs/specs/options.md] SCENARIO: 24.20
#[tokio::test]
async fn test_always_on_without_an_agent_empties_set_and_reports_http() {
    let (app, state, storage) = app_without_options_agent();
    storage
        .seed_world(&always_on_world(), &TestMap::single_room("start"))
        .expect("seed always-on world");
    create_game(&app, "always_on_world").await;
    seed_offered_set(&state, &["Inspect the ledger"]);

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "the message should complete");

    assert!(
        current_options(&state).is_empty(),
        "the turn end must clear the offered set even when no agent can regenerate it"
    );
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        !dock.contains("options-strip"),
        "with no agent the dock must render no options strip: {dock}"
    );
    assert_eq!(
        unavailable_agent_reports(&state),
        1,
        "the unavailable agent must be reported once in the story log"
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

// Set B is next in the mock response queue: a regeneration would overwrite set A with it.
// [docs/specs/options.md] SCENARIO: 24.19
#[tokio::test]
async fn test_mode_switch_keeps_offered_set_http() {
    let (app, state, storage) =
        options_app(tagged_provider(vec![SET_A.to_string(), SET_B.to_string()]));
    let game_id = storage.current_game_id();
    let starting_mode = state
        .game_catalogue
        .current_game()
        .expect("current_game should succeed")
        .expect("a current game must exist")
        .narrator_mode;
    assert_eq!(
        starting_mode,
        NarratorMode::Novel,
        "the game must start in Novel mode"
    );

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "setup turn should complete");

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "/options should complete");
    assert_eq!(
        current_options(&state),
        set_a_options(),
        "the setup must install set A"
    );

    let resp = post_form(
        &app,
        &format!("/games/{game_id}/mode"),
        "narrator_mode=interactive_fiction",
    )
    .await;
    assert!(resp.status().is_success(), "mode switch should succeed");
    let switched = state
        .game_catalogue
        .current_game()
        .expect("current_game should succeed")
        .expect("a current game must exist");
    assert_eq!(
        switched.narrator_mode,
        NarratorMode::InteractiveFiction,
        "the switch must actually change the game's narrator mode"
    );
    assert!(
        wait_idle(&state, 1000).await,
        "the mode switch must leave no generation running"
    );

    assert_eq!(
        current_options(&state),
        set_a_options(),
        "a narrator-mode switch must leave the offered set alone"
    );
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        dock.contains("Search the desk"),
        "the dock must still render set A: {dock}"
    );
}

// `useOption` fills the command input with the option's text and submits
// `/action/check` — the same request this test makes, so the persisted message
// store is the observable outcome of the click.
// [docs/specs/options.md] SCENARIO: 24.13
#[tokio::test]
async fn test_using_offered_option_submits_as_input_http() {
    let (app, state, _storage) = options_app(tagged_provider(vec![SET_A.to_string()]));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "setup turn should complete");

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "/options should complete");

    let inputs_before = state
        .message_service
        .load_messages()
        .unwrap()
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .count();

    let resp = post_action_check(&app, "Search the desk").await;
    assert!(
        resp.status().is_success(),
        "the option submit should accept"
    );
    assert!(
        wait_idle(&state, 1000).await,
        "the submitted option should complete"
    );

    let messages = state.message_service.load_messages().unwrap();
    let inputs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert_eq!(
        inputs.len(),
        inputs_before + 1,
        "using an option must persist exactly one more Input entry"
    );
    let newest = inputs.last().expect("the new Input entry");
    assert_eq!(
        newest.text(),
        "Search the desk",
        "the Input entry must carry the option's text"
    );

    let narrations = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .count();
    assert!(
        narrations > 0,
        "the submitted option must narrate; messages were: {messages:?}"
    );
}

// [docs/specs/options.md] SCENARIO: 24.14
#[tokio::test]
async fn test_options_dock_icons_are_defined_by_the_shell() {
    let (app, state, _storage) = options_app(tagged_provider(vec![SET_A.to_string()]));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "setup turn should complete");

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "/options should complete");

    let defined = defined_icons(&fetch_body(&app, "/").await);
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    let icons = referenced_icons(&dock);
    assert!(
        !icons.is_empty(),
        "the populated dock must show its controls' icons: {dock}"
    );
    for icon in icons {
        assert!(
            defined.contains(&icon),
            "the options dock references icon {icon}, which the shell does not define"
        );
    }
}
