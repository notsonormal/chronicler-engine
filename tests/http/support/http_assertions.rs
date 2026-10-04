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

/// Extract the message from a rendered error status fragment.
///
/// The generating-status endpoint renders an error turn as
/// `<span class="status error">Error: …</span>`; the prefix says the turn
/// failed but not why, so tests assert on the message between the markers.
pub fn status_error_message(fragment: &str) -> &str {
    fragment
        .strip_prefix(r#"<span class="status error">Error: "#)
        .and_then(|rest| rest.strip_suffix("</span>"))
        .unwrap_or_else(|| panic!("expected an Error status span, got: {fragment}"))
}
