//! Tests for `response.rs` HTTP response helpers

use axum::body::to_bytes;
use axum::http::StatusCode;

use crate::adapters::driving::http::fragments::renderers::response::{
    bad_request, html_escape, internal_error, ok, ok_refresh, service_unavailable,
    service_unavailable_generating,
};
use crate::adapters::driving::http::fragments::renderers::fragment_renderers::render_error;

async fn body_string(resp: axum::response::Response) -> String {
    let bytes = to_bytes(resp.into_body(), 16384).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap_or_default()
}

#[tokio::test]
async fn ok_returns_200_with_body() {
    let resp = ok("hello");
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_string(resp).await, "hello");
}

#[tokio::test]
async fn ok_refresh_sets_hx_refresh_header_and_empty_body() {
    let resp = ok_refresh();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get("HX-Refresh")
            .map(|v| v.to_str().unwrap()),
        Some("true")
    );
    assert_eq!(body_string(resp).await, "");
}

#[tokio::test]
async fn bad_request_returns_400_with_body() {
    let resp = bad_request("invalid input");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_string(resp).await, "invalid input");
}

#[tokio::test]
async fn internal_error_returns_500_with_body() {
    let resp = internal_error("boom");
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body_string(resp).await, "boom");
}

#[tokio::test]
async fn service_unavailable_returns_503_with_body() {
    let resp = service_unavailable("try later");
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body_string(resp).await, "try later");
}

#[tokio::test]
async fn service_unavailable_generating_uses_error_div_wrapper() {
    let resp = service_unavailable_generating();
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = body_string(resp).await;
    assert!(
        body.contains("error-message"),
        "body should wrap in error div: {body}"
    );
    assert!(
        body.contains("Generation in progress"),
        "body should include wait text: {body}"
    );
}

#[test]
fn html_escape_table() {
    let cases = [
        ("<test>", "&lt;test&gt;"),
        ("foo & bar", "foo &amp; bar"),
        ("\"quoted\"", "&quot;quoted&quot;"),
        (
            "<foo & \"bar'\">",
            "&lt;foo &amp; &quot;bar&#x27;&quot;&gt;",
        ),
        ("", ""),
        ("line1\nline2", "line1\nline2"),
        ("`code`", "`code`"),
        ("日本語", "日本語"),
        ("&", "&amp;"),
        ("<", "&lt;"),
        (">", "&gt;"),
    ];
    for (input, expected) in cases {
        assert_eq!(html_escape(input), expected, "input: {input:?}");
    }
}

#[test]
fn html_escape_is_not_idempotent() {
    let escaped = html_escape("<&>");
    assert_eq!(escaped, "&lt;&amp;&gt;");
    assert_eq!(html_escape(&escaped), "&amp;lt;&amp;amp;&amp;gt;");
}

#[test]
fn render_error_wraps_in_error_div() {
    let html = render_error("disk failed");
    assert!(html.contains("error-message"));
    assert!(html.contains("Error:"));
    assert!(html.contains("disk failed"));
}

#[test]
fn render_error_escapes_html_in_message() {
    let html = render_error("<script>alert('xss')</script>");
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn render_error_preserves_long_messages() {
    let long_msg = "x".repeat(10_000);
    let html = render_error(&long_msg);
    assert!(html.len() > 10_000);
    assert!(html.contains(&long_msg[..100]));
}
