//! [DOC: docs/diataxis/reference/narrative/narrator_mode.md]
//! Prompt preset configurations

use serde::{Deserialize, Serialize};

use crate::domain::model::generation_parameters::GenerationParameters;
use crate::domain::model::settings::NarratorMode;
use crate::domain::model::template::TemplateVars;
use crate::domain::model::utils::settings_defaults;
use crate::domain::model::utils::xml::wrap_xml;
use crate::error::EngineError;

/// [TRIVIAL_ENUM]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresetType {
    #[default]
    System,
    Quantifier,
    Impersonate,
    Options,
}

impl PresetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PresetType::System => "system",
            PresetType::Quantifier => "quantifier",
            PresetType::Impersonate => "impersonate",
            PresetType::Options => "options",
        }
    }

    /// Whether this type holds one active slot per narrator mode. Options
    /// presets serve both modes from a single settings-level slot, so
    /// `allowed_modes` does not gate their activation.
    pub fn is_mode_tagged(self) -> bool {
        !matches!(self, PresetType::Options)
    }
}

impl TryFrom<&str> for PresetType {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "system" => Ok(PresetType::System),
            "quantifier" => Ok(PresetType::Quantifier),
            "impersonate" => Ok(PresetType::Impersonate),
            "options" => Ok(PresetType::Options),
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptPreset {
    pub id: String,
    pub name: String,
    pub role: Option<String>,
    pub instructions: Option<String>,
    pub writing_style: Option<String>,
    pub output_format: Option<String>,
    #[serde(default = "settings_defaults::default_allowed_modes")]
    pub allowed_modes: Vec<NarratorMode>,
    pub is_default: bool,
    pub preset_type: PresetType,
    #[serde(default)]
    pub parameters: GenerationParameters,
    /// How many Options the dock offers. Only an Options preset sets it.
    #[serde(default)]
    pub option_count: Option<u32>,
}

impl Default for PromptPreset {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            role: None,
            instructions: None,
            writing_style: None,
            output_format: None,
            allowed_modes: settings_defaults::default_allowed_modes(),
            is_default: false,
            preset_type: PresetType::default(),
            parameters: GenerationParameters::default(),
            option_count: None,
        }
    }
}

impl PromptPreset {
    pub fn validate(&self) -> Result<(), EngineError> {
        if let Some(key) = self.parameters.refused_custom_key() {
            return Err(EngineError::Validation(format!(
                "Preset '{}' cannot set `{key}` in its custom parameters: the engine sets it",
                self.id
            )));
        }
        match self.option_count {
            Some(0) => Err(EngineError::Validation(format!(
                "Preset '{}' cannot offer 0 Options",
                self.id
            ))),
            Some(_) if self.preset_type != PresetType::Options => {
                Err(EngineError::Validation(format!(
                    "Preset '{}' is not an Options preset and cannot set an Options count",
                    self.id
                )))
            }
            _ => Ok(()),
        }
    }

    /// Whether this preset may be selected for `mode`.
    pub fn allows(&self, mode: NarratorMode) -> bool {
        self.allowed_modes.contains(&mode)
    }

    /// No-arg [`Self::allows`] form for askama templates (per-mode activation buttons).
    pub fn allows_novel(&self) -> bool {
        self.allows(NarratorMode::Novel)
    }

    /// No-arg [`Self::allows`] form for askama templates (per-mode activation buttons).
    pub fn allows_interactive_fiction(&self) -> bool {
        self.allows(NarratorMode::InteractiveFiction)
    }

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
        template_vars: Option<&TemplateVars>,
    ) -> String {
        self.render_field_parts(global_rules, template_vars)
            .into_iter()
            .map(|(_, rendered)| rendered)
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    pub(crate) fn render_field_parts(
        &self,
        global_rules: &[String],
        template_vars: Option<&TemplateVars>,
    ) -> Vec<(PresetField, String)> {
        let render_text = |source: &str| -> String {
            match template_vars {
                Some(template_vars) => template_vars.render(source),
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
            parts.push((
                PresetField::OutputFormat,
                wrap_xml(&render_text(output_format), "output_format"),
            ));
        }
        parts
    }
}
