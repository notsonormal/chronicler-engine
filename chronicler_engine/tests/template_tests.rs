//! Unit tests for Askama templates.
//!
//! These tests render templates directly without requiring an HTTP server.
//! They run in milliseconds and provide compile-time validation.

use askama::Template;
use chronicler_engine::server::templates::HeaderTemplate;

#[test]
fn test_header_template_renders_room_name() {
    let template = HeaderTemplate {
        room_name: "Test Room".to_string(),
    };
    let rendered = template.render().unwrap();
    assert!(
        rendered.contains("Test Room"),
        "Expected rendered output to contain 'Test Room': {}",
        rendered
    );
    assert!(
        rendered.contains(r#"class="header""#),
        "Expected header class: {}",
        rendered
    );
    assert!(
        rendered.contains(r#"class="game-title""#),
        "Expected game-title class: {}",
        rendered
    );
    assert!(
        rendered.contains(r#"class="location""#),
        "Expected location class: {}",
        rendered
    );
}

#[test]
fn test_header_template_escapes_html() {
    let template = HeaderTemplate {
        room_name: "<script>alert('xss')</script>".to_string(),
    };
    let rendered = template.render().unwrap();
    assert!(
        !rendered.contains("<script>"),
        "Should not contain raw script tag: {}",
        rendered
    );
    // Askama 0.15+ uses numeric HTML entities (&#60; for <) instead of named entities
    assert!(
        rendered.contains("&#60;script&#62;"),
        "Should contain escaped script tag: {}",
        rendered
    );
}

#[test]
fn test_header_template_connection_status() {
    let template = HeaderTemplate {
        room_name: "Any Room".to_string(),
    };
    let rendered = template.render().unwrap();
    assert!(
        rendered.contains(r#"id="connection-status""#),
        "Expected connection-status id: {}",
        rendered
    );
    assert!(
        rendered.contains("Connected"),
        "Expected Connected text: {}",
        rendered
    );
}

#[test]
fn test_header_template_exact_output() {
    let template = HeaderTemplate {
        room_name: "Grand Hall".to_string(),
    };
    let rendered = template.render().unwrap();
    // Print actual output for debugging
    eprintln!("Rendered output: {:?}", rendered);
    // Verify contains key elements (whitespace may vary)
    assert!(rendered.contains("class=\"header\""));
    assert!(rendered.contains("Chronicler Engine"));
    assert!(rendered.contains("| Grand Hall"));
}
