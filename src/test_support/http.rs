//! HTTP response helpers for unit tests that call handlers directly.

/// Read a handler's response body as text.
#[allow(clippy::expect_used)]
pub async fn body_text(response: axum::response::Response<axum::body::Body>) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("handler response body");
    String::from_utf8_lossy(&bytes).to_string()
}
