use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::character::{NpcCard, PlayerCard};
use crate::model::map::{MapDef, Room};
use crate::model::message::Message;
use crate::model::message_history::MessageHistory;
use crate::model::trigger::NpcEncounterLog;
use crate::model::world::WorldCard;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogType {
    Narration,
    Dialogue,
    System,
    Input,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub id: u64,
    pub sender: Option<String>,
    pub text: String,
    pub log_type: LogType,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub location_header: Option<String>,
    #[serde(default)]
    pub event_header: Option<String>,
    #[serde(default)]
    pub swipe_count: usize,
    #[serde(default)]
    pub active_swipe_index: usize,
}

impl Default for LogEntry {
    fn default() -> Self {
        Self {
            id: 0,
            sender: None,
            text: String::new(),
            log_type: LogType::Narration,
            timestamp: Utc::now(),
            location_header: None,
            event_header: None,
            swipe_count: 1,
            active_swipe_index: 0,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum GenerationStatus {
    #[default]
    Idle,
    Generating,
    Error(String),
}

impl GenerationStatus {
    pub fn is_generating(&self) -> bool {
        matches!(self, Self::Generating)
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::Error(msg) => Some(msg),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum GenerationPhase {
    #[default]
    Narrating,
    Quantifying,
    GeneratingEvent,
}

impl GenerationPhase {
    pub fn display_text(&self) -> &'static str {
        match self {
            Self::Narrating => "Generating narration...",
            Self::Quantifying => "Quantifying scene...",
            Self::GeneratingEvent => "Generating event...",
        }
    }

    pub fn as_endpoint_str(&self) -> &'static str {
        match self {
            Self::Narrating => "narrating",
            Self::Quantifying => "quantifying",
            Self::GeneratingEvent => "generating-event",
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputBuffer {
    pub input: String,
    pub status: GenerationStatus,
    pub phase: GenerationPhase,
}

// ─── Sub-state structs ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MovementState {
    pub current_room_id: String,
    pub dynamic_rooms: HashMap<String, Room>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredTriggerContext {
    pub npc_id: String,
    pub trigger_idx: usize,
    pub trigger_name: String,
    pub trigger_repeat: bool,
    pub trigger_narration_prompt: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NarrativeState {
    pub history: MessageHistory,
    pub input_buffer: InputBuffer,
    pub last_trigger: Option<StoredTriggerContext>,
    #[serde(default)]
    pub pending_location: Option<String>,
    #[serde(default)]
    pub pending_event: Option<String>,
    /// Name of the backend used for the last narration call.
    #[serde(default)]
    pub last_backend_name: Option<String>,
    /// Name of the model used for the last narration call.
    #[serde(default)]
    pub last_model_name: Option<String>,
}

impl NarrativeState {
    pub fn history(&self) -> Vec<LogEntry> {
        self.history.to_log_entries()
    }

    pub fn from_snapshot(snapshot: &crate::model::state_snapshot::NarrativeSnapshot) -> Self {
        Self {
            history: MessageHistory::new(),
            input_buffer: snapshot.input_buffer.clone(),
            last_trigger: snapshot.last_trigger.clone(),
            pending_location: snapshot.pending_location.clone(),
            pending_event: snapshot.pending_event.clone(),
            last_backend_name: snapshot.last_backend_name.clone(),
            last_model_name: snapshot.last_model_name.clone(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneState {
    pub npcs_in_area: Vec<NpcCard>,
    /// Confidence of the last quantifier run (High, Medium, Low, or None if not run).
    #[serde(default)]
    pub quantifier_confidence: Option<String>,
}

// ─── GameState ────────────────────────────────────────────────────────────────

/// [DOC: docs/architecture/system.md]
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

/// New fields added to `GameState` get a `Default::default()` fallback here,
/// so existing call sites do not break when the struct grows.
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

    fn push_message(&mut self, text: String, sender: Option<String>, log_type: LogType) {
        let location_header = self.narrative.pending_location.take();
        let event_header = self.narrative.pending_event.take();
        let message = Message::new(sender, text, log_type, location_header, event_header);
        self.narrative.history.append(message);
    }

    pub fn add_log(&mut self, text: String, sender: Option<String>, log_type: LogType) {
        self.push_message(text, sender, log_type);
    }

    /// [DOC: docs/system/navigation.md]
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
