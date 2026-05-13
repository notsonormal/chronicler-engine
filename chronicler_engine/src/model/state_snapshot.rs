use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::state::{GameState, MovementState, NarrativeState, SceneState};
use crate::model::trigger::CharacterState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameStateSnapshot {
    pub id: String,
    pub turn_id: String,
    pub swipe_index: u32,
    pub movement: MovementState,
    pub narrative: NarrativeState,
    pub scene: SceneState,
    pub character_state: CharacterState,
    pub committed: bool,
    pub created_at: DateTime<Utc>,
}

impl GameStateSnapshot {
    pub fn from_game_state(state: &GameState, turn_id: String, swipe_index: u32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            turn_id,
            swipe_index,
            movement: state.movement.clone(),
            narrative: state.narrative.clone(),
            scene: state.scene.clone(),
            character_state: state.character_state.clone(),
            committed: false,
            created_at: Utc::now(),
        }
    }

    pub fn apply_to(&self, state: &mut GameState) {
        state.movement = self.movement.clone();
        state.narrative = self.narrative.clone();
        state.scene = self.scene.clone();
        state.character_state = self.character_state.clone();
    }
}
