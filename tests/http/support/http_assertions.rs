//! Rendered-HTML assertions shared by the http test binary.

use chronicler_engine::adapters::driving::http::AppState;

/// How many model calls the Game recorded for one Agent, read through the
/// forensics seam. A test that pins "the Job made no Options call" reads this
/// rather than the fake's own call log.
pub fn llm_call_count(state: &AppState, agent_name: &str) -> usize {
    state
        .game_view_query
        .list_latest_llm_messages(50)
        .expect("list_latest_llm_messages should succeed")
        .into_iter()
        .filter(|message| message.agent_name == agent_name)
        .count()
}

/// The selected option is the only server-observable proof that a stored
/// posture (or mode) reached the rendered form; an unselected select renders
/// the same shell whatever the stored value is.
pub fn assert_option_selected(html: &str, value: &str, msg: &str) {
    assert!(
        html.contains(&format!(r#"<option value="{value}" selected"#)),
        "{msg}: {html}"
    );
}

/// The story log renders one `id="entry-N"` row per Message, in history
/// order, so the last one is the Message the player sees at the log's bottom.
pub fn assert_story_log_ends_with_message(story_log: &str, message_id: u64, msg: &str) {
    let last_rendered = story_log
        .rsplit(r#"id="entry-"#)
        .next()
        .and_then(|rest| rest.split('"').next())
        .expect("the story log fragment renders entries");
    assert_eq!(last_rendered, message_id.to_string(), "{msg}: {story_log}");
}

pub fn status_error_message(fragment: &str) -> &str {
    let marker = r#"<pre class="error-detail-raw">"#;
    let start = fragment
        .find(marker)
        .map(|i| i + marker.len())
        .unwrap_or_else(|| panic!("expected a status error disclosure, got: {fragment}"));
    let rest = &fragment[start..];
    rest.split("</pre>").next().unwrap_or(rest)
}

fn attribute_value(opening_tag: &str, name: &str) -> Option<String> {
    let marker = format!(" {name}=\"");
    let start = opening_tag.find(&marker)? + marker.len();
    let length = opening_tag[start..].find('"')?;
    Some(opening_tag[start..start + length].to_string())
}

fn opening_tag_end(markup: &str) -> usize {
    let mut quote: Option<char> = None;
    for (offset, character) in markup.char_indices() {
        if Some(character) == quote {
            quote = None;
        } else if quote.is_none() && (character == '"' || character == '\'') {
            quote = Some(character);
        } else if quote.is_none() && character == '>' {
            return offset;
        }
    }
    panic!("an opening tag must close: {markup}");
}

/// The rendered text of `markup`, read by dropping everything between `<` and
/// `>`: a text node that contains a literal angle bracket is therefore read
/// wrong. Rendered fragments cannot carry one — the templates escape `<` and
/// `>` — so the fixtures this reads never trip the assumption.
fn visible_text(markup: &str) -> String {
    let mut text = String::new();
    let mut inside_tag = false;
    for character in markup.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => text.push(character),
            _ => {}
        }
    }
    text
}

/// Only a button with no visible text depends on its `aria-label` for a name.
pub fn icon_buttons_without_matching_name(html: &str) -> Vec<String> {
    const CLOSE: &str = "</button>";
    let mut offenders = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<button") {
        let button = &rest[start..];
        let opening_end = opening_tag_end(button);
        let close = button.find(CLOSE).expect("a button must close");
        let opening_tag = &button[..=opening_end];
        if visible_text(&button[opening_end + 1..close])
            .trim()
            .is_empty()
        {
            let label = attribute_value(opening_tag, "aria-label");
            let tooltip = attribute_value(opening_tag, "title");
            let named = label.as_deref().is_some_and(|name| !name.trim().is_empty());
            let says_the_same = tooltip.is_none() || tooltip == label;
            if !named || !says_the_same {
                offenders.push(opening_tag.to_string());
            }
        }
        rest = &button[close + CLOSE.len()..];
    }
    offenders
}

pub fn svgs_not_hidden_from_assistive_technology(html: &str) -> Vec<String> {
    html.match_indices("<svg")
        .map(|(start, _)| {
            let svg = &html[start..];
            svg[..=opening_tag_end(svg)].to_string()
        })
        .filter(|opening_tag| {
            attribute_value(opening_tag, "aria-hidden").as_deref() != Some("true")
        })
        .collect()
}

fn icon_ids(markup: &str, marker: &str) -> Vec<String> {
    const PREFIX: &str = "i-";
    assert!(
        marker.ends_with(PREFIX),
        "the marker must end with `{PREFIX}`"
    );
    markup
        .match_indices(marker)
        .map(|(start, found)| {
            let name = &markup[start + found.len()..];
            name[..name.find('"').expect("an icon id must close")].to_string()
        })
        .collect()
}

pub fn referenced_icons(html: &str) -> Vec<String> {
    icon_ids(html, "href=\"#i-")
}

pub fn defined_icons(shell: &str) -> Vec<String> {
    icon_ids(shell, "<symbol id=\"i-")
}
