//! [DOC: docs/system/game_flow.md]
//! Main game state and builder

use std::collections::HashMap;
use std::sync::Arc;

use crate::model::character::{NpcCard, PlayerCard};
use crate::model::map::{MapDef, Room};
use crate::model::message::{Message, Swipe};
use crate::model::trigger::NpcEncounterLog;
use crate::model::world::WorldCard;
use super::message_types::MessageType;
use super::movement::MovementState;
use super::narrative_state::NarrativeState;
use super::scene_state::SceneState;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct GameState {
    pub world: Arc<WorldCard>,
    pub map: Arc<MapDef>,
    pub player: Arc<PlayerCard>,
    pub npcs: HashMap<String, NpcCard>,
    pub movement: MovementState,
    pub narrative: NarrativeState,
    pub scene: SceneState,
    pub npc_encounter_log: NpcEncounterLog,
}

// Builder uses Option + Default so new GameState fields don't break callers.
pub struct GameStateBuilder {
    world: Arc<WorldCard>,
    map: Arc<MapDef>,
    player: Arc<PlayerCard>,
    starting_room: String,
    npcs: Vec<NpcCard>,
    narrative: Option<NarrativeState>,
    scene: Option<SceneState>,
    npc_encounter_log: Option<NpcEncounterLog>,
}

impl GameStateBuilder {
    pub fn new(
        world: Arc<WorldCard>,
        map: Arc<MapDef>,
        player: Arc<PlayerCard>,
        starting_room: impl Into<String>,
    ) -> Self {
        Self {
            world,
            map,
            player,
            starting_room: starting_room.into(),
            npcs: Vec::new(),
            narrative: None,
            scene: None,
            npc_encounter_log: None,
        }
    }

    pub fn with_npcs(mut self, npcs: Vec<NpcCard>) -> Self {
        self.npcs = npcs;
        self
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
        let mut npcs_map = HashMap::new();
        for npc in self.npcs {
            npcs_map.insert(npc.id.clone(), npc);
        }

        GameState {
            world: self.world,
            map: self.map,
            player: self.player,
            npcs: npcs_map,
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
        snapshot: &crate::model::state_snapshot::GameStateSnapshot,
        world: Arc<WorldCard>,
        map: Arc<MapDef>,
        player: Arc<PlayerCard>,
        npcs: HashMap<String, NpcCard>,
    ) -> Self {
        Self {
            world,
            map,
            player,
            npcs,
            movement: snapshot.movement.clone(),
            narrative: NarrativeState::from_snapshot(&snapshot.narrative),
            scene: snapshot.scene.clone(),
            npc_encounter_log: snapshot.npc_encounter_log.clone(),
        }
    }

    pub fn new(
        world: Arc<WorldCard>,
        map: Arc<MapDef>,
        player: Arc<PlayerCard>,
        npcs: Vec<NpcCard>,
        starting_room: String,
    ) -> Self {
        GameStateBuilder::new(world, map, player, starting_room)
            .with_npcs(npcs)
            .build()
    }

    pub fn init_scenario_npcs(&mut self, scenario: &crate::model::scenario::StartingScenario) {
        for npc_id in &scenario.npcs {
            if let Some(npc) = self.npcs.get(npc_id).cloned() {
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

    pub fn current_room(&self) -> Option<&Room> {
        self.map
            .get_room_by_id(&self.movement.current_room_id)
            .or_else(|| {
                self.movement
                    .dynamic_rooms
                    .get(&self.movement.current_room_id)
            })
    }
}
