//! [DOC: docs/diataxis/reference/game_flow.md]
//! The game state of one Swipe: movement, scene and encounter log

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::model::character::NpcCard;
use crate::domain::model::map::{MapDef, Room};
use crate::domain::model::scenario::StartingScenario;
use crate::domain::model::trigger::NpcEncounterLog;
use super::movement::MovementState;
use super::scene_state::SceneState;
use super::trigger_context::EventRecord;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwipeState {
    pub movement: MovementState,
    pub scene: SceneState,
    pub npc_encounter_log: NpcEncounterLog,
    /// The event record of the Trigger that fired for this Swipe.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_context: Option<EventRecord>,
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

    /// The room the player stands in: a map room, else an added room.
    pub fn room<'a>(&'a self, map: &'a MapDef) -> Option<&'a Room> {
        self.room_by_id(map, &self.movement.current_room_id)
    }

    pub fn room_by_id<'a>(&'a self, map: &'a MapDef, room_id: &str) -> Option<&'a Room> {
        map.get_room_by_id(room_id)
            .or_else(|| self.movement.dynamic_rooms.get(room_id))
    }

    /// The state a Swipe holds from the text save until its analysis updates it:
    /// the start state, with no trigger context. An event generation marks its own
    /// one-shot Trigger fired, because the rules read the fired set from there.
    pub fn pre_analysis(&self, event: Option<&EventRecord>) -> Self {
        let mut state = self.clone();
        state.trigger_context = None;
        if let Some(record) = event {
            if !record.trigger.trigger_repeat {
                state
                    .npc_encounter_log
                    .mark_trigger_fired(&record.trigger.npc_id, record.trigger.trigger_idx);
            }
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
