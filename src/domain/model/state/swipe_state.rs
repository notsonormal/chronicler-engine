//! [DOC: docs/diataxis/reference/game_flow.md]
//! The game state of one Swipe: movement, scene and encounter log

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::model::character::NpcCard;
use crate::domain::model::scenario::StartingScenario;
use crate::domain::model::trigger::NpcEncounterLog;
use super::movement::MovementState;
use super::scene_state::SceneState;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwipeState {
    pub movement: MovementState,
    pub scene: SceneState,
    pub npc_encounter_log: NpcEncounterLog,
}

impl SwipeState {
    pub fn init_scenario_npcs(
        &mut self,
        scenario: &StartingScenario,
        npcs: &HashMap<String, NpcCard>,
    ) {
        for npc_id in &scenario.npcs {
            if let Some(npc) = npcs.get(npc_id).cloned() {
                let encounter = self
                    .npc_encounter_log
                    .npcs
                    .entry(npc_id.clone())
                    .or_default();
                encounter.times_met = 1;
                encounter.currently_meeting = true;
                if !self.scene.npcs_in_area.iter().any(|n| n.id == *npc_id) {
                    self.scene.npcs_in_area.push(npc);
                }
            }
        }
    }
}
