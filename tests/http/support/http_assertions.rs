//! Rendered-HTML assertions shared by the http test binary.

/// The selected option is the only server-observable proof that a stored
/// posture (or mode) reached the rendered form; an unselected select renders
/// the same shell whatever the stored value is.
pub fn assert_option_selected(html: &str, value: &str, msg: &str) {
    assert!(
        html.contains(&format!(r#"<option value="{value}" selected"#)),
        "{msg}: {html}"
    );
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

/// Skips any `>` that sits inside a quoted attribute value.
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

/// Each returned id keeps the `i-` prefix.
fn icon_ids(markup: &str, marker: &str) -> Vec<String> {
    const PREFIX: &str = "i-";
    assert!(
        marker.ends_with(PREFIX),
        "the marker must end with `{PREFIX}`"
    );
    markup
        .match_indices(marker)
        .map(|(start, found)| {
            let name = &markup[start + found.len() - PREFIX.len()..];
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
