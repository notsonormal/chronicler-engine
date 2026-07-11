//! [DOC: docs/system/game_flow.md]
//! Main game state and builder

use std::collections::HashMap;

use crate::domain::model::character::NpcCard;
use crate::domain::model::message::{Message, Swipe};
use crate::domain::model::trigger::NpcEncounterLog;
use super::message_types::MessageType;
use super::movement::MovementState;
use super::narrative_state::NarrativeState;
use super::scene_state::SceneState;

/// Mutable game state. World data lives on the orchestrator and threads through engine call sites as `&Arc<T>`/`&HashMap<_, _>` args.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct GameState {
    pub movement: MovementState,
    pub narrative: NarrativeState,
    pub scene: SceneState,
    pub npc_encounter_log: NpcEncounterLog,
}

// Builder uses Option + Default so new GameState fields don't break callers.
pub struct GameStateBuilder {
    starting_room: String,
    narrative: Option<NarrativeState>,
    scene: Option<SceneState>,
    npc_encounter_log: Option<NpcEncounterLog>,
}

impl GameStateBuilder {
    pub fn new(starting_room: impl Into<String>) -> Self {
        Self {
            starting_room: starting_room.into(),
            narrative: None,
            scene: None,
            npc_encounter_log: None,
        }
    }

    pub fn with_narrative(mut self, narrative: NarrativeState) -> Self {
        self.narrative = Some(narrative);
        self
    }

    pub fn with_scene(mut self, scene: SceneState) -> Self {
        self.scene = Some(scene);
        self
    }

    pub fn with_npc_encounter_log(mut self, log: NpcEncounterLog) -> Self {
        self.npc_encounter_log = Some(log);
        self
    }

    pub fn build(self) -> GameState {
        GameState {
            movement: MovementState {
                current_room_id: self.starting_room,
                dynamic_rooms: HashMap::new(),
            },
            narrative: self.narrative.unwrap_or_default(),
            scene: self.scene.unwrap_or_default(),
            npc_encounter_log: self.npc_encounter_log.unwrap_or_default(),
        }
    }
}

impl GameState {
    pub fn from_snapshot(
        snapshot: &crate::domain::model::state::game_state_snapshot::GameStateSnapshot,
    ) -> Self {
        Self {
            movement: snapshot.movement.clone(),
            narrative: NarrativeState::from_snapshot(&snapshot.narrative),
            scene: snapshot.scene.clone(),
            npc_encounter_log: snapshot.npc_encounter_log.clone(),
        }
    }

    pub fn new(starting_room: impl Into<String>) -> Self {
        GameStateBuilder::new(starting_room).build()
    }

    pub fn init_scenario_npcs(
        &mut self,
        scenario: &crate::domain::model::scenario::StartingScenario,
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

    fn push_message(&mut self, text: String, sender: Option<String>, message_type: MessageType) {
        let location_header = self.narrative.pending_location.take();
        let event_header = self.narrative.pending_event.take();

        if message_type == MessageType::Narration || message_type == MessageType::Dialogue {
            if let Some(ref mut target) = self.narrative.retry_target {
                let target_is_event = target.event_header().is_some();
                let new_is_event = event_header.is_some();
                if target_is_event == new_is_event {
                    let swipe = Swipe {
                        text: text.clone(),
                        snapshot_id: None,
                        location_header: location_header.clone(),
                        event_header: event_header.clone(),
                    };
                    target.swipes.push(swipe);
                    target.set_active_swipe(target.swipes.len() - 1);
                    return;
                }
            }
        }

        let message = Message::new(sender, text, message_type, location_header, event_header);
        self.narrative.history.append(message);
    }

    pub fn add_message(&mut self, text: String, sender: Option<String>, message_type: MessageType) {
        self.push_message(text, sender, message_type);
    }
}
