//! HTTP E2E tests for the write-path races: a second request while a Job holds the slot. Cases: an Action while a Job runs (actions.md 3.4), two clicks at one moment (actions.md 3.5), /options while a Job runs (options.md 24.21).

use std::sync::Arc;

use axum::http::StatusCode;
use tokio::sync::Barrier;

use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::{active_game_of, ModelHold, ModelRole, ScriptedModel};

use crate::support::app_wiring::app_with_model;
use crate::support::http_assertions::llm_call_count;
use crate::support::http_requests::{fetch_body, post_action, response_body, wait_settled};

const STILL_THINKING: &str = "Still thinking...";

/// Every player-input message in the story log, in order.
fn input_texts(state: &AppState) -> Vec<String> {
    state
        .message_service
        .load_messages(active_game_of(state))
        .expect("the story log reads back")
        .into_iter()
        .filter(|message| message.message_type == MessageType::Input)
        .map(|message| message.text().to_string())
        .collect()
}

/// A Job whose narration call never answers until the test releases it: the
/// busy state a second request meets.
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
        input_texts(&state),
        vec!["look".to_string()],
        "the refused Action must save no input of its own"
    );

    hold.release();
    assert!(
        wait_settled(&state, 1000).await,
        "the held Action should complete once released"
    );
    assert_eq!(
        input_texts(&state),
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
        input_texts(&state),
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
