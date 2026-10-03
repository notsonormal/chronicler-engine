//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Shared template rendering helpers: the Askama renderer for `<option>`
//! lists.

use askama::Template;

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

/// Render `options` as `<option>` markup. The template only reads its fields,
/// so rendering cannot fail; a failure degrades to no options rather than a
/// page error.
pub(crate) fn select_options_html(options: Vec<SelectOptionView>) -> SafeHtml {
    SafeHtml::new(
        SelectOptionsTemplate { options }
            .render()
            .unwrap_or_default(),
    )
}
