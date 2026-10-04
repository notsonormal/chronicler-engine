//! [DOC: docs/diataxis/reference/game_flow.md]
//! Game state and session management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Game {
    pub id: u64,
    pub world_name: String,
    pub world_key: String,
    pub persona_key: String,
    pub persona_name: String,
    pub name: String,
    /// Player-facing label shown in the header and Games tab. Distinct from
    /// the stable generated `name`; display names may collide.
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub narrator_mode: NarratorMode,
    pub narrative_perspective: NarrativePerspective,
    pub narrative_tense: NarrativeTense,
    pub active_system_prompt_preset_id: String,
    pub active_quantifier_prompt_preset_id: String,
    pub active_impersonate_prompt_preset_id: String,
    /// Mode-agnostic options preset; inherited from `AppSettings` at creation.
    pub active_options_prompt_preset_id: String,
    /// Per-game override of the world's always-on options toggle.
    pub options_always_on: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresetSelection {
    pub system_id: String,
    pub quantifier_id: String,
    pub impersonate_id: String,
    pub options_id: String,
}

impl PresetSelection {
    pub fn new(
        system_id: impl Into<String>,
        quantifier_id: impl Into<String>,
        impersonate_id: impl Into<String>,
        options_id: impl Into<String>,
    ) -> Self {
        Self {
            system_id: system_id.into(),
            quantifier_id: quantifier_id.into(),
            impersonate_id: impersonate_id.into(),
            options_id: options_id.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewGame {
    pub world_name: String,
    pub world_key: String,
    pub persona_key: String,
    pub persona_name: String,
    /// The stable generated name. The stored display name is derived from it
    /// when the row is written; a rename is the only other way to set it.
    pub name: String,
    pub narrator_mode: NarratorMode,
    pub narrative_perspective: NarrativePerspective,
    pub narrative_tense: NarrativeTense,
    pub system_prompt_preset_id: String,
    pub quantifier_prompt_preset_id: String,
    pub impersonate_prompt_preset_id: String,
    pub options_prompt_preset_id: String,
    pub options_always_on: bool,
}
