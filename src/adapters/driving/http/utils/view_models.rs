//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! View-model helpers shared between template code and tests.

use pulldown_cmark::{CowStr, Event, Options, Parser, html};

pub(crate) fn markdown_to_html(text: &str) -> String {
    let parser =
        Parser::new_ext(text, Options::ENABLE_SMART_PUNCTUATION).flat_map(sanitize_markdown_event);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

pub(crate) fn sanitize_markdown_event(event: Event<'_>) -> Vec<Event<'static>> {
    match event {
        Event::Text(text) => render_quote_events(&text),
        Event::Html(raw_html) | Event::InlineHtml(raw_html) => {
            vec![Event::Text(raw_html.to_string().into())]
        }
        safe_event => vec![safe_event.into_static()],
    }
}

pub(crate) fn render_quote_events(text: &str) -> Vec<Event<'static>> {
    let mut events = Vec::new();
    let mut text_start = 0;

    for (quote_index, character) in text.char_indices() {
        let quote_tag = match character {
            '\u{201C}' => "<q>",
            '\u{201D}' => "</q>",
            _ => continue,
        };
        if text_start < quote_index {
            events.push(Event::Text(text[text_start..quote_index].to_owned().into()));
        }
        events.push(Event::InlineHtml(CowStr::from(quote_tag).into_static()));
        text_start = quote_index + character.len_utf8();
    }

    if text_start < text.len() {
        events.push(Event::Text(text[text_start..].to_owned().into()));
    }
    events
}
