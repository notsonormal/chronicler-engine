//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Multi-stage prompt builder.

use crate::domain::model::character::{NpcCard, Relationship};
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::template::TemplateVars;
use crate::domain::model::utils::template::render_template;

pub(crate) fn wrap_xml(content: &str, tag: &str) -> String {
    let indented = content
        .lines()
        .map(|line| {
            if line.is_empty() {
                line.to_string()
            } else {
                format!("    {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("<{tag}>\n{indented}\n</{tag}>")
}

pub(crate) fn sanitize_for_prompt(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut result = String::with_capacity(input.len());
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '{' && i + 1 < chars.len() && chars[i + 1] == '{' {
            if let Some(next) = scan_filtered_block(&chars, i + 2) {
                result.push_str("[FILTERED]");
                i = next;
            } else {
                result.push('{');
                result.push('{');
                i += 2;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

fn scan_filtered_block(chars: &[char], start: usize) -> Option<usize> {
    let mut k = start;
    while k + 1 < chars.len() {
        if chars[k] == '}' && chars[k + 1] == '}' {
            return if k > start { Some(k + 2) } else { None };
        }
        k += 1;
    }
    None
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// [TRIVIAL_ENUM]
pub(crate) enum Section {
    Role,
    Instructions,
    WritingStyle,
    GlobalRules,
    OutputFormat,
}

pub(crate) fn render_preset_xml_parts(
    preset: &PromptPreset,
    global_rules: &[String],
    response_length: Option<&str>,
    template_vars: Option<&TemplateVars>,
) -> Vec<(Section, String)> {
    let render_text = |source: &str| -> String {
        match template_vars {
            Some(template_vars) => render_template(source, template_vars),
            None => source.to_string(),
        }
    };
    let mut parts = Vec::new();
    if let Some(role) = preset.role.as_deref() {
        parts.push((Section::Role, wrap_xml(&render_text(role), "role")));
    }
    if let Some(instructions) = preset.instructions.as_deref() {
        parts.push((
            Section::Instructions,
            wrap_xml(&render_text(instructions), "instructions"),
        ));
    }
    if let Some(writing_style) = preset.writing_style.as_deref() {
        parts.push((
            Section::WritingStyle,
            wrap_xml(&render_text(writing_style), "writing_style"),
        ));
    }
    if !global_rules.is_empty() {
        let rules_text = global_rules
            .iter()
            .map(|global_rule| format!("- {}", render_text(global_rule)))
            .collect::<Vec<_>>()
            .join("\n");
        parts.push((Section::GlobalRules, wrap_xml(&rules_text, "global_rules")));
    }
    if let Some(output_format) = &preset.output_format {
        let mut output_text = render_text(output_format);
        if let Some(response_length) = response_length {
            output_text.push_str("\n\nResponse Length:\n");
            output_text.push_str(response_length);
        }
        parts.push((
            Section::OutputFormat,
            wrap_xml(&output_text, "output_format"),
        ));
    }
    parts
}

pub(crate) fn build_system_prompt(
    preset: &PromptPreset,
    global_rules: &[String],
    vars: &TemplateVars,
) -> String {
    render_preset_xml_parts(preset, global_rules, None, Some(vars))
        .into_iter()
        .filter(|(section, _)| {
            matches!(
                section,
                Section::Role | Section::Instructions | Section::GlobalRules
            )
        })
        .map(|(_, rendered)| rendered)
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub(crate) fn build_post_history_prompt(
    preset: &PromptPreset,
    response_length: Option<&str>,
    vars: &TemplateVars,
) -> String {
    render_preset_xml_parts(preset, &[], response_length, Some(vars))
        .into_iter()
        .filter(|(section, _)| matches!(section, Section::WritingStyle | Section::OutputFormat))
        .map(|(_, rendered)| rendered)
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub(crate) fn render_known_npc_entry(
    npc: &NpcCard,
    in_area_ids: &std::collections::HashSet<&str>,
    template_vars: &TemplateVars,
) -> String {
    let presence = if in_area_ids.contains(npc.id.as_str()) {
        "(in room)"
    } else {
        "(elsewhere)"
    };
    let mut entry = format!("- {} {}\n", npc.sheet.name, presence);

    let summary_text = npc
        .sheet
        .summary
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            npc.sheet
                .description
                .lines()
                .take(3)
                .collect::<Vec<_>>()
                .join("\n")
        });
    let rendered_summary = render_template(&summary_text, template_vars);

    for line in rendered_summary.lines() {
        entry.push_str(&format!("  {line}\n"));
    }
    entry.push('\n');
    entry
}

pub(crate) fn render_present_relationships(
    npc: &NpcCard,
    all_npcs: &[NpcCard],
    in_area_ids: &std::collections::HashSet<&str>,
    template_vars: &TemplateVars,
) -> Option<String> {
    let present_relations: Vec<&Relationship> = npc
        .relationships
        .iter()
        .filter(|r| in_area_ids.contains(r.with.as_str()))
        .collect();

    if present_relations.is_empty() {
        return None;
    }

    let mut block = String::from("Relationships:\n");
    for rel in present_relations {
        let partner_name = all_npcs
            .iter()
            .find(|n| n.id == rel.with)
            .map(|n| n.sheet.name.as_str())
            .unwrap_or(&rel.with);
        block.push_str(&format!(
            "  → {}: {}\n",
            partner_name,
            render_template(rel.display_text(), template_vars)
        ));
    }
    Some(block)
}
