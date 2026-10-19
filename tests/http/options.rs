//! HTTP E2E tests for options-autogeneration: on-demand /options, the always-on turn-end hook, and response-shape parsing.

use std::sync::Arc;

use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::test_support::active_game_of;
use chronicler_engine::domain::model::settings::{AppSettings, NarratorMode};
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::test_support::{
    analysis_report, configured_runner_without, ScriptedModel, TestAppBuilder, TestDataBuilder,
    TestMap, TestWorld,
};

use crate::support::app_wiring::{app_with_script, app_with_script_and_agents};
use crate::support::http_assertions::{defined_icons, referenced_icons};
use crate::support::http_fixtures::{if_world, seed_offered_set};
use crate::support::http_requests::{
    fetch_body, post_action, post_action_check, post_empty, post_form, wait_settled,
};
const SET_A: &str = "<suggestion>Search the desk</suggestion>\
<suggestion>Question the guard</suggestion><suggestion>Leave the hall</suggestion>";
const SET_B: &str = "<suggestion>Open the gate</suggestion>\
<suggestion>Feed the horse</suggestion><suggestion>Climb the tower</suggestion>";
const NUMBERED: &str = "1. Search the desk\n2. Question the guard\n3. Leave the hall";

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

async fn create_game(app: &axum::Router, world_key: &str) {
    let body = format!("world_key={world_key}&persona_key=test_player");
    let resp = post_form(app, "/games", &body).await;
    assert!(resp.status().is_success(), "game creation should succeed");
}

fn current_options(state: &AppState) -> Vec<String> {
    state
        .game_view_query
        .get_current_options(active_game_of(state))
        .expect("get_current_options should succeed")
}

fn message_count(state: &AppState) -> usize {
    state
        .game_view_query
        .story_messages(active_game_of(state))
        .unwrap()
        .len()
}

/// The dock's notice line, or `None` when the dock renders the offered set.
async fn dock_notice(app: &axum::Router) -> Option<String> {
    let dock = fetch_body(app, "/fragment/options-dock").await;
    dock.split("options-notice-line").nth(1).map(|rest| {
        let line = rest.split_once('>').map(|(_, after)| after).unwrap_or("");
        line.split('<').next().unwrap_or("").to_string()
    })
}

// [docs/specs/options.md] SCENARIO: 24.1
#[tokio::test]
async fn test_on_demand_options_render_in_dock_http() {
    let (app, state, _storage) =
        app_with_script(ScriptedModel::new().turn("You look around.").options(SET_A));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "setup turn should complete"
    );

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "/options should complete");

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
    // A World with no starting scenario writes an opening Narration with no text,
    // so the Game's story holds no text at all.
    let data = TestDataBuilder::default_test()
        .world(TestWorld::minimal())
        .map(TestMap::single_room("start"))
        .build();
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .model(Arc::new(ScriptedModel::new()))
        .build_with_state();

    let resp = post_action(&app, "/options").await;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("No scene to generate options for yet"),
        "the validation failure must be named: {body_str}"
    );
    assert!(wait_settled(&state, 1000).await);

    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        !dock.contains("options-strip"),
        "a rejected /options must render no options strip: {dock}"
    );
    assert_eq!(
        dock_notice(&app).await,
        None,
        "a rejected /options writes no notice: {dock}"
    );
    assert!(current_options(&state).is_empty());
}

// [docs/specs/options.md] SCENARIO: 24.4
#[tokio::test]
async fn test_regenerate_replaces_set_without_history_http() {
    let (app, state, _storage) = app_with_script(
        ScriptedModel::new()
            .turn("You look around.")
            .options(SET_A)
            .options(SET_B),
    );

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);
    let count_before = message_count(&state);

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(dock.contains("Search the desk"), "set A first: {dock}");

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

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
    let (app, state, _storage) = app_with_script(
        ScriptedModel::new()
            .turn("You look around.")
            .options(SET_A)
            .options("no parseable options in this response")
            .options("still nothing parseable here"),
    );

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);
    assert_eq!(current_options(&state).len(), 3, "set A installed");

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

    let options = current_options(&state);
    assert_eq!(
        options.len(),
        3,
        "the failed regeneration must keep the prior set"
    );
    assert!(options.contains(&"Search the desk".to_string()));

    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        !dock.contains("options-strip"),
        "the notice takes the strip's place: {dock}"
    );
    assert_eq!(
        dock_notice(&app).await.as_deref(),
        Some("Options generation failed"),
        "the dock must name the failure: {dock}"
    );

    let resp = post_empty(&app, "/status/reset-generating").await;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        dock.contains("Search the desk"),
        "the stop must give the prior set back: {dock}"
    );
}

// [docs/specs/options.md] SCENARIO: 24.18
#[tokio::test]
async fn test_on_demand_options_without_an_agent_reports_unavailable_http() {
    let (app, state, _storage) = app_with_script_and_agents(
        ScriptedModel::new().turn("You look around."),
        AppSettings::default(),
        |recorder| configured_runner_without(recorder, "options"),
    );

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "setup turn should complete"
    );
    seed_offered_set(&state, &["Inspect the ledger"]);
    assert_eq!(
        dock_notice(&app).await,
        None,
        "only /options may report the unavailable agent"
    );

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "/options should complete");

    assert_eq!(
        current_options(&state),
        vec!["Inspect the ledger".to_string()],
        "an unavailable options agent must keep the prior set"
    );
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        !dock.contains("options-strip"),
        "the notice takes the strip's place: {dock}"
    );
    assert_eq!(
        dock_notice(&app).await.as_deref(),
        Some("Options agent is not available"),
        "the dock must name the unavailable agent: {dock}"
    );

    let resp = post_empty(&app, "/status/reset-generating").await;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        dock.contains("Inspect the ledger"),
        "the stop must give the prior set back: {dock}"
    );
}

// [docs/specs/options.md] SCENARIO: 24.6
#[tokio::test]
async fn test_always_on_generates_after_narration_turn_http() {
    let (app, state, storage) =
        app_with_script(ScriptedModel::new().turn("You look around.").options(SET_A));
    storage
        .seed_world(&always_on_world(), &TestMap::single_room("start"))
        .expect("seed always-on world");

    create_game(&app, "always_on_world").await;

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
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
        dock.contains("Search the desk"),
        "the always-on turn-end set must render: {dock}"
    );
}

// Set B is next in the mock response queue: a regeneration would overwrite set A with it.
// [docs/specs/options.md] SCENARIO: 24.15
#[tokio::test]
async fn test_impersonate_keeps_offered_set_http() {
    let (app, state, storage) = app_with_script(
        ScriptedModel::new()
            .turn("You look around.")
            .options(SET_A)
            .options(SET_B)
            .narration("You say hello.")
            .analysis(analysis_report(&[], None)),
    );
    storage
        .seed_world(&always_on_world(), &TestMap::single_room("start"))
        .expect("seed always-on world");
    create_game(&app, "always_on_world").await;

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "setup turn should complete"
    );
    assert_eq!(
        current_options(&state),
        set_a_options(),
        "the setup must install set A"
    );

    let resp = post_action(&app, "/impersonate hello").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "impersonate should complete"
    );

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
    let (app, state, _storage) = app_with_script(
        ScriptedModel::new()
            .turn("You look around.")
            .options(SET_A)
            .narration("You look around again.")
            .analysis(analysis_report(&[], None)),
    );

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "setup turn should complete"
    );

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "/options should complete");
    assert_eq!(
        current_options(&state),
        set_a_options(),
        "the setup must install set A"
    );

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
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
    let (app, state, storage) = app_with_script(
        ScriptedModel::new()
            .turn("You look around.")
            .options(SET_A)
            .narration("You look around again.")
            .analysis(analysis_report(&[], None))
            .options("no parseable options in this response")
            .options("still no parseable options"),
    );
    storage
        .seed_world(&always_on_world(), &TestMap::single_room("start"))
        .expect("seed always-on world");
    create_game(&app, "always_on_world").await;

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "setup turn should complete"
    );
    assert_eq!(
        current_options(&state),
        set_a_options(),
        "the first always-on generation must install set A"
    );

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

    assert!(
        current_options(&state).is_empty(),
        "a failed regeneration must leave the offered set empty"
    );
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        !dock.contains("options-strip"),
        "a failed regeneration must render no options strip: {dock}"
    );
    assert_eq!(
        dock_notice(&app).await.as_deref(),
        Some("Options generation failed"),
        "the dock must name the failure: {dock}"
    );
}

// [docs/specs/options.md] SCENARIO: 24.20
#[tokio::test]
async fn test_always_on_without_an_agent_empties_set_and_reports_http() {
    let (app, state, storage) = app_with_script_and_agents(
        ScriptedModel::new().turn("You look around."),
        AppSettings::default(),
        |recorder| configured_runner_without(recorder, "options"),
    );
    storage
        .seed_world(&always_on_world(), &TestMap::single_room("start"))
        .expect("seed always-on world");
    create_game(&app, "always_on_world").await;
    seed_offered_set(&state, &["Inspect the ledger"]);

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "the message should complete"
    );

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
        dock_notice(&app).await.as_deref(),
        Some("Options agent is not available"),
        "the dock must name the unavailable agent once: {dock}"
    );
}

// [docs/specs/options.md] SCENARIO: 24.10
#[tokio::test]
async fn test_numbered_list_response_parses_http() {
    let (app, state, _storage) = app_with_script(
        ScriptedModel::new()
            .turn("You look around.")
            .options(NUMBERED),
    );

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

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
    let (app, state, storage) =
        app_with_script(ScriptedModel::new().turn("You look around.").options(SET_A));
    storage
        .seed_world(&if_world(), &TestMap::single_room("start"))
        .expect("seed IF world");
    create_game(&app, "if_world").await;

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await);

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
    let (app, state, _storage) = app_with_script(
        ScriptedModel::new()
            .turn("You look around.")
            .options(SET_A)
            .options(SET_B),
    );
    let game_id = active_game_of(&state);
    let starting_mode = state
        .game_catalogue
        .active_game()
        .expect("active_game should succeed")
        .expect("a current game must exist")
        .narrator_mode;
    assert_eq!(
        starting_mode,
        NarratorMode::Novel,
        "the game must start in Novel mode"
    );

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "setup turn should complete"
    );

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "/options should complete");
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
        .active_game()
        .expect("active_game should succeed")
        .expect("a current game must exist");
    assert_eq!(
        switched.narrator_mode,
        NarratorMode::InteractiveFiction,
        "the switch must actually change the game's narrator mode"
    );
    assert!(
        wait_settled(&state, 1000).await,
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
    let (app, state, _storage) = app_with_script(
        ScriptedModel::new()
            .turn("You look around.")
            .options(SET_A)
            .narration("You search the desk.")
            .analysis(analysis_report(&[], None)),
    );

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "setup turn should complete"
    );

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "/options should complete");

    let inputs_before = state
        .game_view_query
        .story_messages(active_game_of(&state))
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
        wait_settled(&state, 1000).await,
        "the submitted option should complete"
    );

    let messages = state
        .game_view_query
        .story_messages(active_game_of(&state))
        .unwrap();
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
    let (app, state, _storage) =
        app_with_script(ScriptedModel::new().turn("You look around.").options(SET_A));

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "setup turn should complete"
    );

    let resp = post_action(&app, "/options").await;
    assert!(resp.status().is_success());
    assert!(wait_settled(&state, 1000).await, "/options should complete");

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
