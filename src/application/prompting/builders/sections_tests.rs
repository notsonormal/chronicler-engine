//! Tests for `builders/sections.rs` prompt-section renderers.

use crate::application::prompting::builders::sections::{
    render_known_npc_entry, render_preset_xml_parts, render_present_relationships, wrap_xml,
    Section,
};
use crate::domain::model::character::{CharacterSheet, NpcCard, Relationship};
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::template::TemplateVars;
use std::collections::HashSet;

#[test]
fn wrap_xml_indents_non_empty_lines_keeps_empty() {
    let out = wrap_xml("line1\n\nline2", "tag");
    assert_eq!(out, "<tag>\n    line1\n\n    line2\n</tag>");
}

#[test]
fn wrap_xml_preserves_single_empty_content_line() {
    let out = wrap_xml("", "t");
    assert_eq!(out, "<t>\n\n</t>");
}

fn make_npc(
    id: &str,
    name: &str,
    summary: Option<&str>,
    description: &str,
    relationships: Vec<Relationship>,
) -> NpcCard {
    NpcCard {
        id: id.to_string(),
        sheet: CharacterSheet {
            name: name.to_string(),
            description: description.to_string(),
            personality: String::new(),
            scenario: String::new(),
            example_dialogue: String::new(),
            summary: summary.map(|s| s.to_string()),
            profile_image: None,
            headshot_image: None,
        },
        inventory: vec![],
        triggers: vec![],
        relationships,
    }
}

#[test]
fn render_preset_xml_parts_omits_all_fields_when_preset_has_none() {
    let preset = PromptPreset::default();
    let parts = render_preset_xml_parts(&preset, &[], None, None);
    assert!(parts.is_empty(), "no fields set → expected no parts");
}

#[test]
fn render_preset_xml_parts_emits_only_present_fields() {
    let preset = PromptPreset {
        role: Some("r".to_string()),
        ..PromptPreset::default()
    };
    let parts = render_preset_xml_parts(&preset, &["rule one".to_string()], None, None);
    let sections: Vec<_> = parts.iter().map(|(s, _)| *s).collect();
    assert!(sections.contains(&Section::Role));
    assert!(sections.contains(&Section::GlobalRules));
    assert!(!sections.contains(&Section::Instructions));
    assert!(!sections.contains(&Section::WritingStyle));
    assert!(!sections.contains(&Section::OutputFormat));
}

#[test]
fn render_preset_xml_parts_appends_response_length_to_output_format() {
    let preset = PromptPreset {
        output_format: Some("fmt".to_string()),
        ..PromptPreset::default()
    };
    let parts = render_preset_xml_parts(&preset, &[], Some("3 paragraphs"), None);
    let (_, text) = parts
        .iter()
        .find(|(s, _)| *s == Section::OutputFormat)
        .expect("OutputFormat section should be present");
    assert!(text.contains("fmt"), "text: {text}");
    assert!(text.contains("Response Length:"), "text: {text}");
    assert!(text.contains("3 paragraphs"), "text: {text}");
}

#[test]
fn render_known_npc_entry_uses_summary_when_present() {
    let npc = make_npc("n1", "Alice", Some("Short summary."), "Long desc.", vec![]);
    let mut in_area: HashSet<&str> = HashSet::new();
    in_area.insert("n1");
    let vars = TemplateVars::new("player");
    let out = render_known_npc_entry(&npc, &in_area, &vars);
    assert!(out.contains("Alice (in room)"), "out: {out}");
    assert!(out.contains("Short summary."), "out: {out}");
    assert!(!out.contains("Long desc."), "out: {out}");
}

#[test]
fn render_known_npc_entry_falls_back_to_description_first_three_lines() {
    let desc = "line one.\nline two.\nline three.\nline four.";
    let npc = make_npc("n2", "Bob", None, desc, vec![]);
    let in_area: HashSet<&str> = HashSet::new(); // elsewhere
    let vars = TemplateVars::new("player");
    let out = render_known_npc_entry(&npc, &in_area, &vars);
    assert!(out.contains("Bob (elsewhere)"), "out: {out}");
    assert!(out.contains("line one."), "out: {out}");
    assert!(out.contains("line two."), "out: {out}");
    assert!(out.contains("line three."), "out: {out}");
    assert!(!out.contains("line four."), "only first 3 lines: {out}");
}

#[test]
fn render_present_relationships_returns_none_when_no_present_relations() {
    let partner = make_npc("p1", "Partner", None, "d", vec![]);
    let npc = make_npc(
        "n3",
        "Self",
        None,
        "d",
        vec![Relationship {
            with: "p1".to_string(),
            static_text: "allies".to_string(),
            dynamic: String::new(),
        }],
    );
    let in_area: HashSet<&str> = HashSet::new();
    let vars = TemplateVars::new("player");
    assert!(render_present_relationships(&npc, &[partner], &in_area, &vars).is_none());
}

#[test]
fn render_present_relationships_uses_partner_name_when_partner_in_area() {
    let partner = make_npc("p1", "Partner", None, "d", vec![]);
    let npc = make_npc(
        "n4",
        "Self",
        None,
        "d",
        vec![Relationship {
            with: "p1".to_string(),
            static_text: "allies".to_string(),
            dynamic: String::new(),
        }],
    );
    let mut in_area: HashSet<&str> = HashSet::new();
    in_area.insert("p1");
    let vars = TemplateVars::new("player");
    let block = render_present_relationships(&npc, &[partner], &in_area, &vars)
        .expect("present relations should produce block");
    assert!(block.contains("Relationships:"), "block: {block}");
    assert!(block.contains("Partner"), "block: {block}");
    assert!(block.contains("allies"), "block: {block}");
}
