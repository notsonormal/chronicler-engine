//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Shared template rendering helpers: the Askama renderer for `<option>`
//! lists, and the one-line preview of a long text.

use askama::Template;

use crate::adapters::driving::http::utils::fragment::render_or_error_fragment;
use crate::adapters::driving::http::view_models::{SafeHtml, SelectOptionView};

/// Renders an `<option>` list from `SelectOptionView`s, so the selects that
/// use it share one escaping and markup path. The game templates build their
/// own option lists.
#[derive(Template)]
#[template(
    source = r#"{% for option in options %}<option value="{{ option.value }}"{% if option.selected %} selected{% endif %}>{{ option.label }}</option>
{% endfor %}"#,
    ext = "html"
)]
pub struct SelectOptionsTemplate {
    pub options: Vec<SelectOptionView>,
}

pub(crate) fn select_options_html(options: Vec<SelectOptionView>) -> SafeHtml {
    SafeHtml::new(render_or_error_fragment(&SelectOptionsTemplate { options }))
}

/// Sentence punctuation directly before the ellipsis would read as the end of the
/// field rather than a cut, so it goes first — with any space it sat behind.
pub(crate) fn truncated_preview(text: &str) -> String {
    const MAX_PREVIEW_CHARS: usize = 120;
    const TRAILING_PUNCTUATION: [char; 6] = ['.', ',', ';', ':', '!', '?'];
    let text = text.replace('\n', " ");
    let text = text.trim();
    if text.chars().count() <= MAX_PREVIEW_CHARS {
        return text.to_string();
    }
    let head: String = text.chars().take(MAX_PREVIEW_CHARS).collect();
    let words = match head.rsplit_once(' ') {
        Some((words, _)) => words,
        None => head.as_str(),
    };
    let words =
        words.trim_end_matches(|c: char| c.is_whitespace() || TRAILING_PUNCTUATION.contains(&c));
    format!("{words}…")
}
