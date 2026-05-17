use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::state::{
    GameState, GenerationState, MovementState, NarrativeState, SceneState, StoredTriggerContext,
};
use crate::model::trigger::CharacterState;

/// Persistable subset of [`NarrativeState`] — everything *except* messages.
/// Messages are stored in a separate table and hydrated after snapshot load.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NarrativeSnapshot {
    pub generation: GenerationState,
    pub last_trigger: Option<StoredTriggerContext>,
    #[serde(default)]
    pub pending_location: Option<String>,
    #[serde(default)]
    pub pending_event: Option<String>,
}

impl NarrativeSnapshot {
    pub fn from_narrative(state: &NarrativeState) -> Self {
        Self {
            generation: state.generation.clone(),
            last_trigger: state.last_trigger.clone(),
            pending_location: state.pending_location.clone(),
            pending_event: state.pending_event.clone(),
        }
    }
}

/// A frozen point-in-time of game state, stored in the `game_state_snapshots` table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameStateSnapshot {
    pub db_id: Option<u64>,
    pub movement: MovementState,
    pub narrative: NarrativeSnapshot,
    pub scene: SceneState,
    pub character_state: CharacterState,
    pub committed: bool,
    pub created_at: DateTime<Utc>,
}

impl GameStateSnapshot {
    pub fn from_game_state(state: &GameState) -> Self {
        Self {
            db_id: None,
            movement: state.movement.clone(),
            narrative: NarrativeSnapshot::from_narrative(&state.narrative),
            scene: state.scene.clone(),
            character_state: state.character_state.clone(),
            committed: false,
            created_at: Utc::now(),
        }
    }

    pub fn apply_to(&self, state: &mut crate::model::state::GameState) {
        state.movement = self.movement.clone();
        state.narrative.generation = self.narrative.generation.clone();
        state.narrative.last_trigger = self.narrative.last_trigger.clone();
        state.narrative.pending_location = self.narrative.pending_location.clone();
        state.narrative.pending_event = self.narrative.pending_event.clone();
        state.scene = self.scene.clone();
        state.character_state = self.character_state.clone();
    }
}
