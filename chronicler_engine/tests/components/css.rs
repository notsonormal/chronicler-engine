use axum::{body::Body, http::Request};
use tower::util::ServiceExt;

use chronicler_engine::create_app_for_testing;

use crate::create_test_state;

#[tokio::test]
async fn test_css_valid() {
    let state = create_test_state();
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/assets/styles.css")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 65536)
        .await
        .unwrap();
    let css_content = String::from_utf8_lossy(&body);

    assert!(
        css_content.contains(":root"),
        "CSS should contain :root for CSS variables"
    );
    assert!(
        css_content.contains("var(--"),
        "CSS should use CSS custom properties (var())"
    );
    assert!(
        css_content.contains("@media"),
        "CSS should contain @media queries for responsive breakpoints"
    );
}

#[tokio::test]
async fn test_scrollbar_styled() {
    let state = create_test_state();
    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri("/assets/styles.css")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 65536)
        .await
        .unwrap();
    let css_content = String::from_utf8_lossy(&body);

    assert!(
        css_content.contains("::-webkit-scrollbar"),
        "CSS should contain custom scrollbar styling"
    );
    assert!(
        css_content.contains("scrollbar-width"),
        "CSS should contain Firefox scrollbar-width"
    );
}
