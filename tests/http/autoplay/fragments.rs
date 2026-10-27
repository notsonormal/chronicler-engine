//! HTTP E2E tests for the view-only Game of Autoplay (autoplay.md 41.18-41.20, 41.22): the controls the story log, the dock and the Games panel render while a run holds the Game.

use std::sync::Arc;

use chronicler_engine::test_support::{active_game_of, analysis_report, ScriptedModel};

use super::{
    end_paused_run, pause_before_first_job, pause_between_two_jobs, FIRST_INPUT, FIRST_NARRATION,
    SECOND_NARRATION,
};
use crate::support::http_assertions::{assert_action_controls, ActionControls};
use crate::support::app_wiring::{app_after_an_impersonate_with_a_failed_event, app_with_script};
use crate::support::http_fixtures::{create_game_from, seed_offered_set, story_of_game};
use crate::support::http_requests::{fetch_body, post_action, post_empty, wait_settled};

fn button_tags_with_class(html: &str, class: &str) -> Vec<String> {
    html.split("<button")
        .skip(1)
        .filter_map(|rest| rest.split('>').next())
        .filter(|tag| tag.contains(class))
        .map(|tag| tag.to_string())
        .collect()
}

fn story_changing_controls(story_log: &str) -> Vec<(&'static str, Vec<String>)> {
    let swipe_buttons = |label: &str| -> Vec<String> {
        button_tags_with_class(story_log, "swipe-btn")
            .into_iter()
            .filter(|tag| tag.contains(&format!("aria-label=\"{label}\"")))
            .collect()
    };
    vec![
        ("Edit", button_tags_with_class(story_log, "edit-btn")),
        ("Delete", button_tags_with_class(story_log, "delete-btn")),
        ("Previous swipe", swipe_buttons("Previous swipe")),
        ("Retry", swipe_buttons("Retry")),
    ]
}

fn assert_controls(controls: &[(&'static str, Vec<String>)], disabled: bool, label: &str) {
    for (kind, tags) in controls {
        assert!(
            !tags.is_empty(),
            "{label}: the story log offers a {kind} control"
        );
        for tag in tags {
            assert_eq!(
                tag.contains(" disabled"),
                disabled,
                "{label}: {kind}: {tag}"
            );
        }
    }
}

/// The story of a run paused between two Jobs ends on the Impersonate Input, which has
/// one Swipe: its Previous swipe control is disabled whatever the run does.
fn between_jobs_controls(story_log: &str) -> Vec<(&'static str, Vec<String>)> {
    let mut controls = story_changing_controls(story_log);
    controls.retain(|(kind, _)| *kind != "Previous swipe");
    controls
}

fn assert_dock_controls(dock: &str, disabled: bool, label: &str) {
    let buttons = button_tags_with_class(dock, "option-btn");
    assert!(
        !buttons.is_empty(),
        "{label}: the dock offers options: {dock}"
    );
    for button in buttons {
        assert_eq!(button.contains(" disabled"), disabled, "{label}: {button}");
    }
}

// [docs/specs/autoplay.md] SCENARIO: 41.18
#[tokio::test]
async fn test_while_a_run_holds_the_game_the_story_log_renders_its_controls_disabled() {
    let model = Arc::new(
        ScriptedModel::new()
            .turn(FIRST_NARRATION)
            .narration(SECOND_NARRATION)
            .after_narration(analysis_report(&[], None)),
    );
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(wait_settled(&state, 2000).await, "the Job settles");
    assert!(post_empty(&app, "/swipe/new").await.status().is_success());
    assert!(wait_settled(&state, 2000).await, "the Retry settles");
    seed_offered_set(&state, &["Ask the bartender"]);

    let before = fetch_body(&app, "/fragment/story-log").await;
    assert_controls(&story_changing_controls(&before), false, "no run");
    assert_dock_controls(
        &fetch_body(&app, "/fragment/options-dock").await,
        false,
        "no run",
    );

    let pause = pause_before_first_job(&app, &state, game).await;
    let during = fetch_body(&app, "/fragment/story-log").await;
    assert_controls(
        &story_changing_controls(&during),
        true,
        "a run holds the Game",
    );
    assert_dock_controls(
        &fetch_body(&app, "/fragment/options-dock").await,
        true,
        "a run holds the Game",
    );
    assert!(
        during.contains(SECOND_NARRATION) && during.contains("look"),
        "the entries and their text are as before"
    );

    end_paused_run(&app, &state, game, &pause).await;
    let after = fetch_body(&app, "/fragment/story-log").await;
    assert_controls(&story_changing_controls(&after), false, "after the stop");
    assert_dock_controls(
        &fetch_body(&app, "/fragment/options-dock").await,
        false,
        "after the stop",
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.18
#[tokio::test]
async fn test_between_two_jobs_of_a_run_the_story_log_and_the_dock_render_disabled() {
    let model = Arc::new(
        ScriptedModel::new()
            .turn(FIRST_NARRATION)
            .narration(SECOND_NARRATION)
            .after_narration(analysis_report(&[], None))
            .turn(FIRST_INPUT),
    );
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(wait_settled(&state, 2000).await, "the Job settles");
    assert!(post_empty(&app, "/swipe/new").await.status().is_success());
    assert!(wait_settled(&state, 2000).await, "the Retry settles");
    seed_offered_set(&state, &["Ask the bartender"]);

    let before = fetch_body(&app, "/fragment/story-log").await;
    assert_controls(&story_changing_controls(&before), false, "no run");
    assert_dock_controls(
        &fetch_body(&app, "/fragment/options-dock").await,
        false,
        "no run",
    );

    let pause = pause_between_two_jobs(&app, &state, game).await;
    let during = fetch_body(&app, "/fragment/story-log").await;
    assert_controls(
        &between_jobs_controls(&during),
        true,
        "a run holds the Game",
    );
    assert_dock_controls(
        &fetch_body(&app, "/fragment/options-dock").await,
        true,
        "a run holds the Game",
    );
    assert!(
        during.contains(SECOND_NARRATION) && during.contains("look"),
        "the entries and their text are as before"
    );

    end_paused_run(&app, &state, game, &pause).await;
    let after = fetch_body(&app, "/fragment/story-log").await;
    assert_controls(&between_jobs_controls(&after), false, "after the stop");
    assert_dock_controls(
        &fetch_body(&app, "/fragment/options-dock").await,
        false,
        "after the stop",
    );
}

fn swipe_button(story_log: &str, label: &str) -> String {
    button_tags_with_class(story_log, "swipe-btn")
        .into_iter()
        .find(|tag| tag.contains(&format!("aria-label=\"{label}\"")))
        .unwrap_or_else(|| panic!("the story log offers {label}: {story_log}"))
}

// [docs/specs/autoplay.md] SCENARIO: 41.22
#[tokio::test]
async fn test_while_a_run_holds_the_game_retrigger_renders_disabled() {
    let (app, state) = app_after_an_impersonate_with_a_failed_event().await;
    let game = active_game_of(&state);
    let pause = pause_before_first_job(&app, &state, game).await;

    let during = button_tags_with_class(
        &fetch_body(&app, "/fragment/story-log").await,
        "retrigger-btn",
    );
    assert_eq!(
        during.len(),
        1,
        "the story log offers Retrigger: {during:?}"
    );
    assert!(during[0].contains(" disabled"), "Retrigger: {}", during[0]);

    end_paused_run(&app, &state, game, &pause).await;
    let after = button_tags_with_class(
        &fetch_body(&app, "/fragment/story-log").await,
        "retrigger-btn",
    );
    assert_eq!(after.len(), 1, "the story log offers Retrigger: {after:?}");
    assert!(
        !after[0].contains(" disabled"),
        "after the stop, Retrigger: {}",
        after[0]
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.22
#[tokio::test]
async fn test_while_a_run_holds_the_game_next_swipe_renders_disabled() {
    let model = Arc::new(
        ScriptedModel::new()
            .turn(FIRST_NARRATION)
            .narration(SECOND_NARRATION)
            .after_narration(analysis_report(&[], None)),
    );
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);
    assert!(post_action(&app, "look").await.status().is_success());
    assert!(wait_settled(&state, 2000).await, "the Job settles");
    assert!(post_empty(&app, "/swipe/new").await.status().is_success());
    assert!(wait_settled(&state, 2000).await, "the Retry settles");
    let narration = story_of_game(&state, game)
        .pop()
        .expect("the story ends on the Narration");
    assert!(
        post_empty(&app, &format!("/message/{}/swipe/0", narration.id))
            .await
            .status()
            .is_success()
    );

    let pause = pause_before_first_job(&app, &state, game).await;
    let during = swipe_button(&fetch_body(&app, "/fragment/story-log").await, "Next swipe");
    assert!(
        during.contains(" disabled"),
        "a run holds the Game: {during}"
    );

    end_paused_run(&app, &state, game, &pause).await;
    let after = swipe_button(&fetch_body(&app, "/fragment/story-log").await, "Next swipe");
    assert!(!after.contains(" disabled"), "after the stop: {after}");
}

fn item_of_game<'a>(games_panel: &'a str, game_name: &str) -> &'a str {
    let name = format!("<span class=\"game-name\">{game_name}</span>");
    let start = games_panel
        .find(&name)
        .unwrap_or_else(|| panic!("the Games panel lists {game_name}"));
    let end = games_panel[start..]
        .find("<div class=\"game-actions\">")
        .map(|offset| start + offset)
        .expect("the item has its actions");
    &games_panel[start..end]
}

// [docs/specs/autoplay.md] SCENARIO: 41.19
#[tokio::test]
async fn test_the_games_fragment_marks_a_game_that_a_run_holds() {
    let model = Arc::new(ScriptedModel::new());
    let (app, state, storage) = app_with_script(Arc::clone(&model));
    let game_a = active_game_of(&state);
    let name_a = state
        .game_view_query
        .game_display_name(game_a)
        .expect("Game A reads back");
    let pause_a = pause_before_first_job(&app, &state, game_a).await;
    let game_b = create_game_from(&app, &state, &storage, game_a, "test").await;
    let name_b = state
        .game_view_query
        .game_display_name(game_b)
        .expect("Game B reads back");
    assert_ne!(name_a, name_b, "the two Games have distinct names");

    let panel = fetch_body(&app, "/fragment/games").await;
    assert!(
        item_of_game(&panel, &name_a).contains("run-badge"),
        "the saved Game that a run holds carries the badge"
    );
    assert!(
        !item_of_game(&panel, &name_b).contains("run-badge"),
        "the active Game that no run holds carries no badge"
    );

    let pause_b = pause_before_first_job(&app, &state, game_b).await;
    let panel = fetch_body(&app, "/fragment/games").await;
    assert!(
        item_of_game(&panel, &name_b).contains("run-badge"),
        "the active Game that a run holds carries the badge"
    );

    // One wait covers the loops of both Games, so Game A's loop ends first.
    state
        .game_writer
        .stop(game_a)
        .expect("the stop of Game A lands");
    pause_a.release();
    end_paused_run(&app, &state, game_b, &pause_b).await;
    assert!(
        !state.game_view_query.is_busy(game_a),
        "the stopped run no longer holds Game A"
    );
}

fn assert_config_selects(games_panel: &str, disabled: bool, label: &str) {
    for id in crate::test_utils::GAME_CONFIG_SELECT_IDS {
        let marker = format!("id=\"{id}\"");
        let start = games_panel
            .find(&marker)
            .unwrap_or_else(|| panic!("{label}: the panel renders #{id}"));
        let tag = games_panel[start..]
            .split('>')
            .next()
            .expect("the select tag ends");
        assert_eq!(tag.contains(" disabled"), disabled, "{label}: #{id}");
    }
}

// [docs/specs/autoplay.md] SCENARIO: 41.20
#[tokio::test]
async fn test_while_a_run_holds_the_game_its_config_controls_render_disabled() {
    let model = Arc::new(ScriptedModel::new().turn(FIRST_INPUT));
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    let pause = pause_between_two_jobs(&app, &state, game).await;
    let during = fetch_body(&app, "/fragment/games").await;
    assert_config_selects(&during, true, "a run holds the Game");

    end_paused_run(&app, &state, game, &pause).await;
    let after = fetch_body(&app, "/fragment/games").await;
    assert_config_selects(&after, false, "after the stop");
}

async fn fetch_action_controls(app: &axum::Router) -> ActionControls {
    ActionControls {
        send: fetch_body(app, "/fragment/send-control").await,
        autoplay: fetch_body(app, "/fragment/autoplay-controls").await,
    }
}

// [docs/specs/autoplay.md] SCENARIO: 41.25
#[tokio::test]
async fn test_while_a_run_holds_the_game_the_action_controls_render_send_and_autoplay_disabled_and_stop_present()
 {
    let model = Arc::new(ScriptedModel::new().turn(FIRST_INPUT));
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    let pause = pause_between_two_jobs(&app, &state, game).await;
    let during = fetch_action_controls(&app).await;
    assert_action_controls(&during, true, true, true, "a run holds the Game");

    end_paused_run(&app, &state, game, &pause).await;
    let after = fetch_action_controls(&app).await;
    assert_action_controls(&after, false, false, false, "after the stop");
}
