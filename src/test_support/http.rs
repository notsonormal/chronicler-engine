//! HTTP response helpers for unit tests that call handlers directly.

#[allow(clippy::expect_used)]
pub async fn body_text(response: axum::response::Response<axum::body::Body>) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("handler response body");
    String::from_utf8_lossy(&bytes).to_string()
}

/// An action that went through answers 200 with no body and no header that
/// steers the browser: the data reaches the screen through the change numbers.
#[allow(clippy::expect_used)]
pub async fn assert_answers_nothing(response: axum::response::Response<axum::body::Body>) {
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    for name in ["HX-Retarget", "HX-Reswap", "HX-Refresh"] {
        assert!(
            response.headers().get(name).is_none(),
            "a successful action must carry no {name} header"
        );
    }
    let body = body_text(response).await;
    assert!(
        body.is_empty(),
        "a successful action answers nothing: {body}"
    );
}
