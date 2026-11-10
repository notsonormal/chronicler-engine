//! HTTP E2E tests for the write path while a Job runs. They cover the races (actions.md 3.4, 3.5 and options.md 24.21) and the stop at each model call (dashboard.md 39.4, 39.5). They cover the reset (reset.md 7.3) and the Job of another Game (games.md 18.4). They cover the edit, delete and swipe switch, and the busy refusal before their other checks (story_log.md 8.7, 8.18 and swipe_switch.md 36.3, 36.8). They also cover the preset an event Narration uses (actions.md 6.4).

use std::sync::Arc;

use axum::http::StatusCode;
use tokio::sync::Barrier;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::domain::model::game::GameId;
use chronicler_engine::domain::model::message::Message;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::{
    active_game_of, analysis_report, sent_system_text, ModelHold, ScriptedModel, TestAppBuilder,
    TestDataBuilder,
};

use crate::support::app_wiring::{app_with_model, app_with_script};
use crate::support::http_assertions::{assert_still_thinking, llm_call_count, STILL_THINKING};
use crate::support::http_fixtures::{
    create_game_from, room_header_of, story_of, story_of_game, OFFERED_SET,
};
use crate::support::http_requests::{
    fetch_status_label, post_action, post_empty, post_form, response_body, wait_for_jobs,
    wait_idle, wait_settled, post_on_last_message, post_on_message, fetch_part,
};

fn held_action() -> (Arc<ScriptedModel>, ModelHold) {
    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let hold = model.hold(Role::Narrator);
    (model, hold)
}

// [docs/specs/actions.md] SCENARIO: 3.4
#[tokio::test]
async fn test_action_while_job_runs_is_refused_http() {
    let (model, hold) = held_action();
    let (app, state) = app_with_model(Arc::clone(&model));

    let first = post_action(&app, "look").await;
    assert!(
        first.status().is_success(),
        "the first Action should accept"
    );
    assert!(
        hold.wait_started(1000),
        "the held narration call should reach the model"
    );

    let refused = post_action(&app, "listen").await;
    assert_eq!(refused.status(), StatusCode::CONFLICT);
    let body = response_body(refused).await;
    assert_still_thinking(&body);
    assert_eq!(
        texts_of(&state, active_game_of(&state), MessageType::Input),
        vec!["look".to_string()],
        "the refused Action must save no input of its own"
    );

    hold.release();
    assert!(
        wait_settled(&state, 1000).await,
        "the held Action should complete once released"
    );
    assert_eq!(
        texts_of(&state, active_game_of(&state), MessageType::Input),
        vec!["look".to_string()],
        "the held Action's own input is the only one saved"
    );
    let narrations = story_of(&state)
        .into_iter()
        .filter(|message| message.message_type == MessageType::Narration)
        .count();
    assert_eq!(
        narrations, 2,
        "the opening Narration and the held Action's own one, and nothing of the \
         refused Action's"
    );
}

// [docs/specs/actions.md] SCENARIO: 3.5
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_two_actions_at_the_same_moment_save_one_input_http() {
    let (model, hold) = held_action();
    let (app, state) = app_with_model(Arc::clone(&model));
    let barrier = Arc::new(Barrier::new(2));

    let click = |app: axum::Router, barrier: Arc<Barrier>| async move {
        barrier.wait().await;
        let resp = post_action(&app, "look").await;
        let status = resp.status();
        (status, response_body(resp).await)
    };
    let first = tokio::spawn(click(app.clone(), Arc::clone(&barrier)));
    let second = tokio::spawn(click(app.clone(), Arc::clone(&barrier)));
    let bodies = [
        first.await.expect("the first click joins"),
        second.await.expect("the second click joins"),
    ];

    let refused = bodies
        .iter()
        .filter(|(_, body)| body.contains(STILL_THINKING))
        .count();
    assert_eq!(refused, 1, "exactly one click is refused: {bodies:?}");
    for (status, body) in &bodies {
        let expected = if body.contains(STILL_THINKING) {
            StatusCode::CONFLICT
        } else {
            StatusCode::OK
        };
        assert_eq!(
            *status, expected,
            "the refused click answers 409 and the accepted one 2xx: {bodies:?}"
        );
    }
    assert!(
        hold.wait_started(1000),
        "the accepted click's narration call should reach the model"
    );

    hold.release();
    assert!(
        wait_settled(&state, 1000).await,
        "the accepted click's Job should complete once released"
    );
    assert_eq!(
        texts_of(&state, active_game_of(&state), MessageType::Input),
        vec!["look".to_string()],
        "two clicks at the same moment save exactly one input"
    );
    let narrations = story_of(&state)
        .into_iter()
        .filter(|message| message.message_type == MessageType::Narration)
        .count();
    assert_eq!(
        narrations, 2,
        "the opening Narration and one Narration of the accepted Action"
    );
    assert_eq!(
        llm_call_count(&state, "narrator"),
        1,
        "the refused click spends no narration call"
    );
}

// [docs/specs/options.md] SCENARIO: 24.21
#[tokio::test]
async fn test_options_while_job_runs_is_refused_http() {
    let (model, hold) = held_action();
    let (app, state) = app_with_model(Arc::clone(&model));

    let first = post_action(&app, "look").await;
    assert!(
        first.status().is_success(),
        "the first Action should accept"
    );
    assert!(
        hold.wait_started(1000),
        "the held narration call should reach the model"
    );

    let refused = post_action(&app, "/options").await;
    assert_eq!(refused.status(), StatusCode::CONFLICT);
    let body = response_body(refused).await;
    assert_still_thinking(&body);

    hold.release();
    assert!(
        wait_settled(&state, 1000).await,
        "the held Action should complete once released"
    );
    assert_eq!(
        llm_call_count(&state, "options"),
        0,
        "the refused /options must generate nothing"
    );
    let dock = fetch_part(&app, "options-dock").await;
    assert!(
        !dock.contains("options-strip"),
        "the refused /options must leave the dock empty: {dock}"
    );
}

fn texts_of(state: &AppState, game: GameId, kind: MessageType) -> Vec<String> {
    story_of_game(state, game)
        .into_iter()
        .filter(|message| message.message_type == kind)
        .map(|message| message.text().to_string())
        .collect()
}

const STOPPED_NARRATION: &str = "The stopped Job's narration.";
const NEXT_NARRATION: &str = "The next Job's narration.";

// [docs/specs/dashboard.md] SCENARIO: 39.5
#[tokio::test]
async fn test_a_stopped_job_writes_nothing_and_the_next_job_saves_its_own_text() {
    let model = Arc::new(
        ScriptedModel::new()
            .narration(STOPPED_NARRATION)
            .narration(NEXT_NARRATION)
            .after_narration(analysis_report(&[], None)),
    );
    let stopped_hold = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        stopped_hold.wait_started(1000),
        "the first Job should reach its narrator call"
    );

    let stop = post_empty(&app, "/status/reset-busy").await;
    assert_eq!(
        response_body(stop).await,
        "reset",
        "a stop of a busy Game answers reset"
    );

    let next_hold = model.hold(Role::Narrator);
    assert!(post_action(&app, "listen").await.status().is_success());
    assert!(
        next_hold.wait_started(1000),
        "the second Job should reach its narrator call"
    );

    // The call released first takes the first scripted answer.
    stopped_hold.release();
    assert!(
        stopped_hold.wait_taken(1000),
        "the stopped call should take its answer"
    );
    next_hold.release();
    assert!(wait_for_jobs(&state, 2000).await, "both Jobs should end");
    assert!(
        wait_idle(&state, 2000).await,
        "the next Job should end idle"
    );

    assert_eq!(
        texts_of(&state, game, MessageType::Input),
        vec!["look".to_string(), "listen".to_string()],
        "the stopped Job's Input stays, and the next Job saves its own"
    );
    let narrations = texts_of(&state, game, MessageType::Narration);
    assert_eq!(
        narrations.iter().skip(1).cloned().collect::<Vec<_>>(),
        vec![NEXT_NARRATION.to_string()],
        "the stopped Job's late answer adds no Narration, so the next Job's text is \
         the only one after the opening"
    );
}

// [docs/specs/reset.md] SCENARIO: 7.3
#[tokio::test]
async fn test_reset_while_job_runs_keeps_only_the_new_game_opening() {
    let model = Arc::new(ScriptedModel::new().narration("The stopped narration."));
    let hold = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let before = active_game_of(&state);

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the Job should reach its narrator call"
    );

    let reset = post_empty(&app, "/reset").await;
    assert!(reset.status().is_success(), "the reset should accept");

    hold.release();
    assert!(
        wait_for_jobs(&state, 2000).await,
        "the Job the reset ended should end"
    );
    let after = active_game_of(&state);
    assert_ne!(after, before, "the reset points the app at a new Game");
    assert!(
        wait_idle(&state, 2000).await,
        "the new Game should end idle"
    );

    assert!(
        texts_of(&state, after, MessageType::Input).is_empty(),
        "the new Game holds no Input of the Job the reset ended"
    );
    let narrations = texts_of(&state, after, MessageType::Narration);
    assert_eq!(
        narrations.len(),
        1,
        "the new Game holds its opening Narration only, got: {narrations:?}"
    );
    assert!(
        !narrations
            .iter()
            .any(|text| text == "The stopped narration."),
        "the late answer of the Job the reset ended must not join the new Game"
    );
    assert_eq!(
        fetch_status_label(&app).await,
        "Ready",
        "the new Game answers Ready, with no failure line"
    );
}

// [docs/specs/story_log.md] SCENARIO: 8.7
#[tokio::test]
async fn test_edit_and_delete_while_job_runs_are_refused() {
    let (model, hold) = held_action();
    let (app, state) = app_with_model(Arc::clone(&model));
    let game = active_game_of(&state);

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the Job should reach its narrator call"
    );
    let before = story_of_game(&state, game);
    let target = before.last().expect("the Job saved its Input").id;

    let edit = post_form(&app, &format!("/history/{target}"), "text=An+edited+text").await;
    assert_eq!(edit.status(), StatusCode::CONFLICT);
    let edit_body = response_body(edit).await;
    assert_still_thinking(&edit_body);

    let delete = post_on_last_message(&app, "/history/delete").await;
    assert_eq!(delete.status(), StatusCode::CONFLICT);
    let delete_body = response_body(delete).await;
    assert_still_thinking(&delete_body);

    let after = story_of_game(&state, game);
    assert_eq!(
        after.len(),
        before.len(),
        "a refused edit and a refused delete write nothing"
    );
    assert_eq!(
        after.last().map(|message| message.text().to_string()),
        before.last().map(|message| message.text().to_string()),
        "the Message the client tried to edit keeps the text it had"
    );

    hold.release();
    assert!(
        wait_settled(&state, 1000).await,
        "the held Job should complete once released"
    );
}

// [docs/specs/story_log.md] SCENARIO: 8.18
#[tokio::test]
async fn test_the_busy_refusal_comes_before_the_other_checks_of_an_edit_and_a_delete() {
    let (model, hold) = held_action();
    let (app, state) = app_with_model(Arc::clone(&model));
    let game = active_game_of(&state);

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the Job should reach its narrator call"
    );
    let before = story_of_game(&state, game);
    let first = before.first().expect("the Game holds its opening").id;
    let no_message = before.iter().map(|message| message.id).max().unwrap_or(0) + 1000;

    let edit = post_form(
        &app,
        &format!("/history/{no_message}"),
        "text=An+edited+text",
    )
    .await;
    assert_eq!(edit.status(), StatusCode::CONFLICT);
    assert_still_thinking(&response_body(edit).await);

    let delete = post_on_message(&app, "/history/delete", first).await;
    assert_eq!(delete.status(), StatusCode::CONFLICT);
    assert_still_thinking(&response_body(delete).await);

    assert_eq!(
        story_of_game(&state, game),
        before,
        "a refused edit and a refused delete write nothing"
    );

    hold.release();
    assert!(
        wait_settled(&state, 1000).await,
        "the held Job should complete once released"
    );
}

/// A Game whose last Message is a Narration with two Swipes, and a continue Job of
/// that Game held at its narrator call.
struct HeldOnTwoSwipes {
    app: axum::Router,
    state: AppState,
    game: GameId,
    narration: Message,
    hold: ModelHold,
}

async fn held_job_on_two_swipes() -> HeldOnTwoSwipes {
    let model = Arc::new(
        ScriptedModel::new()
            .turn("You look around.")
            .turn("You look around again.")
            .turn("You listen."),
    );
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(wait_settled(&state, 1000).await, "the first Job settles");
    assert!(
        post_on_last_message(&app, "/swipe/new")
            .await
            .status()
            .is_success()
    );
    assert!(wait_settled(&state, 1000).await, "the retry settles");

    let narration = story_of_game(&state, game)
        .into_iter()
        .rev()
        .find(|message| {
            message.message_type == MessageType::Narration && message.swipe_count() == 2
        })
        .expect("the retry leaves two Swipes on the Narration");

    // A continue saves no Input, so the Narration stays the last Message.
    let hold = model.hold(Role::Narrator);
    assert!(post_action(&app, "").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the held Job should reach its narrator call"
    );
    HeldOnTwoSwipes {
        app,
        state,
        game,
        narration,
        hold,
    }
}

impl HeldOnTwoSwipes {
    async fn release(self) {
        self.hold.release();
        assert!(
            wait_settled(&self.state, 1000).await,
            "the held Job should complete once released"
        );
    }
}

// [docs/specs/swipe_switch.md] SCENARIO: 36.3
#[tokio::test]
async fn test_swipe_switch_while_job_runs_is_refused() {
    let held = held_job_on_two_swipes().await;
    let narration = &held.narration;
    let target = if narration.active_swipe_index == 0 {
        1
    } else {
        0
    };

    let resp = post_empty(
        &held.app,
        &format!("/message/{}/swipe/{target}", narration.id),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let body = response_body(resp).await;
    assert_still_thinking(&body);

    let after = story_of_game(&held.state, held.game)
        .into_iter()
        .find(|message| message.id == narration.id)
        .expect("the Narration stays");
    assert_eq!(
        after.active_swipe_index, narration.active_swipe_index,
        "a refused swipe switch must not change the active Swipe"
    );

    held.release().await;
}

// [docs/specs/swipe_switch.md] SCENARIO: 36.8
#[tokio::test]
async fn test_the_busy_refusal_comes_before_the_other_checks_of_a_swipe_switch() {
    let held = held_job_on_two_swipes().await;
    let before = story_of_game(&held.state, held.game);
    let earlier = before
        .iter()
        .find(|message| message.id != held.narration.id)
        .expect("the story holds a Message before the Narration")
        .id;

    let past_the_last = post_empty(
        &held.app,
        &format!("/message/{}/swipe/2", held.narration.id),
    )
    .await;
    assert_eq!(past_the_last.status(), StatusCode::CONFLICT);
    assert_still_thinking(&response_body(past_the_last).await);

    let not_the_last = post_empty(&held.app, &format!("/message/{earlier}/swipe/0")).await;
    assert_eq!(not_the_last.status(), StatusCode::CONFLICT);
    assert_still_thinking(&response_body(not_the_last).await);

    assert_eq!(
        story_of_game(&held.state, held.game),
        before,
        "a refused swipe switch changes no Message"
    );

    held.release().await;
}

// [docs/specs/games.md] SCENARIO: 18.4
#[tokio::test]
async fn test_a_job_goes_on_for_its_own_game_while_another_game_is_active() {
    let model = Arc::new(
        ScriptedModel::new()
            .narration(GAME_A_NARRATION)
            .after_narration(analysis_report(&[], None))
            .narration(GAME_B_NARRATION)
            .after_narration(analysis_report(&[], None)),
    );
    let hold_a = model.hold(Role::Narrator);
    let (app, state, storage) = app_with_script(Arc::clone(&model));
    let game_a = active_game_of(&state);

    assert!(post_action(&app, "look on A").await.status().is_success());
    assert!(
        hold_a.wait_started(1000),
        "Game A's Job should reach its narrator call"
    );

    let game_b = create_game_from(&app, &state, &storage, game_a, "test").await;

    let hold_b = model.hold(Role::Narrator);
    assert!(post_action(&app, "look on B").await.status().is_success());
    assert!(
        hold_b.wait_started(1000),
        "Game B's Job should reach its narrator call"
    );

    // The call released first takes the first scripted answer.
    hold_a.release();
    assert!(
        hold_a.wait_taken(1000),
        "Game A's call should take its answer"
    );
    hold_b.release();
    assert!(wait_for_jobs(&state, 2000).await, "both Jobs should end");

    let switch_back = post_empty(&app, &format!("/games/{}/switch", game_a.0)).await;
    assert!(switch_back.status().is_success(), "the switch back accepts");
    assert_eq!(active_game_of(&state), game_a, "Game A is active again");

    let a_narrations = texts_of(&state, game_a, MessageType::Narration);
    let b_narrations = texts_of(&state, game_b, MessageType::Narration);
    assert!(
        a_narrations.contains(&GAME_A_NARRATION.to_string()),
        "Game A's story log holds its own Narration, got: {a_narrations:?}"
    );
    assert!(
        !a_narrations.contains(&GAME_B_NARRATION.to_string()),
        "Game A's story log holds no Narration of Game B, got: {a_narrations:?}"
    );
    assert!(
        b_narrations.contains(&GAME_B_NARRATION.to_string()),
        "Game B's story log holds its own Narration, got: {b_narrations:?}"
    );
    assert!(
        !b_narrations.contains(&GAME_A_NARRATION.to_string()),
        "Game B's story log holds no Narration of Game A, got: {b_narrations:?}"
    );
}

const GAME_A_NARRATION: &str = "Game A's narration.";
const GAME_B_NARRATION: &str = "Game B's narration.";
const MAIN_NARRATION: &str = "The main narration.";
const EVENT_NARRATION: &str = "The Character greets you warmly.";
const SECOND_PRESET_TEXT: &str = "SecondPresetMarker";

// [docs/specs/actions.md] SCENARIO: 6.4
#[tokio::test]
async fn test_the_event_narration_call_uses_the_preset_active_when_it_runs() {
    use chronicler_engine::test_support::{empty_sqlite_storage, TestPromptPreset};

    let data = trigger_world();
    let model = Arc::new(triggered_script(MAIN_NARRATION, EVENT_NARRATION));
    let hold = model.hold(Role::Narrator);
    let storage = Arc::new(empty_sqlite_storage());
    let (app, state) = TestAppBuilder::default_test()
        .data(data)
        .storage(Arc::clone(&storage))
        .model(Arc::clone(&model))
        .build_with_state();
    let game = active_game_of(&state);

    storage
        .save_preset(&TestPromptPreset::system(
            "system_second",
            SECOND_PRESET_TEXT,
        ))
        .expect("the second preset seeds");

    assert!(
        post_action(&app, "enter the shop")
            .await
            .status()
            .is_success()
    );
    assert!(
        hold.wait_started(1000),
        "the Job should reach its narrator call"
    );

    let game_row = storage.require_game(game.0).expect("the Game reads back");
    let changed = post_form(
        &app,
        &format!("/games/{}/presets", game.0),
        &format!(
            "system_preset_id=system_second&quantifier_preset_id={}&impersonate_preset_id={}&options_preset_id={}",
            game_row.active_quantifier_prompt_preset_id,
            game_row.active_impersonate_prompt_preset_id,
            game_row.active_options_prompt_preset_id,
        ),
    )
    .await;
    assert_eq!(
        changed.status(),
        StatusCode::OK,
        "the preset change should save"
    );

    hold.release();
    assert!(wait_settled(&state, 2000).await, "the Job should settle");

    let narrations = texts_of(&state, game, MessageType::Narration);
    assert_eq!(
        narrations.iter().skip(1).cloned().collect::<Vec<_>>(),
        vec![MAIN_NARRATION.to_string(), EVENT_NARRATION.to_string()],
        "the Job narrates its own text and the Trigger's, after the opening Narration"
    );

    let calls = state
        .game_view_query
        .list_latest_llm_messages(20)
        .expect("the LLM call records read back");
    let event_call = calls
        .iter()
        .find(|call| call.parsed_response.contains(EVENT_NARRATION))
        .unwrap_or_else(|| panic!("the event Narration's call must be recorded: {calls:?}"));
    assert!(
        sent_system_text(event_call).contains(SECOND_PRESET_TEXT),
        "the event Narration's call must show the system prompt of the preset the client set, got: {}",
        sent_system_text(event_call)
    );
    let main_call = calls
        .iter()
        .find(|call| call.parsed_response.contains(MAIN_NARRATION))
        .unwrap_or_else(|| panic!("the Narration's call must be recorded: {calls:?}"));
    assert!(
        !sent_system_text(main_call).contains(SECOND_PRESET_TEXT),
        "the Narration's call must show the preset that was active when the Job started, got: {}",
        sent_system_text(main_call)
    );
}

fn trigger_world() -> chronicler_engine::test_support::TestData {
    TestDataBuilder::default_test()
        .characters(vec![triggered_character()])
        .build()
}

fn triggered_script(narration: &str, event_narration: &str) -> ScriptedModel {
    ScriptedModel::new()
        .narration(narration)
        .after_narration(analysis_report(&["character_1"], None))
        .event_narration(event_narration)
        .after_narration(analysis_report(&["character_1"], None))
}

fn stoppable_triggered_script(narration: &str, event_narration: &str) -> ScriptedModel {
    ScriptedModel::new()
        .narration(narration)
        .after_narration(analysis_report(&["character_1"], None))
        .optional()
        .event_narration(event_narration)
        .optional()
        .after_narration(analysis_report(&["character_1"], None))
        .optional()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JobKind {
    /// A free-text Action: an Input save, then a Narration generation. A row
    /// whose analysis fires a Trigger runs an event Narration after it.
    Narration { fires_trigger: bool },
    /// `/impersonate`: an Impersonate generation, whose Input comes at its text save.
    Impersonate,
    /// The Retry of a typed Input, which ends at its text save.
    InputRetry,
    /// The event Narration a fired Trigger starts, held at its narrator call.
    EventNarration,
    /// The same event Narration, held at its own analysis call: the second
    /// analysis call of the Job.
    EventNarrationAnalysis,
}

impl JobKind {
    fn label(self) -> String {
        match self {
            Self::Narration {
                fires_trigger: true,
            } => "Narration that fires a Trigger".to_string(),
            Self::Narration {
                fires_trigger: false,
            } => "Narration".to_string(),
            Self::Impersonate => "Impersonate".to_string(),
            Self::InputRetry => "Retry of a typed Input".to_string(),
            Self::EventNarration | Self::EventNarrationAnalysis => "event Narration".to_string(),
        }
    }

    fn action_input(self) -> Option<&'static str> {
        match self {
            Self::Narration { .. } => Some(ACTION_INPUT),
            Self::EventNarration | Self::EventNarrationAnalysis => Some(EVENT_ACTION_INPUT),
            Self::Impersonate | Self::InputRetry => None,
        }
    }
}

const ACTION_INPUT: &str = "look";
const EVENT_ACTION_INPUT: &str = "enter the shop";
const EVENT_TEXT: &str = "The event's narration.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StoryLogAfterStop {
    /// The Input of the Job's plan stays, and the generation saved no Narration.
    PlanInput,
    /// The text save landed: the Input and the Narration of the Job, with no
    /// room on the Narration and no event Narration.
    InputAndNarration,
    /// The state update landed too: the Narration reports the room it arrived
    /// in, and the offered set, which the text save emptied, stays empty.
    InRoom { room: &'static str },
    /// The text save landed as the Input of an Impersonate Job: no Narration,
    /// and the player has not moved.
    ImpersonateInput,
    /// No Message of the Job.
    NoMessage,
    /// The Narration that fired the Trigger stays, and the event Narration
    /// does not join it.
    MainNarrationOnly,
    /// The event Narration's text save landed, with its pre-analysis state: the
    /// one-shot Trigger is marked fired in it.
    EventNarrationWithFiredTrigger,
}

struct StopRow {
    kind: JobKind,
    held: Role,
    status: Option<&'static str>,
    after_stop: StoryLogAfterStop,
}

/// A transaction takes no time a client can observe, so a stop just before one
/// leaves the records of the model call before it.
static STOP_ROWS: &[StopRow] = &[
    StopRow {
        kind: JobKind::Narration {
            fires_trigger: false,
        },
        held: Role::Narrator,
        status: Some("Generating narration..."),
        after_stop: StoryLogAfterStop::PlanInput,
    },
    StopRow {
        kind: JobKind::Narration {
            fires_trigger: true,
        },
        held: Role::Quantifier,
        status: Some("Quantifying..."),
        after_stop: StoryLogAfterStop::InputAndNarration,
    },
    StopRow {
        kind: JobKind::Narration {
            fires_trigger: false,
        },
        held: Role::Options,
        status: Some("Generating options..."),
        after_stop: StoryLogAfterStop::InRoom { room: MOVED_ROOM },
    },
    StopRow {
        kind: JobKind::Impersonate,
        held: Role::Narrator,
        status: Some("Generating narration..."),
        after_stop: StoryLogAfterStop::NoMessage,
    },
    StopRow {
        kind: JobKind::Impersonate,
        held: Role::Quantifier,
        status: Some("Quantifying..."),
        after_stop: StoryLogAfterStop::ImpersonateInput,
    },
    StopRow {
        kind: JobKind::InputRetry,
        held: Role::Narrator,
        status: Some("Generating narration..."),
        after_stop: StoryLogAfterStop::NoMessage,
    },
    StopRow {
        kind: JobKind::EventNarration,
        held: Role::Trigger,
        status: Some("Generating event..."),
        after_stop: StoryLogAfterStop::MainNarrationOnly,
    },
    StopRow {
        kind: JobKind::EventNarrationAnalysis,
        held: Role::Quantifier,
        status: Some("Quantifying..."),
        after_stop: StoryLogAfterStop::EventNarrationWithFiredTrigger,
    },
];

const SCRIPTED_NARRATION: &str = "The Job's narration.";
const WORLD_OPENING_TEXT: &str = "The World begins.";
const MOVED_ROOM: &str = "room2";
const STARTING_ROOM: &str = "room_1";

struct StopJob {
    app: axum::Router,
    state: AppState,
    game: GameId,
    hold: ModelHold,
    story_before: Vec<Message>,
}

async fn start_stop_job(row: &StopRow) -> StopJob {
    match row.kind {
        JobKind::Narration { fires_trigger } => {
            let model = Arc::new(
                if fires_trigger {
                    stoppable_triggered_script(SCRIPTED_NARRATION, SCRIPTED_NARRATION)
                } else {
                    ScriptedModel::new()
                        .narration(SCRIPTED_NARRATION)
                        .after_narration(analysis_report(&[], Some(MOVED_ROOM)))
                        .optional()
                }
                .options(OFFERED_SET)
                .optional(),
            );
            let hold = model.hold(row.held);
            let (app, state, storage) = app_with_script(Arc::clone(&model));
            let game =
                active_game_on_an_always_on_world(&app, &state, &storage, fires_trigger).await;
            let story_before = story_of_game(&state, game);
            assert!(post_action(&app, ACTION_INPUT).await.status().is_success());
            StopJob {
                app,
                state,
                game,
                hold,
                story_before,
            }
        }
        JobKind::Impersonate => {
            let model = Arc::new(
                ScriptedModel::new()
                    .narration(SCRIPTED_NARRATION)
                    .after_narration(analysis_report(&[], Some(MOVED_ROOM)))
                    .optional(),
            );
            let hold = model.hold(row.held);
            let (app, state, storage) = app_with_script(Arc::clone(&model));
            let game = active_game_on_an_always_on_world(&app, &state, &storage, false).await;
            let story_before = story_of_game(&state, game);
            assert!(
                post_action(&app, "/impersonate hello")
                    .await
                    .status()
                    .is_success()
            );
            StopJob {
                app,
                state,
                game,
                hold,
                story_before,
            }
        }
        JobKind::InputRetry => {
            let model = Arc::new(
                ScriptedModel::new()
                    .turn(SCRIPTED_NARRATION)
                    .narration(SCRIPTED_NARRATION)
                    .after_narration(analysis_report(&[], None))
                    .optional(),
            );
            let (app, state, _storage) = app_with_script(Arc::clone(&model));
            let game = active_game_of(&state);
            // A typed Input is the last Message, so `/swipe/new` retries it.
            assert!(post_action(&app, ACTION_INPUT).await.status().is_success());
            assert!(wait_settled(&state, 2000).await, "the first Job settles");
            assert!(
                post_on_last_message(&app, "/history/delete")
                    .await
                    .status()
                    .is_success()
            );
            let story_before = story_of_game(&state, game);
            let hold = model.hold(row.held);
            assert!(
                post_on_last_message(&app, "/swipe/new")
                    .await
                    .status()
                    .is_success()
            );
            StopJob {
                app,
                state,
                game,
                hold,
                story_before,
            }
        }
        JobKind::EventNarration | JobKind::EventNarrationAnalysis => {
            let data = trigger_world();
            let model = Arc::new(stoppable_triggered_script(SCRIPTED_NARRATION, EVENT_TEXT));
            // The Job's first analysis call comes before the event's own, so
            // the hold of the event's call waits behind a hold of the first.
            let first_analysis =
                (row.kind == JobKind::EventNarrationAnalysis).then(|| model.hold(Role::Quantifier));
            let hold_at_row_call = first_analysis.is_none().then(|| model.hold(row.held));
            let (app, state) = TestAppBuilder::default_test()
                .data(data)
                .model(Arc::clone(&model))
                .build_with_state();
            let game = active_game_of(&state);
            let story_before = story_of_game(&state, game);
            assert!(
                post_action(&app, EVENT_ACTION_INPUT)
                    .await
                    .status()
                    .is_success()
            );
            let hold = match (first_analysis, hold_at_row_call) {
                (Some(first), None) => {
                    assert!(
                        first.wait_started(2000),
                        "the first analysis call should reach the model"
                    );
                    let event_analysis = model.hold(Role::Quantifier);
                    first.release();
                    event_analysis
                }
                (None, Some(hold)) => hold,
                _ => unreachable!("a row holds exactly one call"),
            };
            StopJob {
                app,
                state,
                game,
                hold,
                story_before,
            }
        }
    }
}

fn triggered_character() -> chronicler_engine::domain::model::character::CharacterCard {
    use chronicler_engine::domain::model::trigger::{
        ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement,
    };
    use chronicler_engine::test_support::TestPersona;

    chronicler_engine::domain::model::character::CharacterCard {
        id: "character_1".to_string(),
        sheet: TestPersona::standard().sheet,
        inventory: vec![],
        triggers: vec![Trigger {
            requirement: TriggerRequirement {
                operator: ComparisonOperator::Eq,
                threshold: 0,
            },
            narration: TriggerNarration {
                name: "Greeting".to_string(),
                narration_prompt: EVENT_NARRATION.to_string(),
            },
            repeat: false,
            room_id: None,
        }],
        relationships: vec![],
    }
}

// [docs/specs/dashboard.md] SCENARIO: 39.4
#[tokio::test]
async fn test_a_stop_at_each_model_call_keeps_the_records_of_that_step() {
    let mut failures: Vec<String> = Vec::new();
    for row in STOP_ROWS {
        if let Err(join_error) = tokio::spawn(run_stop_row(row)).await {
            failures.push(join_error.to_string());
        }
    }
    assert!(
        failures.is_empty(),
        "a stop did not keep the records of the design's table at {} stop point(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

async fn run_stop_row(row: &'static StopRow) {
    let label = format!("{} at the {} call", row.kind.label(), row.held.agent_name());
    let start = start_stop_job(row).await;
    assert!(
        start.hold.wait_started(2000),
        "{label}: the Job should reach the held call"
    );

    if let Some(status) = row.status {
        assert_eq!(
            fetch_status_label(&start.app).await,
            status,
            "{label}: the status display names the step the call runs in"
        );
    }

    let stop = post_empty(&start.app, "/status/reset-busy").await;
    assert_eq!(
        response_body(stop).await,
        "reset",
        "{label}: a stop answers reset"
    );
    assert!(
        wait_idle(&start.state, 2000).await,
        "{label}: the Game should end idle at once"
    );
    assert_eq!(
        fetch_status_label(&start.app).await,
        "Ready",
        "{label}: a stopped Game is idle, with no failure line"
    );

    start.hold.release();
    assert!(
        wait_for_jobs(&start.state, 2000).await,
        "{label}: the Job should end"
    );

    assert_story_log_after_stop(row, &label, &start).await;
}

async fn assert_story_log_after_stop(row: &StopRow, label: &str, start: &StopJob) {
    let after = story_of_game(&start.state, start.game);
    let after_texts: Vec<String> = after
        .iter()
        .map(|message| message.text().to_string())
        .collect();
    let before_texts: Vec<String> = start
        .story_before
        .iter()
        .map(|message| message.text().to_string())
        .collect();
    let narrations: Vec<&Message> = after
        .iter()
        .filter(|message| message.message_type == MessageType::Narration)
        .collect();
    let job_narration = narrations
        .iter()
        .find(|message| message.text() == SCRIPTED_NARRATION);
    let plan_input = row.kind.action_input();

    match row.after_stop {
        StoryLogAfterStop::PlanInput => {
            let input = plan_input.expect("this row's Job saves a plan Input");
            assert_eq!(
                texts_of(&start.state, start.game, MessageType::Input),
                vec![input.to_string()],
                "{label}: the Input of the Job stays, and nothing else"
            );
            assert_eq!(
                texts_of(&start.state, start.game, MessageType::Narration),
                vec![WORLD_OPENING_TEXT.to_string()],
                "{label}: the cancelled narrator call saves no Narration"
            );
        }
        StoryLogAfterStop::InputAndNarration => {
            let input = plan_input.expect("this row's Job saves a plan Input");
            assert_eq!(
                texts_of(&start.state, start.game, MessageType::Input),
                vec![input.to_string()],
                "{label}: the Input of the Job stays"
            );
            assert_eq!(
                texts_of(&start.state, start.game, MessageType::Narration),
                vec![
                    WORLD_OPENING_TEXT.to_string(),
                    SCRIPTED_NARRATION.to_string()
                ],
                "{label}: the text stays, and no event Narration joins it"
            );
            let narration = job_narration.expect("the text save wrote the Narration");
            assert_eq!(
                room_header_of(&start.state, narration.id),
                None,
                "{label}: the pre-analysis state is the start state, so the player has not moved"
            );
            assert_eq!(
                narration.event_header(),
                None,
                "{label}: the stop ends the Job before a Trigger's event Narration"
            );
        }
        StoryLogAfterStop::InRoom { room } => {
            let input = plan_input.expect("this row's Job saves a plan Input");
            assert_eq!(
                texts_of(&start.state, start.game, MessageType::Input),
                vec![input.to_string()],
                "{label}: the Input of the Job stays"
            );
            assert_eq!(
                texts_of(&start.state, start.game, MessageType::Narration),
                vec![
                    WORLD_OPENING_TEXT.to_string(),
                    SCRIPTED_NARRATION.to_string()
                ],
                "{label}: the text stays"
            );
            let narration = job_narration.expect("the text save wrote the Narration");
            let expected = format!("Room {room}");
            assert_eq!(
                room_header_of(&start.state, narration.id).as_deref(),
                Some(expected.as_str()),
                "{label}: the state update landed, so the Narration shows the room it arrived in"
            );
            let dock = fetch_part(&start.app, "options-dock").await;
            assert!(
                !dock.contains("options-strip"),
                "{label}: the text save emptied the offered set, and the refused Options run leaves it empty, got: {dock}"
            );
        }
        StoryLogAfterStop::ImpersonateInput => {
            assert_eq!(
                after_texts,
                vec![
                    WORLD_OPENING_TEXT.to_string(),
                    SCRIPTED_NARRATION.to_string()
                ],
                "{label}: the Impersonate Job saves its Input and no Narration of its own"
            );
            let impersonate_input = after
                .iter()
                .find(|message| message.message_type == MessageType::Input)
                .expect("the text save wrote the Input");
            assert_eq!(
                room_header_of(&start.state, impersonate_input.id),
                None,
                "{label}: the pre-analysis state is the start state, so the Input shows no room"
            );
            let room = start
                .state
                .game_view_query
                .get_debug_state(start.game)
                .expect("the debug state reads back")
                .current_room_id;
            assert_eq!(
                room, STARTING_ROOM,
                "{label}: the refused state update leaves the player in the room the Job started in"
            );
        }
        StoryLogAfterStop::NoMessage => {
            assert_eq!(
                after_texts, before_texts,
                "{label}: a stopped Job saves no Message of its own"
            );
        }
        StoryLogAfterStop::MainNarrationOnly => {
            let input = plan_input.expect("this row's Job saves a plan Input");
            assert_eq!(
                texts_of(&start.state, start.game, MessageType::Input),
                vec![input.to_string()],
                "{label}: the Input of the Job stays"
            );
            assert_eq!(
                narrations.len(),
                start
                    .story_before
                    .iter()
                    .filter(|message| message.message_type == MessageType::Narration)
                    .count()
                    + 1,
                "{label}: the Narration that fired the Trigger stays, and the event Narration does not join it"
            );
            assert!(
                narrations
                    .iter()
                    .all(|message| message.event_header().is_none()),
                "{label}: no event Narration is saved after the stop"
            );
        }
        StoryLogAfterStop::EventNarrationWithFiredTrigger => {
            let input = plan_input.expect("this row's Job saves a plan Input");
            assert_eq!(
                texts_of(&start.state, start.game, MessageType::Input),
                vec![input.to_string()],
                "{label}: the Input of the Job stays"
            );
            assert_eq!(
                texts_of(&start.state, start.game, MessageType::Narration),
                before_texts
                    .iter()
                    .cloned()
                    .chain([SCRIPTED_NARRATION.to_string(), EVENT_TEXT.to_string()])
                    .collect::<Vec<_>>(),
                "{label}: the Narration that fired the Trigger and the text of the event Narration stay"
            );
            let debug = start
                .state
                .game_view_query
                .get_debug_state(start.game)
                .expect("the debug state reads back");
            assert_eq!(
                debug
                    .encounters
                    .get("character_1")
                    .and_then(|encounter| encounter.trigger_fired.get(&0))
                    .copied(),
                Some(true),
                "{label}: the one-shot Trigger is fired in the pre-analysis state the stop keeps"
            );
        }
    }
}

async fn active_game_on_an_always_on_world(
    app: &axum::Router,
    state: &AppState,
    storage: &Arc<Storage>,
    fires_trigger: bool,
) -> GameId {
    use chronicler_engine::domain::model::scenario::StartingScenario;
    use chronicler_engine::domain::model::world::WorldCard;
    use chronicler_engine::test_support::TestMap;

    let world = WorldCard {
        key: "always_on_world".to_string(),
        name: "Always On World".to_string(),
        description: "A World whose turn-end Options run is always on.".to_string(),
        options_always_on: true,
        scenarios: vec![StartingScenario {
            id: "start".to_string(),
            name: "Start".to_string(),
            description: "The World's opening.".to_string(),
            starting_room_id: STARTING_ROOM.to_string(),
            text: WORLD_OPENING_TEXT.to_string(),
            characters_present: vec![],
        }],
        ..Default::default()
    };
    let world_id = storage
        .seed_world(&world, &TestMap::two_rooms(STARTING_ROOM, MOVED_ROOM))
        .expect("the always-on World seeds");
    if fires_trigger {
        storage
            .seed_character(world_id, &triggered_character())
            .expect("the Character of the Trigger seeds");
    }
    create_game_from(app, state, storage, active_game_of(state), &world.key).await
}
