//! Tests for `utils/xml.rs` XML formatting helpers.

use crate::domain::model::utils::xml::wrap_xml;

#[test]
fn test_wrap_xml_indents_non_empty_lines_keeps_empty() {
    let out = wrap_xml("line1\n\nline2", "tag");
    assert_eq!(out, "<tag>\n    line1\n\n    line2\n</tag>");
}

#[test]
fn test_wrap_xml_preserves_single_empty_content_line() {
    let out = wrap_xml("", "t");
    assert_eq!(out, "<t>\n\n</t>");
}
