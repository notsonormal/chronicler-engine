//! [DOC: docs/system/game_flow.md]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresetType {
    #[default]
    System,
    Quantifier,
}

impl PresetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PresetType::System => "system",
            PresetType::Quantifier => "quantifier",
        }
    }
}

impl TryFrom<&str> for PresetType {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "system" => Ok(PresetType::System),
            "quantifier" => Ok(PresetType::Quantifier),
            other => Err(format!("unknown preset type: {other}")),
        }
    }
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

    /// Section order: role → instructions → writing_style → global_rules → output_format
    pub fn assemble_prompt_text(
        &self,
        global_rules: &[String],
        response_length: Option<&str>,
    ) -> String {
        let mut parts: Vec<String> = Vec::new();

        push_section(&mut parts, self.role.as_deref(), "role");
        push_section(&mut parts, self.instructions.as_deref(), "instructions");
        push_section(&mut parts, self.writing_style.as_deref(), "writing_style");

        if !global_rules.is_empty() {
            let rules_text = global_rules
                .iter()
                .map(|r| format!("- {r}"))
                .collect::<Vec<_>>()
                .join("\n");
            parts.push(wrap_xml(&rules_text, "global_rules"));
        }

        if let Some(output_format) = &self.output_format {
            let mut output_text = output_format.clone();
            if let Some(length) = response_length {
                output_text.push_str("\n\nResponse Length:\n");
                output_text.push_str(length);
            }
            parts.push(wrap_xml(&output_text, "output_format"));
        }

        parts.join("\n\n")
    }
}

fn push_section(parts: &mut Vec<String>, content: Option<&str>, tag: &str) {
    if let Some(content) = content {
        parts.push(wrap_xml(content, tag));
    }
}

fn wrap_xml(content: &str, tag: &str) -> String {
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
