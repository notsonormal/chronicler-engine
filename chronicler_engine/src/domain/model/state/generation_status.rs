//! [DOC: docs/system/game_flow.md]
//! Generation status enums and input buffer — phase/status are independent axes; live state machine lives in `application/action_pipeline/pipeline.rs`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum GenerationStatus {
    #[default]
    Idle,
    Generating,
    Error(String),
}

impl GenerationStatus {
    pub fn is_generating(&self) -> bool {
        matches!(self, Self::Generating)
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::Error(msg) => Some(msg),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum GenerationPhase {
    #[default]
    Narrating,
    Quantifying,
    GeneratingEvent,
}

impl GenerationPhase {
    pub fn display_text(&self) -> &'static str {
        match self {
            Self::Narrating => "Generating narration...",
            Self::Quantifying => "Quantifying scene...",
            Self::GeneratingEvent => "Generating event...",
        }
    }

    pub fn as_endpoint_str(&self) -> &'static str {
        match self {
            Self::Narrating => "narrating",
            Self::Quantifying => "quantifying",
            Self::GeneratingEvent => "generating-event",
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputBuffer {
    pub input: String,
    pub status: GenerationStatus,
    pub phase: GenerationPhase,
}
