//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Multi-stage prompt builder.

use crate::domain::model::character::{NpcCard, Relationship};
use crate::domain::model::prompt_preset::{PresetField, PromptPreset};
use crate::domain::model::template::TemplateVars;
use crate::domain::model::utils::template::render_template;

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

pub(crate) fn render_preset_xml_parts(
    preset: &PromptPreset,
    global_rules: &[String],
    response_length: Option<&str>,
    template_vars: Option<&TemplateVars>,
) -> Vec<(PresetField, String)> {
    preset.render_field_parts(global_rules, response_length, template_vars)
}

pub(crate) fn build_system_prompt(
    preset: &PromptPreset,
    global_rules: &[String],
    vars: &TemplateVars,
) -> String {
    render_preset_xml_parts(preset, global_rules, None, Some(vars))
        .into_iter()
        .filter(|(field, _)| {
            matches!(
                field,
                PresetField::Role | PresetField::Instructions | PresetField::GlobalRules
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
        .filter(|(field, _)| matches!(field, PresetField::WritingStyle | PresetField::OutputFormat))
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
