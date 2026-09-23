//! HTTP request plumbing for the http test binary: POST builders, body readers, generation-idle polling.

use axum::body::Body;
use axum::http::{self, Method, Request};
use tower::util::ServiceExt;

use chronicler_engine::adapters::driving::http::AppState;

use crate::test_utils::wait_for_condition_async;

/// Consume a response into its body as a String. The single body reader for
/// the binary — one cap (65536) covers every fragment and page this suite
/// reads.
pub async fn response_body(resp: axum::response::Response<Body>) -> String {
    let bytes = axum::body::to_bytes(resp.into_body(), 65536)
        .await
        .expect("read response body");
    String::from_utf8_lossy(&bytes).to_string()
}

/// GET the given URI and return the response body as a String.
/// Panics if the request fails or returns non-success status.
pub async fn fetch_body(app: &axum::Router, uri: &str) -> String {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert!(response.status().is_success(), "Expected success for {uri}");
    response_body(response).await
}

/// POST a url-encoded body to an arbitrary URI.
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

/// POST a url-encoded `command=...` body to `/action`.
pub async fn post_action(app: &axum::Router, command: &str) -> axum::response::Response<Body> {
    post_form(
        app,
        "/action",
        &format!("command={}", command.replace(' ', "+")),
    )
    .await
}

/// POST a url-encoded `command=...` body to `/action/check`.
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

/// POST to a no-body endpoint (`/swipe/new`, `/history/delete`, `/reset`).
pub async fn post_empty(app: &axum::Router, uri: &str) -> axum::response::Response<Body> {
    let req = Request::builder()
        .uri(uri)
        .method(Method::POST)
        .body(Body::empty())
        .unwrap();
    app.clone().oneshot(req).await.unwrap()
}

/// POST a url-encoded body carrying the `HX-Request` header, the way htmx
/// sends every fragment swap. Use this for endpoints whose response is a
/// fragment rather than a full page.
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

/// Poll `AppState` until generation is idle (timeout `timeout_ms`, 15ms interval).
pub async fn wait_idle(state: &AppState, timeout_ms: u64) -> bool {
    wait_for_condition_async(
        std::time::Duration::from_millis(timeout_ms),
        std::time::Duration::from_millis(15),
        || async {
            !state
                .message_service
                .load_or_fresh()
                .narrative
                .input_buffer
                .status
                .is_generating()
        },
    )
    .await
}
