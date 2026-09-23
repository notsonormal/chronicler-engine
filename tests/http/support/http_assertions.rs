//! Rendered-HTML assertions shared by the http test binary.

/// Assert the rendered fragment marks `value` as the selected option.
///
/// The selected option is the only server-observable proof that a stored
/// posture (or mode) reached the rendered form — an unselected select renders
/// the same shell whatever the stored value is.
pub fn assert_option_selected(html: &str, value: &str, msg: &str) {
    assert!(
        html.contains(&format!(r#"<option value="{value}" selected"#)),
        "{msg}: {html}"
    );
}
