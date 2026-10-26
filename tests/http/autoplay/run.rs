//! HTTP E2E tests for the run of Autoplay (autoplay.md 41.1-41.17, 41.21): the Jobs of a run, its ends, the budget, the refused writes, and the progress status.

use std::sync::Arc;

use axum::http::StatusCode;

use chronicler_engine::adapters::driven::storage::{StorageCall, TestOverride};
use chronicler_engine::domain::model::settings::AppSettings;
use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::domain::model::message::Message;
use chronicler_engine::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::domain::model::trigger::ComparisonOperator;
use chronicler_engine::domain::model::world::WorldCard;
use chronicler_engine::test_support::{
    active_game_of, analysis_report, configured_runner, empty_sqlite_storage, ScriptedModel,
    TestMap, TestCharacter, TestPromptPreset,
};

use super::{
    assert_status, end_paused_run, narrator_calls, pause_between_two_jobs, start_autoplay, stop,
    story_after_opening, wait_for_run, FIRST_INPUT, FIRST_NARRATION, LATE_ANSWER, SECOND_INPUT,
    SECOND_NARRATION,
};
use crate::support::app_wiring::{
    app_with_a_triggered_character, app_with_model_and_storage, app_with_script,
};
use crate::support::http_assertions::assert_still_thinking;
use crate::support::http_fixtures::{create_game_from, story_of_game};
use crate::support::http_requests::{
    assert_answers_nothing, fetch_body, fetch_busy_status, post_action, post_empty, post_form,
    response_body, wait_for_jobs, wait_for_jobs_running_at_most, wait_settled,
};

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
    assert_answers_nothing(started).await;
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
    assert_status(&app, "Ready", "the Game is idle after the run").await;

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
    const EVENT_NARRATION: &str = "The Character greets you warmly.";
    // The Trigger is scoped to the second room, which only the Continue Job's
    // analysis moves the player into.
    let character = TestCharacter::with_room_scoped_trigger(
        "character_1",
        "Test Character",
        ComparisonOperator::Eq,
        0,
        "room2",
    );
    let model = ScriptedModel::new()
        .narration(FIRST_INPUT)
        .after_narration(analysis_report(&[], None))
        .narration(FIRST_NARRATION)
        .after_narration(analysis_report(&["character_1"], Some("room2")))
        .event_narration(EVENT_NARRATION)
        .after_narration(analysis_report(&["character_1"], None))
        .narration(SECOND_INPUT)
        .after_narration(analysis_report(&[], None))
        .narration(SECOND_NARRATION)
        .after_narration(analysis_report(&[], None));
    let (app, state) =
        app_with_a_triggered_character(TestMap::two_rooms("room_1", "room2"), character, model);
    let game = active_game_of(&state);

    let started = start_autoplay(&app, "cycles=2").await;
    assert!(started.status().is_success(), "the start should accept");
    wait_for_run(&state, game).await;

    let story = story_of_game(&state, game);
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
    const EVENT_NARRATION: &str = "The Character greets you warmly.";
    let character = TestCharacter::with_room_scoped_trigger(
        "character_1",
        "Test Character",
        ComparisonOperator::Eq,
        0,
        "room2",
    );
    let model = ScriptedModel::new()
        .narration(FIRST_INPUT)
        .after_narration(analysis_report(&["character_1"], Some("room2")))
        .event_narration(EVENT_NARRATION)
        .after_narration(analysis_report(&["character_1"], None))
        .narration(SECOND_INPUT)
        .after_narration(analysis_report(&[], None))
        .narration(SECOND_NARRATION)
        .after_narration(analysis_report(&[], None));
    let (app, state) =
        app_with_a_triggered_character(TestMap::two_rooms("room_1", "room2"), character, model);
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
    assert_status(&app, "Ready", "the Game is idle after the run").await;
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
    let status = fetch_busy_status(&app).await;
    assert!(
        status.contains("status-error-popover") && status.contains("answer could not be read"),
        "the status load shows the failure line of the Job, got: {status}"
    );
    assert_eq!(
        narrator_calls(&model).len(),
        2,
        "no Job started after the failed one: its budget of 3 does not matter"
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.5
#[tokio::test]
async fn test_a_failed_options_run_writes_its_options_failure_and_the_run_goes_on() {
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
        "the dock shows the Options failure of the failed Options run, got: {dock}"
    );
    assert_status(&app, "Ready", "a failed Options run leaves no failure line").await;
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
    let model = Arc::new(
        ScriptedModel::new()
            .narration(LATE_ANSWER)
            .turn(FIRST_NARRATION),
    );
    let hold = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);
    let before = story_of_game(&state, game);

    assert!(start_autoplay(&app, "cycles=2").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the first Job of the run should reach its narrator call"
    );

    stop(&app).await;
    assert_status(&app, "Ready", "a stopped run leaves the Game idle at once").await;

    hold.release();
    assert!(hold.wait_taken(1000), "the stopped call takes its answer");
    wait_for_run(&state, game).await;

    assert_eq!(
        story_of_game(&state, game),
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
    let model = Arc::new(ScriptedModel::new().turn(FIRST_INPUT).turn(FIRST_NARRATION));
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    let pause = pause_between_two_jobs(&app, &state, game).await;
    assert_status(
        &app,
        "Autoplay: cycle 1/2",
        "the run holds the Game between its Jobs",
    )
    .await;

    stop(&app).await;
    assert_status(&app, "Ready", "a stopped run leaves the Game idle").await;
    assert!(!state.game_view_query.is_busy(game), "the Game is not busy");

    pause.release();
    wait_for_run(&state, game).await;
    assert_eq!(
        story_after_opening(&state, game),
        vec![(MessageType::Input, FIRST_INPUT.to_string())],
        "the story holds the Input of the first Job and nothing after it"
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
    assert_eq!(refused.status(), StatusCode::CONFLICT);
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
    assert_eq!(refused.status(), StatusCode::CONFLICT);
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
        assert_status(&app, "Ready", body).await;
        assert!(!state.game_view_query.is_busy(game), "{body}");
    }
    assert!(model.calls().is_empty(), "a refused start makes no call");
}

// [docs/specs/autoplay.md] SCENARIO: 41.9
#[tokio::test]
async fn test_a_bad_budget_answers_400_also_on_a_busy_game() {
    let model = Arc::new(ScriptedModel::new().turn(FIRST_NARRATION));
    let hold = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the Job should reach its narrator call"
    );

    let refused = start_autoplay(&app, "cycles=0").await;
    assert_eq!(
        refused.status(),
        StatusCode::BAD_REQUEST,
        "the budget is read before the busy check"
    );

    hold.release();
    assert!(wait_settled(&state, 2000).await, "the held Job settles");
    assert!(
        wait_for_jobs(&state, 2000).await,
        "every task of the Game should have ended"
    );
    assert_eq!(
        story_after_opening(&state, game),
        vec![
            (MessageType::Input, "look".to_string()),
            (MessageType::Narration, FIRST_NARRATION.to_string()),
        ],
        "the held Job went on"
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.9
#[tokio::test]
async fn test_a_budget_of_the_maximum_starts_a_run() {
    let model = Arc::new(ScriptedModel::new().narration(LATE_ANSWER));
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
        "Autoplay: cycle 1/100 Generating narration...",
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
    let model = Arc::new(ScriptedModel::new().narration(LATE_ANSWER));
    let hold = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    assert!(start_autoplay(&app, "").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the first Job should reach its narrator call"
    );
    assert_status(
        &app,
        "Autoplay: cycle 1/10 Generating narration...",
        "the default budget is 10",
    )
    .await;

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
            .turn("You look around again.")
            .turn(FIRST_INPUT),
    );
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(wait_settled(&state, 2000).await, "the first Job settles");
    assert!(post_empty(&app, "/swipe/new").await.status().is_success());
    assert!(wait_settled(&state, 2000).await, "the Retry settles");
    let swiped = story_of_game(&state, game);
    let narration = swiped
        .iter()
        .find(|message| {
            message.message_type == MessageType::Narration && message.swipe_count() == 2
        })
        .expect("the Retry leaves two Swipes on the Narration")
        .clone();
    let other_swipe = 1 - narration.active_swipe_index;

    let pause = pause_between_two_jobs(&app, &state, game).await;
    let story = story_of_game(&state, game);
    let calls_before = model.calls().len();

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
        assert_eq!(response.status(), StatusCode::CONFLICT);
        assert_still_thinking(&response_body(response).await);
    }

    assert_eq!(
        story_of_game(&state, game),
        story,
        "the story is as it was when the pause began"
    );
    assert_eq!(
        model.calls().len(),
        calls_before,
        "no refused write made a call"
    );
    end_paused_run(&app, &state, game, &pause).await;
}

// [docs/specs/autoplay.md] SCENARIO: 41.12
#[tokio::test]
async fn test_while_a_run_holds_the_game_the_config_writes_are_refused_and_the_config_stays() {
    let model = Arc::new(ScriptedModel::new().turn(FIRST_INPUT));
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

    let pause = pause_between_two_jobs(&app, &state, game).await;

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
        assert_eq!(response.status(), StatusCode::CONFLICT);
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

    end_paused_run(&app, &state, game, &pause).await;
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
async fn test_the_status_load_answers_with_the_progress_while_a_run_holds_the_game() {
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
        "Autoplay: cycle 1/2 Generating narration...",
        "the Impersonate Job of cycle 1",
    )
    .await;

    let impersonate_analysis = model.hold(Role::Quantifier);
    impersonate_narrator.release();
    assert!(impersonate_analysis.wait_started(1000));
    assert_status(
        &app,
        "Autoplay: cycle 1/2 Quantifying...",
        "the analysis of cycle 1",
    )
    .await;

    let continue_narrator = model.hold(Role::Narrator);
    impersonate_analysis.release();
    assert!(continue_narrator.wait_started(1000));
    assert_status(
        &app,
        "Autoplay: cycle 1/2 Generating narration...",
        "the Continue Job of cycle 1",
    )
    .await;

    let second_impersonate = model.hold(Role::Narrator);
    continue_narrator.release();
    assert!(second_impersonate.wait_started(1000));
    assert_status(
        &app,
        "Autoplay: cycle 2/2 Generating narration...",
        "the Impersonate Job of cycle 2",
    )
    .await;

    second_impersonate.release();
    wait_for_run(&state, game).await;
    assert_status(&app, "Ready", "the Game is idle after the run").await;
}

// [docs/specs/autoplay.md] SCENARIO: 41.13
#[tokio::test]
async fn test_the_status_load_names_no_step_between_two_jobs_of_a_run() {
    let model = Arc::new(ScriptedModel::new().turn(FIRST_INPUT));
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    let pause = pause_between_two_jobs(&app, &state, game).await;

    assert_status(
        &app,
        "Autoplay: cycle 1/2",
        "a run between two Jobs names no step",
    )
    .await;
    assert_eq!(
        fetch_body(&app, "/debug/is_busy").await.trim(),
        "true",
        "the debug answer reads the holder as busy"
    );
    let debug: serde_json::Value = serde_json::from_str(&fetch_body(&app, "/debug/state").await)
        .expect("/debug/state answers JSON");
    assert_eq!(
        debug["generation_status"], "Busy",
        "the debug state reads the holder as busy"
    );
    assert!(
        debug["step"].is_null(),
        "no Job of the run runs, so the debug state names no step"
    );
    end_paused_run(&app, &state, game, &pause).await;
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

    let game_b = create_game_from(&app, &state, &storage, game_a, "test").await;

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
    let model = Arc::new(ScriptedModel::new().narration(LATE_ANSWER));
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
    assert_status(&app, "Ready", "the new Game is idle").await;
    assert_eq!(
        narrator_calls(&model).len(),
        1,
        "no further Job of the run started"
    );
}

// [docs/specs/autoplay.md] SCENARIO: 41.16
#[tokio::test]
async fn test_a_delete_of_the_game_ends_its_run() {
    let model = Arc::new(ScriptedModel::new().narration(LATE_ANSWER));
    let hold = model.hold(Role::Narrator);
    let (app, state, storage) = app_with_script(Arc::clone(&model));
    let game_a = active_game_of(&state);

    assert!(start_autoplay(&app, "cycles=2").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the run's Job should reach its narrator call"
    );

    create_game_from(&app, &state, &storage, game_a, "test").await;

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
        "Autoplay: cycle 1/1 Generating narration...",
        "run B still holds the Game after run A ended",
    )
    .await;
    let refused = post_action(&app, "look").await;
    assert_eq!(refused.status(), StatusCode::CONFLICT);
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

// [docs/specs/autoplay.md] SCENARIO: 41.23
#[tokio::test]
async fn test_a_run_whose_story_cannot_be_read_ends_with_the_failure_line_of_the_story() {
    let storage = Arc::new(empty_sqlite_storage().with_failure(
        StorageCall::StoriesTail,
        TestOverride::internal("the story read failed"),
    ));
    let model = Arc::new(ScriptedModel::new());
    let (app, state, _storage) = app_with_model_and_storage(
        storage,
        Arc::clone(&model),
        AppSettings::default(),
        configured_runner,
    );
    let game = active_game_of(&state);

    let started = start_autoplay(&app, "cycles=2").await;
    assert!(started.status().is_success(), "the start should accept");
    wait_for_run(&state, game).await;

    let status = fetch_busy_status(&app).await;
    assert!(
        status.contains("status-error-popover") && status.contains("the story read failed"),
        "the status load shows the failure line of the unreadable story, got: {status}"
    );
    assert!(model.calls().is_empty(), "no model call is made");
}

// [docs/specs/autoplay.md] SCENARIO: 41.24
#[tokio::test]
async fn test_a_run_whose_quantifier_fails_at_each_attempt_plays_its_whole_budget() {
    // Each analysis makes two attempts, and four Jobs of two cycles analyse.
    let mut script = ScriptedModel::new()
        .narration(FIRST_INPUT)
        .narration(FIRST_NARRATION)
        .narration(SECOND_INPUT)
        .narration(SECOND_NARRATION);
    for _ in 0..8 {
        script = script.failed(Role::Quantifier);
    }
    let (app, state, _storage) = app_with_script(Arc::new(script));
    let game = active_game_of(&state);

    let started = start_autoplay(&app, "cycles=2").await;
    assert!(started.status().is_success(), "the start should accept");
    wait_for_run(&state, game).await;

    assert_eq!(
        story_after_opening(&state, game),
        vec![
            (MessageType::Input, FIRST_INPUT.to_string()),
            (MessageType::Narration, FIRST_NARRATION.to_string()),
            (MessageType::Input, SECOND_INPUT.to_string()),
            (MessageType::Narration, SECOND_NARRATION.to_string()),
        ],
        "a fallback of the Quantifier does not stop the run: both cycles are in the story"
    );
    assert_status(&app, "Ready", "the Game is idle after the run").await;
}

// [docs/specs/autoplay.md] SCENARIO: 41.23
#[tokio::test]
async fn test_a_run_whose_start_state_cannot_be_read_ends_with_the_failure_line_of_that_record() {
    let (storage, failures) = empty_sqlite_storage().with_test_failures();
    let model = Arc::new(ScriptedModel::new());
    let (app, state, _storage) = app_with_model_and_storage(
        Arc::new(storage),
        Arc::clone(&model),
        AppSettings::default(),
        configured_runner,
    );
    let game = active_game_of(&state);
    failures.set(
        StorageCall::StoriesSwipeState,
        TestOverride::internal("the start state is unreadable"),
    );

    let started = start_autoplay(&app, "cycles=2").await;
    assert!(started.status().is_success(), "the start should accept");
    wait_for_run(&state, game).await;

    failures.clear_all();
    let status = fetch_busy_status(&app).await;
    assert!(
        status.contains("status-error-popover") && status.contains("the start state"),
        "the status load names the start state, got: {status}"
    );
    assert!(
        status.contains("the start state is unreadable"),
        "the storage error is reachable in the disclosure, got: {status}"
    );
    assert!(model.calls().is_empty(), "no model call is made");
}
