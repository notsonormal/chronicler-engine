//! [DOC: docs/diataxis/reference/game_flow.md]
//! Prompt preset configurations

use serde::{Deserialize, Serialize};

use crate::domain::model::template::TemplateVars;
use crate::domain::model::utils::template::render_template;
use crate::domain::model::utils::xml::wrap_xml;

/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresetType {
    #[default]
    System,
    Quantifier,
    Impersonate,
}

impl PresetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PresetType::System => "system",
            PresetType::Quantifier => "quantifier",
            PresetType::Impersonate => "impersonate",
        }
    }
}

impl TryFrom<&str> for PresetType {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "system" => Ok(PresetType::System),
            "quantifier" => Ok(PresetType::Quantifier),
            "impersonate" => Ok(PresetType::Impersonate),
            other => Err(format!("unknown preset type: {other}")),
        }
    }
}

/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetField {
    Role,
    Instructions,
    WritingStyle,
    GlobalRules,
    OutputFormat,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PromptPreset {
    pub id: String,
    pub name: String,
    pub role: Option<String>,
    pub instructions: Option<String>,
    pub writing_style: Option<String>,
    pub output_format: Option<String>,
    pub is_default: bool,
    pub preset_type: PresetType,
}

impl PromptPreset {
    pub fn preview_text(&self) -> &str {
        self.role
            .as_deref()
            .or(self.instructions.as_deref())
            .or(self.writing_style.as_deref())
            .or(self.output_format.as_deref())
            .unwrap_or("")
    }

    pub fn assemble_text(
        &self,
        global_rules: &[String],
        response_length: Option<&str>,
        template_vars: Option<&TemplateVars>,
    ) -> String {
        self.render_field_parts(global_rules, response_length, template_vars)
            .into_iter()
            .map(|(_, rendered)| rendered)
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    pub(crate) fn render_field_parts(
        &self,
        global_rules: &[String],
        response_length: Option<&str>,
        template_vars: Option<&TemplateVars>,
    ) -> Vec<(PresetField, String)> {
        let render_text = |source: &str| -> String {
            match template_vars {
                Some(template_vars) => render_template(source, template_vars),
                None => source.to_string(),
            }
        };
        let mut parts = Vec::new();
        if let Some(role) = self.role.as_deref() {
            parts.push((PresetField::Role, wrap_xml(&render_text(role), "role")));
        }
        if let Some(instructions) = self.instructions.as_deref() {
            parts.push((
                PresetField::Instructions,
                wrap_xml(&render_text(instructions), "instructions"),
            ));
        }
        if let Some(writing_style) = self.writing_style.as_deref() {
            parts.push((
                PresetField::WritingStyle,
                wrap_xml(&render_text(writing_style), "writing_style"),
            ));
        }
        if !global_rules.is_empty() {
            let rules_text = global_rules
                .iter()
                .map(|global_rule| format!("- {}", render_text(global_rule)))
                .collect::<Vec<_>>()
                .join("\n");
            parts.push((
                PresetField::GlobalRules,
                wrap_xml(&rules_text, "global_rules"),
            ));
        }
        if let Some(output_format) = &self.output_format {
            let mut output_text = render_text(output_format);
            if let Some(response_length) = response_length {
                output_text.push_str("\n\nResponse Length:\n");
                output_text.push_str(response_length);
            }
            parts.push((
                PresetField::OutputFormat,
                wrap_xml(&output_text, "output_format"),
            ));
        }
        parts
    }
}
