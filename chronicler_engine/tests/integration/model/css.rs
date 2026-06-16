use axum::{body::Body, http::Request};
use tower::util::ServiceExt;

use chronicler_engine::TestAppBuilder;

#[tokio::test]
async fn test_css_valid() {
    let app = TestAppBuilder::default_app();

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

    assert!(
        css_content.len() > 1000,
        "CSS should be non-trivial (>1000 chars), got {}",
        css_content.len()
    );

    // :root must define a background variable — guards against accidental removal
    assert!(
        css_content.contains("--color-bg-") || css_content.contains("--bg"),
        "CSS :root should define a background variable (--color-bg- or --bg)"
    );
}

#[tokio::test]
async fn test_scrollbar_styled() {
    let app = TestAppBuilder::default_app();

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

#[tokio::test]
async fn test_css_design_tokens_cover_core_areas() {
    let app = TestAppBuilder::default_app();

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

    let core_prefixes = [
        "--color-bg-",
        "--color-text-",
        "--color-accent-",
        "--color-button-",
        "--color-log-",
        "--font-",
    ];

    let covered = core_prefixes
        .iter()
        .filter(|prefix| css_content.contains(*prefix))
        .count();

    assert!(
        covered >= 5,
        "CSS :root should define variables in at least 5 of 6 core areas \
        (bg, text, accent, button, log, typography), only {covered}/6 found"
    );
}

#[tokio::test]
async fn test_css_responsive_breakpoints() {
    let app = TestAppBuilder::default_app();

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

    let mut found_breakpoint = false;
    for line in css_content.lines() {
        let line = line.trim();
        if !line.starts_with("@media") {
            continue;
        }
        for keyword in ["max-width", "min-width"] {
            if let Some(start) = line.find(keyword) {
                let rest = &line[start + keyword.len()..];
                let rest = rest.trim_start_matches([' ', ':']);
                let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(width) = num_str.parse::<u32>() {
                    assert!(
                        (100..=2000).contains(&width),
                        "Responsive breakpoint {width}px is out of reasonable range (100-2000px)"
                    );
                    found_breakpoint = true;
                }
            }
        }
    }

    assert!(
        found_breakpoint,
        "CSS should contain at least one @media query with a width breakpoint"
    );
}
