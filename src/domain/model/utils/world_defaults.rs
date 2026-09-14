//! [DOC: docs/diataxis/reference/narrative/agent_system.md]
//! Serde default-fn-pointers for WorldManifest fields.

use crate::domain::model::settings::{NarrativePerspective, NarrativeTense, NarratorMode};

pub fn default_map_file() -> String {
    "map.json".to_string()
}

pub fn default_world_narrator_mode() -> NarratorMode {
    NarratorMode::Novel
}

pub fn default_world_narrative_perspective() -> NarrativePerspective {
    NarrativePerspective::Third
}

pub fn default_world_narrative_tense() -> NarrativeTense {
    NarrativeTense::Past
}

pub fn default_options_always_on() -> bool {
    false
}
