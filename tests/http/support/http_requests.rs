//! HTTP request plumbing for the http test binary: POST builders, body readers, generation-idle polling.

use chronicler_engine::test_support::active_game_of;
use axum::body::Body;
use axum::http::{self, Method, Request};
use tower::util::ServiceExt;

use chronicler_engine::adapters::driving::http::AppState;

use crate::test_utils::wait_for_condition_async;

/// The body reader the support helpers use — one cap (65536) covers every
/// fragment and page this suite reads.
pub async fn response_body(resp: axum::response::Response<Body>) -> String {
    let bytes = axum::body::to_bytes(resp.into_body(), 65536)
        .await
        .expect("read response body");
    String::from_utf8_lossy(&bytes).to_string()
}

/// Panics if the request fails or returns non-success status.
pub async fn fetch_body(app: &axum::Router, uri: &str) -> String {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert!(response.status().is_success(), "Expected success for {uri}");
    response_body(response).await
}

/// The generation-status read seam: `GET /status/generating` returns `idle`,
/// the phase's endpoint string, or an `Error: {message}` span.
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

/// Carries `HX-Request`, the way htmx sends every fragment swap.
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

/// Waits for the generation slot too: the pipeline persists `Idle` before its guard
/// releases the slot, so a status-only check can return while `try_claim` still
/// rejects the next action as `ConcurrentGeneration`.
pub async fn wait_idle(state: &AppState, timeout_ms: u64) -> bool {
    let game_id = active_game_of(state);
    wait_for_condition_async(
        std::time::Duration::from_millis(timeout_ms),
        std::time::Duration::from_millis(15),
        || async {
            !state
                .message_service
                .load_or_fresh(game_id)
                .narrative
                .input_buffer
                .status
                .is_generating()
                && !state.generation_gate.is_busy(game_id)
        },
    )
    .await
}
