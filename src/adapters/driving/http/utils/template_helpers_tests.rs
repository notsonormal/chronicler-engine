//! Tests for the shared `<option>` list renderer.

use crate::adapters::driving::http::utils::template_helpers::select_options_html;
use crate::adapters::driving::http::view_models::SelectOptionView;
use crate::domain::model::settings::NarratorMode;

#[test]
fn narrator_mode_options_mark_the_selected_mode() {
    let html = select_options_html(SelectOptionView::narrator_modes(
        NarratorMode::InteractiveFiction,
    ))
    .to_string();

    assert!(
        html.contains(
            r#"<option value="interactive_fiction" selected>Interactive Fiction</option>"#
        ),
        "{html}"
    );
    assert!(
        html.contains(r#"<option value="novel">Novel</option>"#),
        "{html}"
    );
}

#[test]
fn narrator_mode_options_select_novel_by_default() {
    let html =
        select_options_html(SelectOptionView::narrator_modes(NarratorMode::Novel)).to_string();

    assert!(
        html.contains(r#"<option value="novel" selected>Novel</option>"#),
        "{html}"
    );
    assert!(
        html.contains(r#"<option value="interactive_fiction">Interactive Fiction</option>"#),
        "{html}"
    );
}

#[test]
fn option_values_and_labels_are_html_escaped() {
    let options = vec![SelectOptionView {
        value: r#"a"b"#.into(),
        label: r#"x"y & z"#.into(),
        selected: false,
    }];

    let html = select_options_html(options).to_string();
    // Askama escapes to numeric entities: `"` => `&#34;`, `&` => `&#38;`.
    assert!(html.contains("&#34;"), "quotes must be escaped: {html}");
    assert!(html.contains("&#38;"), "ampersands must be escaped: {html}");
    assert!(
        !html.contains(r#"value="a"b""#),
        "a raw quote must not reach an attribute: {html}"
    );
}
