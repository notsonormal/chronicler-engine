//! [DOC: docs/diataxis/reference/game_flow.md]
//! DebugStateView — debug-state DTO for the HTTP `/debug/state` endpoint.

use std::collections::HashMap;

use serde::Serialize;

use crate::domain::model::state::generation_status::{GenerationStatus, Step};
use crate::domain::model::state::message_types::StoryRow;
use crate::domain::model::trigger::NpcEncounterState;

#[derive(Clone, Serialize)]
pub struct DebugStateView {
    pub current_room_id: String,
    pub npcs_in_area: Vec<String>,
    pub generation_status: GenerationStatus,
    /// The step of the running Job; `None` when no Job runs.
    pub step: Option<Step>,
    pub npc_encounter_log: HashMap<String, NpcEncounterState>,
    pub narration_history_tail: Vec<StoryRow>,
    pub narration_history_length: usize,
    pub dynamic_rooms: Vec<String>,
    pub dynamic_room_count: usize,
    pub last_error: Option<String>,
    pub backend_name: Option<String>,
    pub model_name: Option<String>,
    pub current_options: Vec<String>,
}
