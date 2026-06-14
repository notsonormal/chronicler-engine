//! Shared test helpers for HTTP tests

use axum::body::Body;
use axum::http::Request;
use tower::util::ServiceExt;

/// Fetch the response body as a String from the given URI.
/// Panics if the request fails or returns non-success status.
pub async fn fetch_body(app: axum::Router, uri: &str) -> String {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert!(response.status().is_success(), "Expected success for {uri}");
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    String::from_utf8_lossy(&body).to_string()
}
