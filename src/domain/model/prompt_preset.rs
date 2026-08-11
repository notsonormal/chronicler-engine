//! [DOC: docs/diataxis/reference/game_flow.md]
//! Prompt preset configurations

use serde::{Deserialize, Serialize};

/// [TRIVIAL_ENUM]
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
}
