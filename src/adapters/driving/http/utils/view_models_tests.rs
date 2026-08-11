//! Tests for `view_models.rs` markdown → HTML pipeline.

use pulldown_cmark::{CowStr, Event};

use crate::adapters::driving::http::utils::view_models::{
    markdown_to_html, render_quote_events, sanitize_markdown_event,
};

fn events_to_string(events: Vec<Event<'static>>) -> String {
    events
        .into_iter()
        .map(|e| match e {
            Event::Text(t) => t.to_string(),
            Event::Html(t) | Event::InlineHtml(t) => t.to_string(),
            _ => String::new(),
        })
        .collect()
}

#[test]
fn render_quote_events_emits_q_tags_for_curly_quotes() {
    let out = events_to_string(render_quote_events("He said \u{201C}hi\u{201D} loud"));
    assert_eq!(out, "He said <q>hi</q> loud");
}

#[test]
fn render_quote_events_preserves_text_without_curly_quotes() {
    let events = render_quote_events("plain text");
    assert_eq!(events.len(), 1);
    match &events[0] {
        Event::Text(t) => assert_eq!(t.as_ref(), "plain text"),
        other => panic!("expected Text, got {other:?}"),
    }
}

#[test]
fn sanitize_markdown_event_converts_raw_html_to_text() {
    let events = sanitize_markdown_event(Event::Html(CowStr::Borrowed("<script>")));
    assert_eq!(events.len(), 1);
    match &events[0] {
        Event::Text(t) => assert_eq!(t.as_ref(), "<script>"),
        other => panic!("expected Text, got {other:?}"),
    }
}

#[test]
fn markdown_to_html_escapes_raw_html_but_renders_curly_quote_tags() {
    let out = markdown_to_html("<b>x</b> \u{201C}quote\u{201D}");
    assert!(out.contains("&lt;b&gt;"), "raw html escaped: {out}");
    assert!(out.contains("<q>quote</q>"), "curly quotes rendered: {out}");
}
