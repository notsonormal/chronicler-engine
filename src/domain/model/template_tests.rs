//! Tests for template placeholder substitution

use crate::domain::model::prompt_preset::PromptPreset;
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
fn template_vars_new_defaults_to_third_person_past_tense() {
    let vars = TemplateVars::new("Julian");
    assert_eq!(vars.narrative_perspective, "third");
    assert_eq!(vars.narrative_tense, "past");
}

#[test]
fn template_vars_from_persona_defaults_to_third_person_past_tense() {
    let persona = crate::test_support::TestPersona::named("Julian");
    let vars = TemplateVars::from_persona(&persona);
    assert_eq!(vars.narrative_perspective, "third");
    assert_eq!(vars.narrative_tense, "past");
}

#[test]
fn render_template_replaces_narrative_perspective() {
    let mut vars = TemplateVars::new("Julian");
    vars.narrative_perspective = "second".to_string();
    let result = render_template("{{narrative_perspective}}-person view", &vars);
    assert_eq!(result, "second-person view");
}

#[test]
fn render_template_replaces_narrative_tense() {
    let mut vars = TemplateVars::new("Julian");
    vars.narrative_tense = "present".to_string();
    let result = render_template("{{narrative_tense}} tense", &vars);
    assert_eq!(result, "present tense");
}

#[test]
fn render_template_unknown_narrative_placeholder_left_as_is() {
    let vars = TemplateVars::new("Julian");
    let result = render_template("{{narrative_voice}}", &vars);
    assert_eq!(result, "{{narrative_voice}}");
}

#[test]
fn template_vars_new_defaults_option_count_to_three() {
    let vars = TemplateVars::new("Julian");
    assert_eq!(vars.option_count, "3");
}

#[test]
fn render_template_replaces_option_count() {
    let vars = TemplateVars::new("Julian");
    let result = render_template(
        "Generate *exactly* {{option_count}} actions for {{user}}.",
        &vars,
    );
    assert_eq!(result, "Generate *exactly* 3 actions for Julian.");
}

#[test]
fn render_template_option_count_survives_preset_seed_render() {
    let content = include_str!("../../../data/prompt_presets/options/default.json");
    let preset: PromptPreset = serde_json::from_str(content).expect("options seed should parse");
    let instructions = preset.instructions.expect("options seed has instructions");

    let rendered = render_template(&instructions, &TemplateVars::new("Julian"));
    assert!(
        !rendered.contains("{{option_count}}"),
        "{{option_count}} must substitute in the options seed"
    );
    assert!(rendered.contains("3 brief distinct single-sentence suggestions"));
}

fn make_vars(perspective: &str, tense: &str) -> TemplateVars {
    let mut vars = TemplateVars::new("Julian");
    vars.narrative_perspective = perspective.to_string();
    vars.narrative_tense = tense.to_string();
    vars
}

fn load_system_preset_seed_json() -> PromptPreset {
    let content = include_str!("../../../data/prompt_presets/system/default.json");
    serde_json::from_str(content).expect("system preset JSON should parse")
}

fn load_impersonate_preset_seed_json() -> PromptPreset {
    let content = include_str!("../../../data/prompt_presets/impersonate/default.json");
    serde_json::from_str(content).expect("impersonate preset JSON should parse")
}

#[test]
fn system_preset_writing_style_renders_coherently_for_all_voices() {
    let preset = load_system_preset_seed_json();
    let writing_style = preset
        .writing_style
        .expect("system preset has writing_style");

    let third_past = render_template(&writing_style, &make_vars("third", "past"));
    assert!(third_past.contains("third-person limited perspective"));
    assert!(third_past.contains("past tense narrative prose"));

    let second_past = render_template(&writing_style, &make_vars("second", "past"));
    assert!(second_past.contains("second-person limited perspective"));
    assert!(second_past.contains("past tense narrative prose"));

    let third_present = render_template(&writing_style, &make_vars("third", "present"));
    assert!(third_present.contains("third-person limited perspective"));
    assert!(third_present.contains("present tense narrative prose"));

    let second_present = render_template(&writing_style, &make_vars("second", "present"));
    assert!(second_present.contains("second-person limited perspective"));
    assert!(second_present.contains("present tense narrative prose"));
}

#[test]
fn impersonate_preset_writing_style_renders_coherently_for_all_voices() {
    let preset = load_impersonate_preset_seed_json();
    let writing_style = preset
        .writing_style
        .expect("impersonate preset has writing_style");

    let third_past = render_template(&writing_style, &make_vars("third", "past"));
    assert!(third_past.contains("third-person perspective as Julian"));
    assert!(third_past.contains("past tense"));

    let second_past = render_template(&writing_style, &make_vars("second", "past"));
    assert!(second_past.contains("second-person perspective as Julian"));
    assert!(second_past.contains("past tense"));

    let third_present = render_template(&writing_style, &make_vars("third", "present"));
    assert!(third_present.contains("third-person perspective as Julian"));
    assert!(third_present.contains("present tense"));

    let second_present = render_template(&writing_style, &make_vars("second", "present"));
    assert!(second_present.contains("second-person perspective as Julian"));
    assert!(second_present.contains("present tense"));
}

#[test]
fn impersonate_preset_instructions_render_coherent_perspective_and_tense() {
    let preset = load_impersonate_preset_seed_json();
    let instructions = preset
        .instructions
        .expect("impersonate preset has instructions");

    let third_past = render_template(&instructions, &make_vars("third", "past"));
    assert!(third_past.contains("third person for speech and internal thought"));
    assert!(third_past.contains("in past tense"));

    let second_present = render_template(&instructions, &make_vars("second", "present"));
    assert!(second_present.contains("second person for speech and internal thought"));
    assert!(second_present.contains("in present tense"));
}
