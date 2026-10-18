//! HTTP E2E tests for the write path while a Job runs: the races (actions.md 3.4, 3.5; options.md 24.21), the stop at each model call (dashboard.md 39.4, 39.5), the reset (reset.md 7.3), the Job of another Game (games.md 18.4), the edit, delete and swipe switch (story_log.md 8.7; swipe_switch.md 36.3), and the preset an event Narration uses (actions.md 6.4).

use std::sync::Arc;

use axum::http::StatusCode;
use tokio::sync::Barrier;

use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::domain::model::game::GameId;
use chronicler_engine::domain::model::message::Message;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::{
    active_game_of, analysis_report, ModelHold, ModelRole, ScriptedModel, TestAppBuilder,
    TestDataBuilder, TestMap,
};

use crate::support::app_wiring::{app_with_model, app_with_script};
use crate::support::http_assertions::llm_call_count;
use crate::support::http_fixtures::OFFERED_SET;
use crate::support::http_requests::{
    fetch_body, post_action, post_empty, post_form, response_body, wait_for_jobs, wait_idle,
    wait_settled,
};

const STILL_THINKING: &str = "Still thinking...";

/// A held Job: the busy state a second request meets.
fn held_action() -> (Arc<ScriptedModel>, ModelHold) {
    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let hold = model.hold(ModelRole::Narration);
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
    assert_eq!(refused.status(), StatusCode::OK);
    let body = response_body(refused).await;
    assert!(
        body.contains(STILL_THINKING),
        "the refused Action must answer the wait state, got: {body}"
    );
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
    let narrations = state
        .message_service
        .load_messages(active_game_of(&state))
        .expect("the story log reads back")
        .into_iter()
        .filter(|message| message.message_type == MessageType::Narration)
        .count();
    assert_eq!(
        narrations, 1,
        "the refused Action must narrate nothing of its own"
    );
}

// [docs/specs/actions.md] SCENARIO: 3.5
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_two_actions_at_the_same_moment_save_one_input_http() {
    let (model, hold) = held_action();
    let (app, state) = app_with_model(Arc::clone(&model));
    let barrier = Arc::new(Barrier::new(2));

    // Both clicks carry the same Action, released together, so which of them
    // wins is the generation slot's to decide and not the scheduler's.
    let click = |app: axum::Router, barrier: Arc<Barrier>| async move {
        barrier.wait().await;
        let resp = post_action(&app, "look").await;
        assert!(
            resp.status().is_success(),
            "an accepted Action and a refused one both answer 200"
        );
        response_body(resp).await
    };
    let first = tokio::spawn(click(app.clone(), Arc::clone(&barrier)));
    let second = tokio::spawn(click(app.clone(), Arc::clone(&barrier)));
    let bodies = [
        first.await.expect("the first click joins"),
        second.await.expect("the second click joins"),
    ];

    let refused = bodies
        .iter()
        .filter(|body| body.contains(STILL_THINKING))
        .count();
    assert_eq!(refused, 1, "exactly one click is refused: {bodies:?}");
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
    assert_eq!(refused.status(), StatusCode::OK);
    let body = response_body(refused).await;
    assert!(
        body.contains(STILL_THINKING),
        "the refused /options must answer the wait state, got: {body}"
    );

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
    let dock = fetch_body(&app, "/fragment/options-dock").await;
    assert!(
        !dock.contains("options-strip"),
        "the refused /options must leave the dock empty: {dock}"
    );
}

fn messages_of(state: &AppState, game: GameId) -> Vec<Message> {
    state
        .message_service
        .load_messages(game)
        .expect("the story log reads back")
}

fn texts_of(state: &AppState, game: GameId, kind: MessageType) -> Vec<String> {
    messages_of(state, game)
        .into_iter()
        .filter(|message| message.message_type == kind)
        .map(|message| message.text().to_string())
        .collect()
}

/// The one narration answer both Jobs of a stop case script, so which Job takes
/// which answer cannot change the assertions.
const STOPPED_NARRATION: &str = "The stopped Job's narration.";
const NEXT_NARRATION: &str = "The next Job's narration.";

// [docs/specs/dashboard.md] SCENARIO: 39.5
#[ignore = "red until the core step: a stopped job's late answer"]
#[tokio::test]
async fn test_a_stopped_job_writes_nothing_and_the_next_job_saves_its_own_text() {
    let model = Arc::new(
        ScriptedModel::new()
            .narration(STOPPED_NARRATION)
            .analysis(analysis_report(&[], None))
            .narration(NEXT_NARRATION)
            .analysis(analysis_report(&[], None)),
    );
    let stopped_hold = model.hold(ModelRole::Narration);
    let (app, state, _storage) = app_with_script(Arc::clone(&model));
    let game = active_game_of(&state);

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        stopped_hold.wait_started(1000),
        "the first Job should reach its narrator call"
    );

    let stop = post_empty(&app, "/status/reset-generating").await;
    assert_eq!(
        response_body(stop).await,
        "reset",
        "a stop of a busy Game answers reset"
    );

    let next_hold = model.hold(ModelRole::Narration);
    assert!(post_action(&app, "listen").await.status().is_success());
    assert!(
        next_hold.wait_started(1000),
        "the second Job should reach its narrator call"
    );

    // The stopped Job's answer arrives first, and both Jobs end, so the reads
    // below see what its late answer did write, and not a moment before it
    // tried. The call released first takes the first scripted answer.
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
    assert_eq!(
        texts_of(&state, game, MessageType::Narration),
        vec![NEXT_NARRATION.to_string()],
        "the Narration after listen is the next Job's text"
    );
}

// [docs/specs/reset.md] SCENARIO: 7.3
#[tokio::test]
async fn test_reset_while_job_runs_keeps_only_the_new_game_opening() {
    let model = Arc::new(ScriptedModel::new().turn("The stopped narration."));
    let hold = model.hold(ModelRole::Narration);
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
    // The reset ended the Job's Game, so wait for the Job to end before reading
    // what it wrote.
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
    let status = fetch_body(&app, "/status/generating").await;
    assert_eq!(
        status.trim(),
        "idle",
        "the new Game answers idle, with no failure line"
    );
}

// [docs/specs/story_log.md] SCENARIO: 8.7
#[ignore = "red until the core step: an edit and a delete while busy"]
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
    let before = messages_of(&state, game);
    let target = before.last().expect("the Job saved its Input").id;

    let edit = post_form(&app, &format!("/history/{target}"), "text=An+edited+text").await;
    assert_eq!(edit.status(), StatusCode::OK);
    let edit_body = response_body(edit).await;
    assert!(
        edit_body.contains(STILL_THINKING),
        "an edit while the Game is busy must answer the wait state, got: {edit_body}"
    );

    let delete = post_empty(&app, "/history/delete").await;
    assert_eq!(delete.status(), StatusCode::OK);
    let delete_body = response_body(delete).await;
    assert!(
        delete_body.contains(STILL_THINKING),
        "a delete while the Game is busy must answer the wait state, got: {delete_body}"
    );

    let after = messages_of(&state, game);
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

// [docs/specs/swipe_switch.md] SCENARIO: 36.3
#[ignore = "red until the core step: a swipe switch while busy"]
#[tokio::test]
async fn test_swipe_switch_while_job_runs_is_refused() {
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
    assert!(post_empty(&app, "/swipe/new").await.status().is_success());
    assert!(wait_settled(&state, 1000).await, "the retry settles");

    let narration = messages_of(&state, game)
        .into_iter()
        .rev()
        .find(|message| {
            message.message_type == MessageType::Narration && message.swipe_count() == 2
        })
        .expect("the retry leaves two Swipes on the Narration");
    let target = if narration.active_swipe_index == 0 {
        1
    } else {
        0
    };

    // A continue holds a Job without saving an Input of its own, so the
    // Narration with two Swipes is still the Message that a switch names.
    let hold = model.hold(ModelRole::Narration);
    assert!(post_action(&app, "").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the held Job should reach its narrator call"
    );

    let resp = post_empty(&app, &format!("/message/{}/swipe/{target}", narration.id)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = response_body(resp).await;
    assert!(
        body.contains(STILL_THINKING),
        "a swipe switch while the Game is busy must answer the wait state, got: {body}"
    );

    let after = messages_of(&state, game)
        .into_iter()
        .find(|message| message.id == narration.id)
        .expect("the Narration stays");
    assert_eq!(
        after.active_swipe_index, narration.active_swipe_index,
        "a refused swipe switch must not change the active Swipe"
    );

    hold.release();
    assert!(
        wait_settled(&state, 1000).await,
        "the held Job should complete once released"
    );
}

// [docs/specs/games.md] SCENARIO: 18.4
#[ignore = "red until the core step: a job on another game"]
#[tokio::test]
async fn test_a_job_goes_on_for_its_own_game_while_another_game_is_active() {
    let model = Arc::new(
        ScriptedModel::new()
            .narration(GAME_A_NARRATION)
            .analysis(analysis_report(&[], None))
            .narration(GAME_B_NARRATION)
            .analysis(analysis_report(&[], None)),
    );
    let hold_a = model.hold(ModelRole::Narration);
    let (app, state, storage) = app_with_script(Arc::clone(&model));
    let game_a = active_game_of(&state);

    assert!(post_action(&app, "look on A").await.status().is_success());
    assert!(
        hold_a.wait_started(1000),
        "Game A's Job should reach its narrator call"
    );

    // The client moves to Game B, created from Game A's World, and sends an
    // Action there, so both Games are busy at the same moment.
    let persona_key = storage
        .require_game(game_a.0)
        .expect("Game A reads back")
        .persona_key;
    let created = post_form(
        &app,
        "/games",
        &format!("world_key=test&persona_key={persona_key}"),
    )
    .await;
    assert!(created.status().is_success(), "Game B should be created");
    let game_b = active_game_of(&state);
    assert_ne!(game_b, game_a, "the create makes Game B active");

    let hold_b = model.hold(ModelRole::Narration);
    assert!(post_action(&app, "look on B").await.status().is_success());
    assert!(
        hold_b.wait_started(1000),
        "Game B's Job should reach its narrator call"
    );

    // Game A's answer arrives while Game B is active and busy: it saves into
    // Game A. The call released first takes the first scripted answer.
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
const EVENT_NARRATION: &str = "The NPC greets you warmly.";
/// The preset the client sets while the Job's narrator call is held.
const SECOND_PRESET_TEXT: &str = "SecondPresetMarker";

// [docs/specs/actions.md] SCENARIO: 6.4
#[tokio::test]
async fn test_the_event_narration_call_uses_the_preset_active_when_it_runs() {
    use chronicler_engine::test_support::{empty_sqlite_storage, TestPromptPreset};

    let data = trigger_world();
    let model = Arc::new(triggered_script(MAIN_NARRATION, EVENT_NARRATION));
    let hold = model.hold(ModelRole::Narration);
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

    // The client's system-preset change lands while the narrator call is held,
    // so the Job's own Narration started under the preset before it.
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
    assert!(
        changed.status().is_success(),
        "the preset change should save"
    );

    hold.release();
    assert!(wait_settled(&state, 2000).await, "the Job should settle");

    assert_eq!(
        texts_of(&state, game, MessageType::Narration),
        vec![MAIN_NARRATION.to_string(), EVENT_NARRATION.to_string()],
        "the Job narrates its own text and the Trigger's"
    );

    // The records the panel shows: the event Narration's call carries the
    // system prompt of the preset the client set, and the Narration's call the
    // preset that was active when the Job started.
    let calls = state
        .game_view_query
        .list_latest_llm_messages(20)
        .expect("the LLM call records read back");
    let event_call = calls
        .iter()
        .find(|call| call.parsed_response.contains(EVENT_NARRATION))
        .unwrap_or_else(|| panic!("the event Narration's call must be recorded: {calls:?}"));
    assert!(
        event_call.system_prompt.contains(SECOND_PRESET_TEXT),
        "the event Narration's call must show the system prompt of the preset the client set, got: {}",
        event_call.system_prompt
    );
    let main_call = calls
        .iter()
        .find(|call| call.parsed_response.contains(MAIN_NARRATION))
        .unwrap_or_else(|| panic!("the Narration's call must be recorded: {calls:?}"));
    assert!(
        !main_call.system_prompt.contains(SECOND_PRESET_TEXT),
        "the Narration's call must show the preset that was active when the Job started, got: {}",
        main_call.system_prompt
    );
}

/// The World data of the Turn whose analysis fires the Character's Trigger.
fn trigger_world() -> chronicler_engine::test_support::TestData {
    TestDataBuilder::default_test()
        .npcs(vec![triggered_npc()])
        .build()
}

/// The four model calls of that Turn: the Narration, its analysis, the event
/// Narration, and the event's analysis.
fn triggered_script(narration: &str, event_narration: &str) -> ScriptedModel {
    ScriptedModel::new()
        .narration(narration)
        .analysis(analysis_report(&["npc_1"], None))
        .event_narration(event_narration)
        .analysis(analysis_report(&["npc_1"], None))
}

/// The kind of Job the stop table walks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JobKind {
    /// A free-text Action: an Input save, then a Narration generation. A row
    /// whose analysis fires a Trigger runs an event Narration after it.
    Narration { fires_trigger: bool },
    /// `/impersonate`: an Impersonate generation, whose Input comes at its text save.
    Impersonate,
    /// A retry of a typed Input: a user regen, which ends at its text save.
    TypedInputRetry,
    /// The event Narration a fired Trigger starts.
    EventNarration,
}

impl JobKind {
    /// The words a failing row reports.
    fn label(self) -> String {
        match self {
            Self::Narration {
                fires_trigger: true,
            } => "Narration that fires a Trigger".to_string(),
            Self::Narration {
                fires_trigger: false,
            } => "Narration".to_string(),
            Self::Impersonate => "Impersonate".to_string(),
            Self::TypedInputRetry => "Retry of a typed Input".to_string(),
            Self::EventNarration => "event Narration".to_string(),
        }
    }

    /// The text of the Action the Job's request sends, and so the text of the
    /// Input its plan saves before the generation. `None` for a request that
    /// sends no Action: an Impersonate and a retry save no Input before their
    /// generation.
    fn action_input(self) -> Option<&'static str> {
        match self {
            Self::Narration { .. } => Some(ACTION_INPUT),
            Self::EventNarration => Some(EVENT_ACTION_INPUT),
            Self::Impersonate | Self::TypedInputRetry => None,
        }
    }
}

/// The free-text Action every Narration row sends, and the Input it saves.
const ACTION_INPUT: &str = "look";
/// The Action of the row whose analysis fires the Trigger, and the Input it saves.
const EVENT_ACTION_INPUT: &str = "enter the shop";

/// What the story log holds after a stop, per the row of the design's
/// generation table.
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
}

struct StopRow {
    kind: JobKind,
    held: ModelRole,
    /// The step the status poll answers while the call is held, where the
    /// design names the step.
    status: Option<&'static str>,
    after_stop: StoryLogAfterStop,
}

/// The rows of the design's stop table, one per (kind of Job, model call) the
/// fake can hold. A save unit takes no time a client can observe, so a stop
/// just before one leaves the records of the model call before it.
static STOP_ROWS: &[StopRow] = &[
    StopRow {
        kind: JobKind::Narration {
            fires_trigger: false,
        },
        held: ModelRole::Narration,
        status: Some("narrating"),
        after_stop: StoryLogAfterStop::PlanInput,
    },
    StopRow {
        kind: JobKind::Narration {
            fires_trigger: true,
        },
        held: ModelRole::Analysis,
        status: Some("quantifying"),
        after_stop: StoryLogAfterStop::InputAndNarration,
    },
    StopRow {
        kind: JobKind::Narration {
            fires_trigger: false,
        },
        held: ModelRole::Options,
        // The design names the status of the generation's own steps only.
        status: None,
        after_stop: StoryLogAfterStop::InRoom { room: MOVED_ROOM },
    },
    StopRow {
        kind: JobKind::Impersonate,
        held: ModelRole::Narration,
        status: Some("narrating"),
        after_stop: StoryLogAfterStop::NoMessage,
    },
    StopRow {
        kind: JobKind::Impersonate,
        held: ModelRole::Analysis,
        status: Some("quantifying"),
        after_stop: StoryLogAfterStop::ImpersonateInput,
    },
    StopRow {
        kind: JobKind::TypedInputRetry,
        held: ModelRole::Narration,
        status: Some("narrating"),
        after_stop: StoryLogAfterStop::NoMessage,
    },
    StopRow {
        kind: JobKind::EventNarration,
        held: ModelRole::Event,
        status: Some("generating-event"),
        after_stop: StoryLogAfterStop::MainNarrationOnly,
    },
];

const SCRIPTED_NARRATION: &str = "The Job's narration.";
const WORLD_OPENING_TEXT: &str = "The World begins.";
/// The room the scripted analysis moves the player to.
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
                    triggered_script(SCRIPTED_NARRATION, SCRIPTED_NARRATION)
                } else {
                    ScriptedModel::new()
                        .narration(SCRIPTED_NARRATION)
                        .analysis(analysis_report(&[], Some(MOVED_ROOM)))
                }
                .options(OFFERED_SET),
            );
            let hold = model.hold(row.held);
            let (app, state, storage) = app_with_script(Arc::clone(&model));
            let game =
                active_game_on_an_always_on_world(&app, &state, &storage, fires_trigger).await;
            let story_before = messages_of(&state, game);
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
            // The two-room map lets the scripted analysis move the player, so
            // the row can show that the pre-analysis state kept the room.
            let data = TestDataBuilder::default_test()
                .map(TestMap::two_rooms(STARTING_ROOM, MOVED_ROOM))
                .build();
            let model = Arc::new(
                ScriptedModel::new()
                    .narration(SCRIPTED_NARRATION)
                    .analysis(analysis_report(&[], Some(MOVED_ROOM))),
            );
            let hold = model.hold(row.held);
            let (app, state) = TestAppBuilder::default_test()
                .data(data)
                .model(Arc::clone(&model))
                .build_with_state();
            let game = active_game_of(&state);
            let story_before = messages_of(&state, game);
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
        JobKind::TypedInputRetry => {
            let model = Arc::new(
                ScriptedModel::new()
                    .turn(SCRIPTED_NARRATION)
                    .turn(SCRIPTED_NARRATION),
            );
            let (app, state, _storage) = app_with_script(Arc::clone(&model));
            let game = active_game_of(&state);
            // A typed Input is the last Message of the story, so `/swipe/new` is
            // a retry of it.
            assert!(post_action(&app, ACTION_INPUT).await.status().is_success());
            assert!(wait_settled(&state, 2000).await, "the first Job settles");
            assert!(
                post_empty(&app, "/history/delete")
                    .await
                    .status()
                    .is_success()
            );
            let story_before = messages_of(&state, game);
            let hold = model.hold(row.held);
            assert!(post_empty(&app, "/swipe/new").await.status().is_success());
            StopJob {
                app,
                state,
                game,
                hold,
                story_before,
            }
        }
        JobKind::EventNarration => {
            let data = trigger_world();
            let model = Arc::new(triggered_script(SCRIPTED_NARRATION, SCRIPTED_NARRATION));
            let hold = model.hold(row.held);
            let (app, state) = TestAppBuilder::default_test()
                .data(data)
                .model(Arc::clone(&model))
                .build_with_state();
            let game = active_game_of(&state);
            let story_before = messages_of(&state, game);
            assert!(
                post_action(&app, EVENT_ACTION_INPUT)
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
    }
}

/// The Character whose Trigger fires on the analysis that first sees it in the
/// room, which starts an event Narration.
fn triggered_npc() -> chronicler_engine::domain::model::character::NpcCard {
    use chronicler_engine::domain::model::trigger::{
        ComparisonOperator, Trigger, TriggerNarration, TriggerRequirement,
    };
    use chronicler_engine::test_support::TestPersona;

    chronicler_engine::domain::model::character::NpcCard {
        id: "npc_1".to_string(),
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
#[ignore = "red until the core step: a stop at each model call"]
#[tokio::test]
async fn test_a_stop_at_each_model_call_keeps_the_records_of_that_step() {
    // Every row runs, so one report names every stop point that broke.
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
    let label = format!("{} at the {} call", row.kind.label(), row.held.label());
    let start = start_stop_job(row).await;
    assert!(
        start.hold.wait_started(2000),
        "{label}: the Job should reach the held call"
    );

    if let Some(status) = row.status {
        let answered = fetch_body(&start.app, "/status/generating").await;
        assert_eq!(
            answered.trim(),
            status,
            "{label}: the status poll names the step the call runs in"
        );
    }

    let stop = post_empty(&start.app, "/status/reset-generating").await;
    assert_eq!(
        response_body(stop).await,
        "reset",
        "{label}: a stop answers reset"
    );
    // The Game is idle at once, before the held call is released.
    assert!(
        wait_idle(&start.state, 2000).await,
        "{label}: the Game should end idle at once"
    );
    let status = fetch_body(&start.app, "/status/generating").await;
    assert_eq!(
        status.trim(),
        "idle",
        "{label}: a stopped Game is idle, with no failure line"
    );

    start.hold.release();
    // The Job ended, so the reads below see what its late answer wrote, and not
    // a moment before it tried.
    assert!(
        wait_for_jobs(&start.state, 2000).await,
        "{label}: the Job should end"
    );

    assert_story_log_after_stop(row, &label, &start).await;
}

async fn assert_story_log_after_stop(row: &StopRow, label: &str, start: &StopJob) {
    let after = messages_of(&start.state, start.game);
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
                narration.room_report(),
                None,
                "{label}: the pre-analysis state names no room, so the player has not moved"
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
                narration.room_report(),
                Some(expected.as_str()),
                "{label}: the state update landed, so the Narration reports the room it arrived in"
            );
            let dock = fetch_body(&start.app, "/fragment/options-dock").await;
            assert!(
                !dock.contains("options-strip"),
                "{label}: the text save emptied the offered set, and the refused Options run leaves it empty, got: {dock}"
            );
        }
        StoryLogAfterStop::ImpersonateInput => {
            assert_eq!(
                texts_of(&start.state, start.game, MessageType::Input),
                vec![SCRIPTED_NARRATION.to_string()],
                "{label}: the text save of the Impersonate Job stays"
            );
            assert!(
                narrations.is_empty(),
                "{label}: an Impersonate Job saves no Narration, got: {narrations:?}"
            );
            let impersonate_input = after
                .iter()
                .find(|message| message.message_type == MessageType::Input)
                .expect("the text save wrote the Input");
            assert_eq!(
                impersonate_input.room_report(),
                None,
                "{label}: the pre-analysis state reports no room"
            );
            let room = start
                .state
                .message_service
                .load_or_fresh(start.game)
                .swipe
                .movement
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
    }
}

/// Points the app at a Game on an always-on World with two rooms: the Job's
/// Options call exists, the analysis can move the player, and the opening
/// Narration takes the start state's room. `fires_trigger` seeds the Character
/// whose Trigger the scripted analysis fires.
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
            npcs: vec![],
        }],
        ..Default::default()
    };
    let world_id = storage
        .seed_world(&world, &TestMap::two_rooms(STARTING_ROOM, MOVED_ROOM))
        .expect("the always-on World seeds");
    if fires_trigger {
        storage
            .seed_character(world_id, &triggered_npc())
            .expect("the Character of the Trigger seeds");
    }
    let persona_key = storage
        .require_game(active_game_of(state).0)
        .expect("the Game reads back")
        .persona_key;
    let created = post_form(
        app,
        "/games",
        &format!("world_key={}&persona_key={persona_key}", world.key),
    )
    .await;
    assert!(
        created.status().is_success(),
        "the always-on Game should be created"
    );
    active_game_of(state)
}
