//! [DOC: docs/diataxis/reference/game_flow.md]
//! The game state of one Swipe: movement, scene and encounter log

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::model::character::NpcCard;
use crate::domain::model::scenario::StartingScenario;
use crate::domain::model::trigger::NpcEncounterLog;
use super::movement::MovementState;
use super::scene_state::SceneState;
use super::trigger_context::TriggerRef;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwipeState {
    pub movement: MovementState,
    pub scene: SceneState,
    pub npc_encounter_log: NpcEncounterLog,
    /// The Trigger that fired for this Swipe, kept as a reference: the prompt is
    /// assembled from it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_context: Option<TriggerRef>,
}

impl SwipeState {
    pub fn starting(
        scenario: Option<&StartingScenario>,
        characters: &HashMap<String, NpcCard>,
        starting_room: &str,
    ) -> Self {
        let mut state = Self {
            movement: MovementState {
                current_room_id: starting_room.to_string(),
                dynamic_rooms: HashMap::new(),
            },
            scene: SceneState::default(),
            npc_encounter_log: NpcEncounterLog::default(),
            trigger_context: None,
        };
        if let Some(scenario) = scenario {
            state.init_scenario_npcs(scenario, characters);
        }
        state
    }

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
