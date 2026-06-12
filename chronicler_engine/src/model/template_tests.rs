//! Tests for template placeholder substitution

use crate::model::template::{render_template, TemplateVars};

#[test]
fn render_template_replaces_user() {
    let text = "Hello {{user}}";
    let vars = TemplateVars::new("Julian");
    let result = render_template(text, &vars);
    assert_eq!(result, "Hello Julian");
}

#[test]
fn render_template_unknown_placeholder_left_as_is() {
    let text = "Hello {{char}}";
    let vars = TemplateVars::new("Julian");
    let result = render_template(text, &vars);
    assert_eq!(result, "Hello {{char}}");
}

#[test]
fn render_template_multiple_same_key() {
    let text = "{{user}} meets {{user}}";
    let vars = TemplateVars::new("Julian");
    let result = render_template(text, &vars);
    assert_eq!(result, "Julian meets Julian");
}
