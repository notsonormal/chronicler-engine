//! HTTP E2E tests for Autoplay (autoplay.md 41.*): the run of Jobs, its three ends, the view-only Game, the progress status, and the config writes a run refuses.

use std::sync::Arc;

use axum::http::StatusCode;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::domain::model::game::GameId;
use chronicler_engine::domain::model::message::Message;
use chronicler_engine::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::trigger::ComparisonOperator;
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::test_support::{
    active_game_of, analysis_report, ScriptedModel, TestMap, TestNpc, TestPromptPreset,
};

use crate::support::app_wiring::{
    app_after_an_impersonate_with_a_failed_event, app_with_a_triggered_npc, app_with_script,
};
use crate::support::http_assertions::assert_still_thinking;
use crate::support::http_fixtures::seed_offered_set;
use crate::support::http_requests::{
    assert_no_unscripted_model_call, fetch_body, fetch_generating_status, post_action, post_empty,
    post_form, response_body, wait_for_jobs, wait_for_jobs_running_at_most, wait_settled,
};

const FIRST_INPUT: &str = "I try the door.";
const FIRST_NARRATION: &str = "The door creaks open.";
const SECOND_INPUT: &str = "I step inside.";
const SECOND_NARRATION: &str = "A cold hall waits.";
const LATE_ANSWER: &str = "The late answer.";

async fn start_autoplay(app: &axum::Router, body: &str) -> axum::response::Response {
    post_form(app, "/autoplay", body).await
}

/// Waits for the task of the whole run, so a read after it sees every write the run
/// made.
async fn wait_for_run(state: &AppState, game: GameId) {
    assert!(wait_for_jobs(state, 5000).await, "the run should end");
    assert!(
        !state.game_view_query.is_busy(game),
        "an ended run leaves its Game idle"
    );
    assert_no_unscripted_model_call(state);
}

fn messages_of(state: &AppState, game: GameId) -> Vec<Message> {
    state
        .game_view_query
        .story_messages(game)
        .expect("the story log reads back")
}

fn story_after_opening(state: &AppState, game: GameId) -> Vec<(MessageType, String)> {
    messages_of(state, game)
        .into_iter()
        .skip(1)
        .map(|message| {
            let text = message.text().to_string();
            (message.message_type, text)
        })
        .collect()
}

fn narrator_calls(model: &ScriptedModel) -> Vec<chronicler_engine::test_support::ModelCall> {
    model
        .calls()
        .into_iter()
        .filter(|call| call.role == Role::Narrator)
        .collect()
}

/// Creates a second Game from the World and Persona of `first`. The create makes
/// it the active Game.
async fn create_game_from(
    app: &axum::Router,
    state: &AppState,
    storage: &Storage,
    first: GameId,
) -> GameId {
    let persona_key = storage
        .require_game(first.0)
        .expect("the first Game reads back")
        .persona_key;
    let created = post_form(
        app,
        "/games",
        &format!("world_key=test&persona_key={persona_key}"),
    )
    .await;
    assert!(
        created.status().is_success(),
        "the second Game should be created"
    );
    let second = active_game_of(state);
    assert_ne!(second, first, "the create makes the second Game active");
    second
}

async fn assert_status(app: &axum::Router, expected: &str, label: &str) {
    assert_eq!(
        fetch_generating_status(app).await.trim(),
        expected,
        "{label}"
    );
}

async fn stop(app: &axum::Router) {
    let answered = post_empty(app, "/status/reset-generating").await;
    assert_eq!(
        response_body(answered).await,
        "reset",
        "a stop of a held Game answers reset"
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.1
#[tokio::test]
async fn test_a_run_on_a_fresh_game_plays_its_cycles_and_the_game_is_then_idle() {
    let model = Arc::new(
        ScriptedModel::new()
            .turn(FIRST_INPUT)
            .turn(FIRST_NARRATION)
            .turn(SECOND_INPUT)
            .turn(SECOND_NARRATION),
    );
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    let started = start_autoplay(&app, "cycles=2").await;
    assert!(started.status().is_success(), "the start should accept");
    assert!(
        response_body(started).await.contains("Thinking..."),
        "the start answers with the thinking status"
    );
    wait_for_run(&state, game).await;

    assert_eq!(
        story_after_opening(&state, game),
        vec![
            (MessageType::Input, FIRST_INPUT.to_string()),
            (MessageType::Narration, FIRST_NARRATION.to_string()),
            (MessageType::Input, SECOND_INPUT.to_string()),
            (MessageType::Narration, SECOND_NARRATION.to_string()),
        ],
        "the run writes an Impersonate Input and a Narration, twice"
    );
    assert_status(&app, "idle", "the Game is idle after the run").await;

    let calls = narrator_calls(&model);
    assert_eq!(calls.len(), 4, "two cycles make four narrator calls");
    for (index, input) in [(1, FIRST_INPUT), (3, SECOND_INPUT)] {
        let prompt = format!("{}{}", calls[index].system_prompt, calls[index].user_prompt);
        assert!(
            prompt.contains(&format!("<PlayerInput>\n{input}\n</PlayerInput>")),
            "the Continue Job answers the Impersonate Input {input:?}, got: {prompt}"
        );
    }
}

// [docs/specs/autoplay.md] SCENARIO: 41.2
#[tokio::test]
async fn test_a_run_whose_last_message_is_an_unanswered_input_starts_with_a_continue() {
    let model = Arc::new(ScriptedModel::new().turn(FIRST_INPUT).turn(FIRST_NARRATION));
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    assert!(
        post_action(&app, "/impersonate")
            .await
            .status()
            .is_success()
    );
    assert!(wait_settled(&state, 2000).await, "the Impersonate Job ends");
    assert_eq!(
        story_after_opening(&state, game),
        vec![(MessageType::Input, FIRST_INPUT.to_string())],
        "the setup leaves an Input that no Narration answers"
    );

    let started = start_autoplay(&app, "cycles=1").await;
    assert!(started.status().is_success(), "the start should accept");
    wait_for_run(&state, game).await;

    assert_eq!(
        story_after_opening(&state, game),
        vec![
            (MessageType::Input, FIRST_INPUT.to_string()),
            (MessageType::Narration, FIRST_NARRATION.to_string()),
        ],
        "the run's first Job is a Continue, and that Continue is the one cycle of its budget"
    );
    assert_eq!(
        narrator_calls(&model).len(),
        2,
        "the setup made one narrator call and the run one: no Impersonate call"
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.3
#[tokio::test]
async fn test_a_trigger_in_a_continue_job_adds_its_event_narration_to_the_cycle() {
    const EVENT_NARRATION: &str = "The NPC greets you warmly.";
    // The Trigger is scoped to the second room, which only the Continue Job's
    // analysis moves the player into.
    let npc =
        TestNpc::with_room_scoped_trigger("npc_1", "Test NPC", ComparisonOperator::Eq, 0, "room2");
    let model = ScriptedModel::new()
        .narration(FIRST_INPUT)
        .after_narration(analysis_report(&[], None))
        .narration(FIRST_NARRATION)
        .after_narration(analysis_report(&["npc_1"], Some("room2")))
        .event_narration(EVENT_NARRATION)
        .after_narration(analysis_report(&["npc_1"], None))
        .narration(SECOND_INPUT)
        .after_narration(analysis_report(&[], None))
        .narration(SECOND_NARRATION)
        .after_narration(analysis_report(&[], None));
    let (app, state) = app_with_a_triggered_npc(TestMap::two_rooms("room_1", "room2"), npc, model);
    let game = active_game_of(&state);

    let started = start_autoplay(&app, "cycles=2").await;
    assert!(started.status().is_success(), "the start should accept");
    wait_for_run(&state, game).await;

    let story = messages_of(&state, game);
    let texts: Vec<&str> = story.iter().skip(1).map(Message::text).collect();
    assert_eq!(
        texts,
        vec![
            FIRST_INPUT,
            FIRST_NARRATION,
            EVENT_NARRATION,
            SECOND_INPUT,
            SECOND_NARRATION
        ],
        "the event Narration is part of the first cycle: the second cycle follows it, \
         and the budget of 2 counts the answered Inputs"
    );
    assert!(
        story
            .iter()
            .skip(1)
            .nth(2)
            .and_then(Message::event_header)
            .is_some(),
        "the third Message is the event Narration"
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.21
#[tokio::test]
async fn test_a_trigger_in_an_impersonate_job_answers_the_input_and_ends_the_cycle() {
    const EVENT_NARRATION: &str = "The NPC greets you warmly.";
    let npc =
        TestNpc::with_room_scoped_trigger("npc_1", "Test NPC", ComparisonOperator::Eq, 0, "room2");
    let model = ScriptedModel::new()
        .narration(FIRST_INPUT)
        .after_narration(analysis_report(&["npc_1"], Some("room2")))
        .event_narration(EVENT_NARRATION)
        .after_narration(analysis_report(&["npc_1"], None))
        .narration(SECOND_INPUT)
        .after_narration(analysis_report(&[], None))
        .narration(SECOND_NARRATION)
        .after_narration(analysis_report(&[], None));
    let (app, state) = app_with_a_triggered_npc(TestMap::two_rooms("room_1", "room2"), npc, model);
    let game = active_game_of(&state);

    let started = start_autoplay(&app, "cycles=2").await;
    assert!(started.status().is_success(), "the start should accept");
    wait_for_run(&state, game).await;

    assert_eq!(
        story_after_opening(&state, game),
        vec![
            (MessageType::Input, FIRST_INPUT.to_string()),
            (MessageType::Narration, EVENT_NARRATION.to_string()),
            (MessageType::Input, SECOND_INPUT.to_string()),
            (MessageType::Narration, SECOND_NARRATION.to_string()),
        ],
        "the event Narration answers the first Input and ends the first cycle: \
         three Jobs run, and an unscripted fourth call would fail the run"
    );
    assert_status(&app, "idle", "the Game is idle after the run").await;
}

// [docs/specs/autoplay.md] SCENARIO: 41.4
#[tokio::test]
async fn test_a_failed_narrator_call_ends_the_run_and_its_failure_line_stays() {
    let model = Arc::new(
        ScriptedModel::new()
            .turn(FIRST_INPUT)
            .failed(Role::Narrator),
    );
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    let started = start_autoplay(&app, "cycles=3").await;
    assert!(started.status().is_success(), "the start should accept");
    wait_for_run(&state, game).await;

    assert_eq!(
        story_after_opening(&state, game),
        vec![(MessageType::Input, FIRST_INPUT.to_string())],
        "the Impersonate Input stays, and the failed Continue saved no Narration"
    );
    let status = fetch_generating_status(&app).await;
    assert!(
        status.contains("status-error-popover") && status.contains("answer could not be read"),
        "the status poll shows the failure line of the Job, got: {status}"
    );
    assert_eq!(
        narrator_calls(&model).len(),
        2,
        "no Job started after the failed one: its budget of 3 does not matter"
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.5
#[tokio::test]
async fn test_a_failed_options_run_writes_its_notice_and_the_run_goes_on() {
    const UNPARSEABLE: &str = "no parseable options in this response";
    let model = Arc::new(
        ScriptedModel::new()
            .turn(FIRST_INPUT)
            .turn(FIRST_NARRATION)
            .options(UNPARSEABLE)
            .options(UNPARSEABLE)
            .turn(SECOND_INPUT)
            .turn(SECOND_NARRATION)
            .options(UNPARSEABLE)
            .options(UNPARSEABLE),
    );
    let (app, state, storage) = app_with_script(Arc::clone(&model));
    storage
        .seed_world(&always_on_world(), &TestMap::single_room("start"))
        .expect("the always-on World seeds");
    let created = post_form(
        &app,
        "/games",
        "world_key=always_on_world&persona_key=test_player",
    )
    .await;
    assert!(created.status().is_success(), "the Game should be created");
    let game = active_game_of(&state);

    let started = start_autoplay(&app, "cycles=2").await;
    assert!(started.status().is_success(), "the start should accept");
    wait_for_run(&state, game).await;

    let story = story_after_opening(&state, game);
    assert_eq!(
        story
            .iter()
            .map(|(_, text)| text.as_str())
            .collect::<Vec<_>>(),
        vec![FIRST_INPUT, FIRST_NARRATION, SECOND_INPUT, SECOND_NARRATION],
        "a failed Options run does not stop the run: both cycles are in the story"
    );
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        dock.contains("options-notice-line") && dock.contains("Options generation failed"),
        "the dock shows the Notice of the failed Options run, got: {dock}"
    );
    assert_status(&app, "idle", "a failed Options run leaves no failure line").await;
}

fn always_on_world() -> WorldCard {
    WorldCard {
        key: "always_on_world".to_string(),
        name: "Always On World".to_string(),
        description: "A World with options always on.".to_string(),
        options_always_on: true,
        ..Default::default()
    }
}

// [docs/specs/autoplay.md] SCENARIO: 41.6
#[tokio::test]
async fn test_a_stop_during_a_job_of_a_run_ends_the_run() {
    let model = Arc::new(ScriptedModel::new().turn(LATE_ANSWER).turn(FIRST_NARRATION));
    let hold = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);
    let before = messages_of(&state, game);

    assert!(start_autoplay(&app, "cycles=2").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the first Job of the run should reach its narrator call"
    );

    stop(&app).await;
    assert_status(&app, "idle", "a stopped run leaves the Game idle at once").await;

    hold.release();
    assert!(hold.wait_taken(1000), "the stopped call takes its answer");
    wait_for_run(&state, game).await;

    assert_eq!(
        messages_of(&state, game),
        before,
        "the late answer of the stopped Job changes nothing"
    );
    assert_eq!(
        narrator_calls(&model).len(),
        1,
        "no further Job of the run started"
    );

    assert!(
        post_action(&app, "look").await.status().is_success(),
        "the Game accepts a Job after the stop"
    );
    assert!(wait_settled(&state, 2000).await, "that Job settles");
    assert_eq!(
        story_after_opening(&state, game),
        vec![
            (MessageType::Input, "look".to_string()),
            (MessageType::Narration, FIRST_NARRATION.to_string()),
        ],
        "the next Job saves its own text"
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.7
#[tokio::test]
async fn test_a_stop_between_two_jobs_of_a_run_ends_the_run() {
    let model = Arc::new(ScriptedModel::new().turn(FIRST_NARRATION));
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);
    let before = messages_of(&state, game);

    // No model call lies between two Jobs of a run, so `ScriptedModel::hold` cannot
    // stop a live driver there. A run hold with no driver is that state: no job
    // entry, and the run hold stays.
    state
        .game_writer
        .start_run(game, 2)
        .expect("a run starts on an idle Game");
    assert_status(
        &app,
        "autoplay 1/2",
        "the run holds the Game between its Jobs",
    )
    .await;

    stop(&app).await;

    assert_status(&app, "idle", "a stopped run leaves the Game idle").await;
    assert!(!state.game_view_query.is_busy(game), "the Game is not busy");
    assert_eq!(messages_of(&state, game), before, "the story is as it was");
    assert!(model.calls().is_empty(), "no model call is made");

    assert!(
        post_action(&app, "look").await.status().is_success(),
        "the Game accepts a Job after the stop"
    );
    assert!(wait_settled(&state, 2000).await, "that Job settles");
}

// [docs/specs/autoplay.md] SCENARIO: 41.8
#[tokio::test]
async fn test_a_start_on_a_busy_game_answers_still_thinking() {
    let model = Arc::new(ScriptedModel::new().turn(FIRST_NARRATION));
    let hold = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the Job should reach its narrator call"
    );

    let refused = start_autoplay(&app, "cycles=1").await;
    assert_eq!(refused.status(), StatusCode::OK);
    assert_still_thinking(&response_body(refused).await);

    hold.release();
    assert!(wait_settled(&state, 2000).await, "the held Job settles");
    assert!(
        wait_for_jobs(&state, 2000).await,
        "every task of the Game should have ended"
    );
    assert_eq!(
        narrator_calls(&model).len(),
        1,
        "only the held Job made a call"
    );
    assert!(
        !state.game_view_query.is_busy(game),
        "no run holds the Game"
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.8
#[tokio::test]
async fn test_a_start_on_a_game_that_a_run_holds_answers_still_thinking() {
    let model = Arc::new(ScriptedModel::new().turn(FIRST_INPUT).turn(FIRST_NARRATION));
    let hold = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    assert!(start_autoplay(&app, "cycles=1").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the run's Job should reach its narrator call"
    );

    let refused = start_autoplay(&app, "cycles=5").await;
    assert_eq!(refused.status(), StatusCode::OK);
    assert_still_thinking(&response_body(refused).await);

    hold.release();
    wait_for_run(&state, game).await;
    assert_eq!(
        story_after_opening(&state, game).len(),
        2,
        "the first run played its own budget of 1 cycle, and the refused start added none"
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.9
#[tokio::test]
async fn test_a_budget_that_is_zero_above_the_maximum_or_not_a_whole_number_answers_400() {
    let model = Arc::new(ScriptedModel::new());
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    for body in [
        "cycles=0",
        "cycles=101",
        "cycles=many",
        "cycles=-3",
        "cycles=2.5",
    ] {
        let refused = start_autoplay(&app, body).await;
        assert_eq!(refused.status(), StatusCode::BAD_REQUEST, "{body}");
        assert_status(&app, "idle", body).await;
        assert!(!state.game_view_query.is_busy(game), "{body}");
    }
    assert!(model.calls().is_empty(), "a refused start makes no call");
}

// [docs/specs/autoplay.md] SCENARIO: 41.9
#[tokio::test]
async fn test_a_budget_of_the_maximum_starts_a_run() {
    let model = Arc::new(ScriptedModel::new().turn(LATE_ANSWER));
    let hold = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    assert!(
        start_autoplay(&app, "cycles=100")
            .await
            .status()
            .is_success()
    );
    assert!(
        hold.wait_started(1000),
        "the first Job should reach its narrator call"
    );
    assert_status(
        &app,
        "autoplay 1/100 narrating",
        "100 is the largest budget",
    )
    .await;

    stop(&app).await;
    hold.release();
    wait_for_run(&state, game).await;
}

// [docs/specs/autoplay.md] SCENARIO: 41.10
#[tokio::test]
async fn test_a_start_with_no_cycles_field_runs_with_a_budget_of_ten() {
    let model = Arc::new(ScriptedModel::new().turn(LATE_ANSWER));
    let hold = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    assert!(start_autoplay(&app, "").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the first Job should reach its narrator call"
    );
    assert_status(&app, "autoplay 1/10 narrating", "the default budget is 10").await;

    stop(&app).await;
    hold.release();
    wait_for_run(&state, game).await;
}

// [docs/specs/autoplay.md] SCENARIO: 41.11
#[tokio::test]
async fn test_while_a_run_holds_the_game_every_story_write_is_refused_between_two_jobs() {
    let model = Arc::new(
        ScriptedModel::new()
            .turn("You look around.")
            .turn("You look around again."),
    );
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(wait_settled(&state, 2000).await, "the first Job settles");
    assert!(post_empty(&app, "/swipe/new").await.status().is_success());
    assert!(wait_settled(&state, 2000).await, "the Retry settles");
    let story = messages_of(&state, game);
    let narration = story
        .iter()
        .find(|message| {
            message.message_type == MessageType::Narration && message.swipe_count() == 2
        })
        .expect("the Retry leaves two Swipes on the Narration");
    let other_swipe = 1 - narration.active_swipe_index;
    let calls_before = model.calls().len();

    state
        .game_writer
        .start_run(game, 2)
        .expect("a run starts on an idle Game");

    let writes = [
        post_action(&app, "look").await,
        post_empty(&app, "/swipe/new").await,
        post_empty(
            &app,
            &format!("/message/{}/swipe/{other_swipe}", narration.id),
        )
        .await,
        post_form(
            &app,
            &format!("/history/{}", narration.id),
            "text=Rewritten",
        )
        .await,
        post_empty(&app, "/history/delete").await,
    ];
    for response in writes {
        assert_eq!(response.status(), StatusCode::OK);
        assert_still_thinking(&response_body(response).await);
    }

    assert_eq!(messages_of(&state, game), story, "the story is as it was");
    assert_eq!(
        model.calls().len(),
        calls_before,
        "no refused write made a call"
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.12
#[tokio::test]
async fn test_while_a_run_holds_the_game_the_config_writes_are_refused_and_the_config_stays() {
    let model = Arc::new(ScriptedModel::new());
    let (app, state, storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);
    storage
        .save_preset(&TestPromptPreset::system(
            "system_second",
            "SecondPresetMarker",
        ))
        .expect("the second preset seeds");
    let before = state
        .game_catalogue
        .active_game()
        .expect("the Game reads back")
        .expect("the active Game exists");
    let other_mode = match before.narrator_mode {
        NarratorMode::Novel => NarratorMode::InteractiveFiction,
        NarratorMode::InteractiveFiction => NarratorMode::Novel,
    };
    let other_perspective = match before.narrative_perspective {
        NarrativePerspective::Second => NarrativePerspective::Third,
        NarrativePerspective::Third => NarrativePerspective::Second,
    };
    let other_tense = match before.narrative_tense {
        NarrativeTense::Present => NarrativeTense::Past,
        NarrativeTense::Past => NarrativeTense::Present,
    };
    let posture_body = format!(
        "narrative_perspective={}&narrative_tense={}",
        other_perspective.as_str(),
        other_tense.as_str()
    );

    state
        .game_writer
        .start_run(game, 2)
        .expect("a run starts on an idle Game");

    let refused = [
        post_form(
            &app,
            &format!("/games/{}/mode", game.0),
            &format!("narrator_mode={}", other_mode.as_str()),
        )
        .await,
        post_form(&app, &format!("/games/{}/posture", game.0), &posture_body).await,
        post_form(
            &app,
            &format!("/games/{}/presets", game.0),
            &format!(
                "system_preset_id=system_second&quantifier_preset_id={}&impersonate_preset_id={}&options_preset_id={}",
                before.active_quantifier_prompt_preset_id,
                before.active_impersonate_prompt_preset_id,
                before.active_options_prompt_preset_id,
            ),
        )
        .await,
    ];
    for response in refused {
        assert_eq!(response.status(), StatusCode::OK);
        assert_still_thinking(&response_body(response).await);
    }
    let during = state
        .game_catalogue
        .active_game()
        .expect("the Game reads back")
        .expect("the active Game exists");
    assert_eq!(
        during, before,
        "the refused writes leave the Game's config as it was"
    );

    stop(&app).await;
    let accepted = post_form(&app, &format!("/games/{}/posture", game.0), &posture_body).await;
    assert!(accepted.status().is_success());
    let after = state
        .game_catalogue
        .active_game()
        .expect("the Game reads back")
        .expect("the active Game exists");
    assert_eq!(
        (after.narrative_perspective, after.narrative_tense),
        (other_perspective, other_tense),
        "the posture write lands once the run is gone"
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.13
#[tokio::test]
async fn test_the_status_poll_answers_with_the_progress_while_a_run_holds_the_game() {
    let model = Arc::new(
        ScriptedModel::new()
            .turn(FIRST_INPUT)
            .turn(FIRST_NARRATION)
            .turn(SECOND_INPUT)
            .turn(SECOND_NARRATION),
    );
    // A hold fires once and then clears, so the next hold goes in while the
    // previous one still blocks its call.
    let impersonate_narrator = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    assert!(start_autoplay(&app, "cycles=2").await.status().is_success());
    assert!(impersonate_narrator.wait_started(1000));
    assert_status(
        &app,
        "autoplay 1/2 narrating",
        "the Impersonate Job of cycle 1",
    )
    .await;

    let impersonate_analysis = model.hold(Role::Quantifier);
    impersonate_narrator.release();
    assert!(impersonate_analysis.wait_started(1000));
    assert_status(&app, "autoplay 1/2 quantifying", "the analysis of cycle 1").await;

    let continue_narrator = model.hold(Role::Narrator);
    impersonate_analysis.release();
    assert!(continue_narrator.wait_started(1000));
    assert_status(
        &app,
        "autoplay 1/2 narrating",
        "the Continue Job of cycle 1",
    )
    .await;

    let second_impersonate = model.hold(Role::Narrator);
    continue_narrator.release();
    assert!(second_impersonate.wait_started(1000));
    assert_status(
        &app,
        "autoplay 2/2 narrating",
        "the Impersonate Job of cycle 2",
    )
    .await;

    second_impersonate.release();
    wait_for_run(&state, game).await;
    assert_status(&app, "idle", "the Game is idle after the run").await;
}

// [docs/specs/autoplay.md] SCENARIO: 41.13
#[tokio::test]
async fn test_the_status_poll_names_no_step_between_two_jobs_of_a_run() {
    let (app, state, _storage) = app_with_script(ScriptedModel::new());
    let game = active_game_of(&state);

    state
        .game_writer
        .start_run(game, 2)
        .expect("a run starts on an idle Game");

    assert_status(&app, "autoplay 1/2", "a run between two Jobs names no step").await;
    assert_eq!(
        fetch_body(&app, "/debug/is_generating").await.trim(),
        "true",
        "the debug answer reads the run hold as busy"
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.14
#[tokio::test]
async fn test_a_run_goes_on_when_the_client_switches_the_active_game() {
    let model = Arc::new(ScriptedModel::new().turn(FIRST_INPUT).turn(FIRST_NARRATION));
    let hold = model.hold(Role::Narrator);
    let (app, state, storage) = app_with_script(Arc::clone(&model));
    let game_a = active_game_of(&state);

    assert!(start_autoplay(&app, "cycles=1").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the run's Job should reach its narrator call"
    );

    let game_b = create_game_from(&app, &state, &storage, game_a).await;

    hold.release();
    wait_for_run(&state, game_a).await;

    assert_eq!(
        story_after_opening(&state, game_a),
        vec![
            (MessageType::Input, FIRST_INPUT.to_string()),
            (MessageType::Narration, FIRST_NARRATION.to_string()),
        ],
        "the run went on for Game A while Game B was active"
    );
    assert!(
        story_after_opening(&state, game_b).is_empty(),
        "Game B holds its opening Narration only"
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.15
#[tokio::test]
async fn test_a_reset_of_the_game_ends_its_run() {
    let model = Arc::new(ScriptedModel::new().turn(LATE_ANSWER));
    let hold = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let before = active_game_of(&state);

    assert!(start_autoplay(&app, "cycles=2").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the run's Job should reach its narrator call"
    );

    assert!(post_empty(&app, "/reset").await.status().is_success());
    let after = active_game_of(&state);
    assert_ne!(after, before, "the reset points the app at a new Game");

    hold.release();
    assert!(hold.wait_taken(1000), "the ended call takes its answer");
    wait_for_run(&state, after).await;

    assert!(
        story_after_opening(&state, after).is_empty(),
        "the new Game holds its opening Narration only"
    );
    assert_status(&app, "idle", "the new Game is idle").await;
    assert_eq!(
        narrator_calls(&model).len(),
        1,
        "no further Job of the run started"
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.16
#[tokio::test]
async fn test_a_delete_of_the_game_ends_its_run() {
    let model = Arc::new(ScriptedModel::new().turn(LATE_ANSWER));
    let hold = model.hold(Role::Narrator);
    let (app, state, storage) = app_with_script(Arc::clone(&model));
    let game_a = active_game_of(&state);

    assert!(start_autoplay(&app, "cycles=2").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the run's Job should reach its narrator call"
    );

    create_game_from(&app, &state, &storage, game_a).await;

    let deleted = post_empty(&app, &format!("/games/{}/delete", game_a.0)).await;
    assert!(deleted.status().is_success(), "the delete should accept");

    hold.release();
    assert!(hold.wait_taken(1000), "the ended call takes its answer");
    wait_for_run(&state, game_a).await;

    assert_eq!(
        narrator_calls(&model).len(),
        1,
        "no further Job of the run started"
    );
    let games = state
        .game_catalogue
        .list_games()
        .expect("the Games list reads back");
    assert!(
        games.iter().all(|game| game.id != game_a.0),
        "the deleted Game is gone"
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.17
#[tokio::test]
async fn test_the_end_of_a_stopped_run_does_not_release_the_run_that_started_after_it() {
    let model = Arc::new(
        ScriptedModel::new()
            .narration(LATE_ANSWER)
            .turn(FIRST_INPUT)
            .turn(FIRST_NARRATION),
    );
    let hold_a = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    assert!(start_autoplay(&app, "cycles=2").await.status().is_success());
    assert!(
        hold_a.wait_started(1000),
        "run A's Job should reach its narrator call"
    );
    stop(&app).await;

    // The call of run A fired its hold, so the hold of run B can go in while run
    // A's call still blocks.
    let hold_b = model.hold(Role::Narrator);
    assert!(
        start_autoplay(&app, "cycles=1").await.status().is_success(),
        "a stopped Game accepts a new run"
    );
    assert!(
        hold_b.wait_started(1000),
        "run B's Job should reach its narrator call"
    );

    hold_a.release();
    assert!(hold_a.wait_taken(1000), "run A's call takes its answer");
    // Run B's task stays held, so run A's task is the only one that can end.
    assert!(
        wait_for_jobs_running_at_most(&state, 1, 2000).await,
        "run A's driver should end while run B stays held"
    );

    assert_status(
        &app,
        "autoplay 1/1 narrating",
        "run B still holds the Game after run A ended",
    )
    .await;
    let refused = post_action(&app, "look").await;
    assert_eq!(refused.status(), StatusCode::OK);
    assert_still_thinking(&response_body(refused).await);

    hold_b.release();
    wait_for_run(&state, game).await;
    assert_eq!(
        story_after_opening(&state, game),
        vec![
            (MessageType::Input, FIRST_INPUT.to_string()),
            (MessageType::Narration, FIRST_NARRATION.to_string()),
        ],
        "run B played its own budget of 1 cycle"
    );
}

fn button_tags_with_class(html: &str, class: &str) -> Vec<String> {
    html.split("<button")
        .skip(1)
        .filter_map(|rest| rest.split('>').next())
        .filter(|tag| tag.contains(class))
        .map(|tag| tag.to_string())
        .collect()
}

fn story_changing_controls(story_log: &str) -> Vec<String> {
    let mut controls = button_tags_with_class(story_log, "edit-btn");
    controls.extend(button_tags_with_class(story_log, "delete-btn"));
    controls.extend(
        button_tags_with_class(story_log, "swipe-btn")
            .into_iter()
            .filter(|tag| {
                tag.contains("aria-label=\"Retry\"")
                    || tag.contains("aria-label=\"Previous swipe\"")
            }),
    );
    controls
}

fn assert_controls(controls: &[String], disabled: bool, label: &str) {
    assert!(
        controls.len() >= 4,
        "{label}: expected Edit, Delete, Previous swipe and Retry controls, got {controls:?}"
    );
    for control in controls {
        assert_eq!(
            control.contains(" disabled"),
            disabled,
            "{label}: {control}"
        );
    }
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

    state
        .game_writer
        .start_run(game, 2)
        .expect("a run starts on an idle Game");
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

    stop(&app).await;
    let after = fetch_body(&app, "/fragment/story-log").await;
    assert_controls(&story_changing_controls(&after), false, "after the stop");
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
async fn test_while_a_run_holds_the_game_retrigger_and_next_swipe_render_disabled() {
    let (app, state) = app_after_an_impersonate_with_a_failed_event().await;
    let game = active_game_of(&state);
    state
        .game_writer
        .start_run(game, 2)
        .expect("a run starts on an idle Game");
    let retrigger = button_tags_with_class(
        &fetch_body(&app, "/fragment/story-log").await,
        "retrigger-btn",
    );
    assert_eq!(
        retrigger.len(),
        1,
        "the story log offers Retrigger: {retrigger:?}"
    );
    assert!(
        retrigger[0].contains(" disabled"),
        "Retrigger: {}",
        retrigger[0]
    );

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
    let narration = messages_of(&state, game)
        .pop()
        .expect("the story ends on the Narration");
    assert!(
        post_empty(&app, &format!("/message/{}/swipe/0", narration.id))
            .await
            .status()
            .is_success()
    );

    state
        .game_writer
        .start_run(game, 2)
        .expect("a run starts on an idle Game");
    let during = swipe_button(&fetch_body(&app, "/fragment/story-log").await, "Next swipe");
    assert!(
        during.contains(" disabled"),
        "a run holds the Game: {during}"
    );

    stop(&app).await;
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
    let game_b = create_game_from(&app, &state, &storage, game_a).await;
    let name_b = state
        .game_view_query
        .game_display_name(game_b)
        .expect("Game B reads back");
    assert_ne!(name_a, name_b, "the two Games have distinct names");

    state
        .game_writer
        .start_run(game_a, 2)
        .expect("a run starts on idle Game A");
    let panel = fetch_body(&app, "/fragment/games").await;
    assert!(
        item_of_game(&panel, &name_a).contains("run-badge"),
        "the saved Game that a run holds carries the badge"
    );
    assert!(
        !item_of_game(&panel, &name_b).contains("run-badge"),
        "the active Game that no run holds carries no badge"
    );

    state
        .game_writer
        .start_run(game_b, 2)
        .expect("a run starts on idle Game B");
    let panel = fetch_body(&app, "/fragment/games").await;
    assert!(
        item_of_game(&panel, &name_b).contains("run-badge"),
        "the active Game that a run holds carries the badge"
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
    let model = Arc::new(ScriptedModel::new());
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    state
        .game_writer
        .start_run(game, 2)
        .expect("a run starts on an idle Game");
    let during = fetch_body(&app, "/fragment/games").await;
    assert_config_selects(&during, true, "a run holds the Game");

    stop(&app).await;
    let after = fetch_body(&app, "/fragment/games").await;
    assert_config_selects(&after, false, "after the stop");
}
