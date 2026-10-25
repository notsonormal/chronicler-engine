//! [DOC: docs/diataxis/reference/game_flow.md]
//! The game state of one Swipe: movement, the Characters present, encounters and the event record

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::model::character::CharacterCard;
use crate::domain::model::map::{MapDef, Room};
use crate::domain::model::scenario::StartingScenario;
use crate::domain::model::trigger::Encounters;
use super::movement::MovementState;
use super::scene_state::SceneState;
use super::event_record::EventRecord;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwipeState {
    pub movement: MovementState,
    pub scene: SceneState,
    /// The stored key is `npc_encounter_log`.
    #[serde(rename = "npc_encounter_log")]
    pub encounters: Encounters,
    /// The event record of the Trigger that fired for this Swipe.
    /// The stored key is `trigger_context`.
    #[serde(
        default,
        rename = "trigger_context",
        skip_serializing_if = "Option::is_none"
    )]
    pub event_record: Option<EventRecord>,
}

impl SwipeState {
    pub fn starting(
        scenario: Option<&StartingScenario>,
        characters: &HashMap<String, CharacterCard>,
        starting_room: &str,
    ) -> Self {
        let mut state = Self {
            movement: MovementState {
                current_room_id: starting_room.to_string(),
                dynamic_rooms: HashMap::new(),
            },
            scene: SceneState::default(),
            encounters: Encounters::default(),
            event_record: None,
        };
        if let Some(scenario) = scenario {
            state.init_scenario_characters(scenario, characters);
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

    pub fn init_scenario_characters(
        &mut self,
        scenario: &StartingScenario,
        characters: &HashMap<String, CharacterCard>,
    ) {
        for character_id in &scenario.characters_present {
            if let Some(character) = characters.get(character_id).cloned() {
                let encounter = self
                    .encounters
                    .characters
                    .entry(character_id.clone())
                    .or_default();
                encounter.times_met = 1;
                encounter.currently_meeting = true;
                if !self
                    .scene
                    .characters_present
                    .iter()
                    .any(|n| n.id == *character_id)
                {
                    self.scene.characters_present.push(character);
                }
            }
        }
    }
}
