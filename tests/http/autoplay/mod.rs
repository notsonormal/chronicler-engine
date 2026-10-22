//! HTTP E2E tests for Autoplay (autoplay.md 41.*), shared helpers: `run` holds the run of Jobs, its ends, the progress status and the refused writes; `fragments` holds the view-only Game's rendering.

mod fragments;
mod run;

use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::application::autoplay::pause::PauseHandle;
use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::domain::model::game::GameId;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::ScriptedModel;

use crate::support::http_fixtures::story_of_game;
use crate::support::http_requests::{
    assert_no_unscripted_model_call, fetch_generating_status, post_empty, post_form, response_body,
    wait_for_jobs,
};

const FIRST_INPUT: &str = "I try the door.";
const FIRST_NARRATION: &str = "The door creaks open.";
const SECOND_INPUT: &str = "I step inside.";
const SECOND_NARRATION: &str = "A cold hall waits.";
const LATE_ANSWER: &str = "The late answer.";

async fn start_autoplay(app: &axum::Router, body: &str) -> axum::response::Response {
    post_form(app, "/autoplay", body).await
}

async fn wait_for_run(state: &AppState, game: GameId) {
    assert!(wait_for_jobs(state, 5000).await, "the run should end");
    assert!(
        !state.game_view_query.is_busy(game),
        "an ended run leaves its Game idle"
    );
    assert_no_unscripted_model_call(state);
}

fn story_after_opening(state: &AppState, game: GameId) -> Vec<(MessageType, String)> {
    story_of_game(state, game)
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

async fn pause_before_first_job(app: &axum::Router, state: &AppState, game: GameId) -> PauseHandle {
    let pause = state.job_deps.pause.hold(game);
    assert!(start_autoplay(app, "cycles=2").await.status().is_success());
    assert!(
        pause.wait_paused(5000),
        "the run should reach its pause before its first Job"
    );
    pause
}

/// The model must script the Impersonate Job of the first cycle.
async fn pause_between_two_jobs(app: &axum::Router, state: &AppState, game: GameId) -> PauseHandle {
    let before_first = pause_before_first_job(app, state, game).await;
    // The first hold has fired, so the second goes in while the loop waits.
    let between = state.job_deps.pause.hold(game);
    before_first.release();
    assert!(
        between.wait_paused(5000),
        "the run should reach its pause between its first Job and its second"
    );
    between
}

async fn end_paused_run(app: &axum::Router, state: &AppState, game: GameId, pause: &PauseHandle) {
    stop(app).await;
    pause.release();
    wait_for_run(state, game).await;
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
