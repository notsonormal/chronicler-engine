//! [DOC: docs/system/game_flow.md]
//! State snapshot serialization

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::state::{
    GameState, InputBuffer, MovementState, NarrativeState, SceneState, StoredTriggerContext,
};
use crate::model::trigger::NpcEncounterLog;

/// [`NarrativeState`] sans messages (separate table, hydrated on load).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NarrativeSnapshot {
    #[serde(rename = "generation")]
    pub input_buffer: InputBuffer,
    pub last_trigger: Option<StoredTriggerContext>,
    #[serde(default)]
    pub pending_location: Option<String>,
    #[serde(default)]
    pub pending_event: Option<String>,
    #[serde(default)]
    pub last_backend_name: Option<String>,
    #[serde(default)]
    pub last_model_name: Option<String>,
}

impl NarrativeSnapshot {
    pub fn from_narrative(state: &NarrativeState) -> Self {
        Self {
            input_buffer: state.input_buffer.clone(),
            last_trigger: state.last_trigger.clone(),
            pending_location: state.pending_location.clone(),
            pending_event: state.pending_event.clone(),
            last_backend_name: state.last_backend_name.clone(),
            last_model_name: state.last_model_name.clone(),
        }
    }
}

/// Frozen game state (persisted to `game_state_snapshots` table).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameStateSnapshot {
    pub db_id: Option<u64>,
    pub movement: MovementState,
    pub narrative: NarrativeSnapshot,
    pub scene: SceneState,
    #[serde(rename = "character_state")]
    pub npc_encounter_log: NpcEncounterLog,
    pub created_at: DateTime<Utc>,
}

impl GameStateSnapshot {
    pub fn from_game_state(state: &GameState) -> Self {
        Self {
            db_id: None,
            movement: state.movement.clone(),
            narrative: NarrativeSnapshot::from_narrative(&state.narrative),
            scene: state.scene.clone(),
            npc_encounter_log: state.npc_encounter_log.clone(),
            created_at: Utc::now(),
        }
    }
}
