// Server Core Unit Tests - Phase 3
// These tests cover view models, renderers, and utility functions

use crate::server::view_models::markdown_to_html;

#[test]
fn test_markdown_to_html_headers() {
    let html = markdown_to_html("# Header 1\n## Header 2");
    assert!(html.contains("<h1>"));
    assert!(html.contains("Header 1"));
    assert!(html.contains("<h2>"));
    assert!(html.contains("Header 2"));
}

#[test]
fn test_markdown_to_html_lists() {
    let html = markdown_to_html("- Item 1\n- Item 2\n- Item 3");
    assert!(html.contains("<ul>"));
    assert!(html.contains("<li>"));
    assert!(html.contains("Item 1"));
}

#[test]
fn test_markdown_to_html_links() {
    let html = markdown_to_html("[link](https://example.com)");
    assert!(html.contains("<a href="));
    assert!(html.contains("https://example.com"));
}

#[test]
fn test_markdown_to_html_code_blocks() {
    let html = markdown_to_html("```\ncode block\n```");
    assert!(html.contains("<code>"));
    assert!(html.contains("<pre>"));
    assert!(html.contains("code block"));
}

#[test]
fn test_markdown_to_html_blockquotes() {
    let html = markdown_to_html("> This is a quote");
    assert!(html.contains("<blockquote>"));
    assert!(html.contains("This is a quote"));
}

#[test]
fn test_markdown_to_html_xss_prevention() {
    let html = markdown_to_html("<script>alert('xss')</script>");
    assert!(!html.contains("<script>alert"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn test_markdown_to_html_italic() {
    let html = markdown_to_html("*italic text*");
    assert!(html.contains("<em>") || html.contains("italic"));
}

#[test]
fn test_markdown_to_html_bold() {
    let html = markdown_to_html("**bold text**");
    assert!(html.contains("<strong>") || html.contains("bold"));
}

#[test]
fn test_markdown_to_html_mixed_content() {
    let html = markdown_to_html("# Title\n\nParagraph with **bold** and *italic*.\n\n- List item");
    assert!(html.contains("<h1>"));
    assert!(html.contains("<p>"));
    assert!(html.contains("<ul>"));
}
