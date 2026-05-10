use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::character::{NpcCard, PlayerCard};
use crate::model::map::{MapDef, Room};
use crate::model::trigger::CharacterState;
use crate::model::world::WorldCard;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogType {
    Narration,
    Dialogue,
    System,
    Input,
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub id: u64,
    pub sender: Option<String>,
    pub text: String,
    pub log_type: LogType,
    pub timestamp: DateTime<Utc>,
}

const MAX_LOG_ENTRIES: usize = 1000;

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
pub struct GenerationState {
    pub input: String,
    pub cursor_position: usize,
    pub scroll_offset: u16,
    pub status: GenerationStatus,
    pub phase: GenerationPhase,
}

impl GenerationState {
    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
        self.cursor_position += 1;
    }

    pub fn pop_char(&mut self) {
        if !self.input.is_empty() {
            self.input.pop();
            self.cursor_position = self.cursor_position.saturating_sub(1);
        }
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
    }
}

// ─── Sub-state structs ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MovementState {
    pub current_room_id: String,
    pub dynamic_rooms: HashMap<String, Room>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NarrativeState {
    pub history: Vec<LogEntry>,
    pub next_log_id: u64,
    pub generation: GenerationState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneState {
    pub npcs_in_area: Vec<NpcCard>,
}

// ─── GameState ────────────────────────────────────────────────────────────────

/// [DOC: docs/architecture/system.md]
#[derive(Debug, Clone)]
pub struct GameState {
    pub world: Arc<WorldCard>,
    pub map: Arc<MapDef>,
    pub player: Arc<PlayerCard>,
    pub npcs: HashMap<String, NpcCard>,
    pub movement: MovementState,
    pub narrative: NarrativeState,
    pub scene: SceneState,
    pub character_state: CharacterState,
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
            narrative: snapshot.narrative.clone(),
            scene: snapshot.scene.clone(),
            character_state: snapshot.character_state.clone(),
        }
    }

    pub fn new(
        world: Arc<WorldCard>,
        map: Arc<MapDef>,
        player: Arc<PlayerCard>,
        npcs: Vec<NpcCard>,
        starting_room: String,
    ) -> Self {
        let mut npcs_map = HashMap::new();
        for npc in npcs {
            npcs_map.insert(npc.id.clone(), npc);
        }

        let mut character_state = CharacterState::default();

        for region in &map.overworld.regions {
            for room in &region.rooms {
                if room.id == starting_room {
                    for npc_id in &room.npcs {
                        if let Some(npc) = npcs_map.get(npc_id) {
                            let encounter_state =
                                character_state.npcs.entry(npc.id.clone()).or_default();
                            encounter_state.times_met = 1;
                            encounter_state.currently_meeting = true;
                        }
                    }
                    break;
                }
            }
        }

        Self {
            world,
            map,
            player,
            npcs: npcs_map,
            movement: MovementState {
                current_room_id: starting_room,
                dynamic_rooms: HashMap::new(),
            },
            narrative: NarrativeState {
                history: Vec::new(),
                next_log_id: 1,
                generation: GenerationState::default(),
            },
            scene: SceneState {
                npcs_in_area: Vec::new(),
            },
            character_state,
        }
    }

    pub fn add_log(&mut self, text: String, sender: Option<String>, log_type: LogType) {
        if self.narrative.history.len() >= MAX_LOG_ENTRIES {
            self.narrative.history.remove(0);
        }
        let id = self.narrative.next_log_id;
        self.narrative.next_log_id += 1;
        self.narrative.history.push(LogEntry {
            id,
            sender,
            text,
            log_type,
            timestamp: Utc::now(),
        });
    }

    /// [DOC: docs/architecture/system.md]
    pub fn edit_log(&mut self, id: u64, new_text: String) -> crate::error::Result<()> {
        let entry = self
            .narrative
            .history
            .iter_mut()
            .find(|e| e.id == id)
            .ok_or_else(|| {
                crate::error::EngineError::Internal(crate::error::internal_error(format!(
                    "Log entry not found: {id}"
                )))
            })?;
        entry.text = new_text;
        Ok(())
    }

    /// [DOC: docs/architecture/system.md]
    pub fn delete_log(&mut self, id: u64) -> crate::error::Result<()> {
        let idx = self
            .narrative
            .history
            .iter()
            .position(|e| e.id == id)
            .ok_or_else(|| {
                crate::error::EngineError::Internal(crate::error::internal_error(format!(
                    "Log entry not found: {id}"
                )))
            })?;
        self.narrative.history.remove(idx);
        Ok(())
    }

    pub fn get_log(&self, id: u64) -> Option<&LogEntry> {
        self.narrative.history.iter().find(|e| e.id == id)
    }

    pub fn get_last_ai_response_index(&self) -> Option<usize> {
        self.narrative
            .history
            .iter()
            .rposition(|e| e.log_type == LogType::Narration || e.log_type == LogType::Dialogue)
    }

    pub fn get_last_input_index(&self) -> Option<usize> {
        self.narrative
            .history
            .iter()
            .rposition(|e| e.log_type == LogType::Input)
    }

    pub fn get_last_input_text(&self) -> Option<(String, String)> {
        let input_idx = self.get_last_input_index()?;
        let input_entry = self.narrative.history.get(input_idx)?;
        let sender = input_entry.sender.clone().unwrap_or_default();
        Some((sender, input_entry.text.clone()))
    }

    pub fn get_history_context(&self) -> &[LogEntry] {
        &self.narrative.history
    }

    /// [DOC: docs/architecture/system.md]
    /// NOTE: Excludes the AI response being retried to prevent the LLM from repeating it.
    pub fn get_history_context_for_retry(&self) -> Vec<LogEntry> {
        let last_ai_idx = self.get_last_ai_response_index();
        if let Some(idx) = last_ai_idx {
            // Exclude the AI response being retried (and any entries after it)
            self.narrative.history[..idx].to_vec()
        } else {
            self.narrative.history.clone()
        }
    }

    /// [DOC: docs/architecture/system.md]
    pub fn replace_last_ai_response(&mut self, new_text: String) -> crate::error::Result<()> {
        let input_idx = self.get_last_input_index().ok_or_else(|| {
            crate::error::EngineError::Internal(crate::error::internal_error("No input to retry"))
        })?;
        let ai_idx = self.get_last_ai_response_index().ok_or_else(|| {
            crate::error::EngineError::Internal(crate::error::internal_error(
                "No AI response to retry",
            ))
        })?;

        if ai_idx <= input_idx {
            return Err(crate::error::EngineError::Internal(
                crate::error::internal_error("AI response must be after input"),
            ));
        }

        let entry = self.narrative.history.get_mut(ai_idx).ok_or_else(|| {
            crate::error::EngineError::Internal(crate::error::internal_error(
                "AI response not found",
            ))
        })?;
        entry.text = new_text;
        Ok(())
    }
}
