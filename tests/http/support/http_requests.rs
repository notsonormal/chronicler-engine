//! HTTP request plumbing for the http test binary: POST builders, body readers, generation-idle polling.

use chronicler_engine::application::pipeline::pipeline_run::UNCERTAIN_NPC_NOTICE;
use chronicler_engine::test_support::active_game_of;
use chronicler_engine::test_support::STRAY_CALL_MARKER;
use axum::body::Body;
use axum::http::{self, Method, Request};
use tower::util::ServiceExt;

use chronicler_engine::adapters::driving::http::AppState;

use crate::test_utils::wait_for_condition_async;

/// One cap covers every fragment and page this suite reads; a body over it
/// fails the read.
pub async fn response_body(resp: axum::response::Response<Body>) -> String {
    let bytes = axum::body::to_bytes(resp.into_body(), 65536)
        .await
        .expect("read response body");
    String::from_utf8_lossy(&bytes).to_string()
}

pub async fn fetch_body(app: &axum::Router, uri: &str) -> String {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert!(response.status().is_success(), "Expected success for {uri}");
    response_body(response).await
}

/// The status endpoint answers `idle`, a phase endpoint string, or an
/// `Error: {message}` span.
pub async fn fetch_generating_status(app: &axum::Router) -> String {
    fetch_body(app, "/status/generating").await
}

pub async fn post_form(
    app: &axum::Router,
    uri: &str,
    body: &str,
) -> axum::response::Response<Body> {
    let req = Request::builder()
        .uri(uri)
        .method(Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from(body.to_string()))
        .unwrap();
    app.clone().oneshot(req).await.unwrap()
}

pub async fn post_action(app: &axum::Router, command: &str) -> axum::response::Response<Body> {
    post_form(
        app,
        "/action",
        &format!("command={}", command.replace(' ', "+")),
    )
    .await
}

pub async fn post_action_check(
    app: &axum::Router,
    command: &str,
) -> axum::response::Response<Body> {
    post_form(
        app,
        "/action/check",
        &format!("command={}", command.replace(' ', "+")),
    )
    .await
}

pub async fn post_empty(app: &axum::Router, uri: &str) -> axum::response::Response<Body> {
    let req = Request::builder()
        .uri(uri)
        .method(Method::POST)
        .body(Body::empty())
        .unwrap();
    app.clone().oneshot(req).await.unwrap()
}

pub async fn post_form_with_hx(
    app: &axum::Router,
    uri: &str,
    body: &str,
) -> axum::response::Response<Body> {
    let req = Request::builder()
        .uri(uri)
        .method(Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .header("HX-Request", "true")
        .body(Body::from(body.to_string()))
        .unwrap();
    app.clone().oneshot(req).await.unwrap()
}

/// Waits for the Game's slot to free. It observes nothing but the wait itself, so
/// a test that also reads the turn's outcome calls `wait_settled`.
pub async fn wait_idle(state: &AppState, timeout_ms: u64) -> bool {
    let game_id = active_game_of(state);
    wait_for_condition_async(
        std::time::Duration::from_millis(timeout_ms),
        std::time::Duration::from_millis(15),
        || async {
            // The job entry is the one busy truth: no Job holds the Game.
            !state.game_view_query.is_busy(game_id)
        },
    )
    .await
}

/// Waits for the Job to settle, then checks the turn's own outcome. Every tier-1
/// test whose turn runs an analysis ends here.
pub async fn wait_settled(state: &AppState, timeout_ms: u64) -> bool {
    let idle = wait_idle(state, timeout_ms).await;
    if idle {
        assert_no_uncertain_npc_notice(state);
        assert_no_unscripted_model_call(state);
    }
    idle
}

pub fn assert_no_uncertain_npc_notice(state: &AppState) {
    assert!(
        !story_log_holds_uncertain_npc_notice(state),
        "the turn fell back to the uncertain NPC notice: its analysis answer was not a room report"
    );
}

/// A model call the test did not script is recorded as a failed attempt whose
/// error names the stray call. This spots one in a test that cannot reach the
/// fake; the fake's own drop fails the test that still holds it.
pub fn assert_no_unscripted_model_call(state: &AppState) {
    let strays: Vec<String> = state
        .game_view_query
        .list_latest_llm_messages(50)
        .expect("list_latest_llm_messages should succeed")
        .into_iter()
        .filter_map(|message| message.error_message)
        .filter(|error| error.contains(STRAY_CALL_MARKER))
        .collect();
    assert!(
        strays.is_empty(),
        "a model call reached the fake without a scripted answer: {strays:?}"
    );
}

pub fn story_log_holds_uncertain_npc_notice(state: &AppState) -> bool {
    state
        .message_service
        .load_messages(active_game_of(state))
        .map(|messages| {
            messages
                .iter()
                .any(|message| message.text().contains(UNCERTAIN_NPC_NOTICE))
        })
        .unwrap_or(false)
}

/// A stop and a reset end a Game's slot at once, while the Job's thread lives
/// on until its refusals return. A test that asserts what a Job's late answer
/// wrote waits here before it reads, so it observes the Job's end and not a
/// moment before it.
pub async fn wait_for_jobs(state: &AppState, timeout_ms: u64) -> bool {
    state.generation_gate.wait_for_jobs(timeout_ms).await
}
