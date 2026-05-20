use serde::{Deserialize, Serialize};

/// [DOC: docs/architecture/system.md]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresetType {
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

/// [DOC: docs/architecture/system.md]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptPreset {
    pub id: String,
    pub name: String,
    pub prompt_text: String,
    pub is_default: bool,
    pub preset_type: PresetType,
}
