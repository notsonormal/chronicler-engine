//! Tests for template placeholder substitution

use crate::domain::model::template::TemplateVars;
use crate::domain::model::utils::template::render_template;

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

#[test]
fn render_template_replaces_persona_macros_from_persona_card() {
    let persona = crate::test_support::TestPersona::named("Julian");
    let vars = TemplateVars::from_persona(&persona);
    let text =
        "{{user}}: {{persona_description}} | {{persona_personality}} | {{persona_background}}";
    let result = render_template(text, &vars);
    assert_eq!(
        result,
        "Julian: The protagonist named Julian. | Determined | Test scenario."
    );
}

#[test]
fn template_vars_new_has_empty_persona_macros() {
    let vars = TemplateVars::new("Julian");
    assert_eq!(vars.user, "Julian");
    assert_eq!(vars.persona_description, "");
    assert_eq!(vars.persona_personality, "");
    assert_eq!(vars.persona_background, "");
    let result = render_template("{{persona_description}}", &vars);
    assert_eq!(result, "");
}

#[test]
fn template_vars_from_persona_populates_all_fields() {
    let persona = crate::test_support::TestPersona::named("Hero");
    let vars = TemplateVars::from_persona(&persona);
    assert_eq!(vars.user, "Hero");
    assert_eq!(vars.persona_description, "The protagonist named Hero.");
    assert_eq!(vars.persona_personality, "Determined");
    assert_eq!(vars.persona_background, "Test scenario.");
}
