//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! DebugStateView — debug-state DTO for the HTTP debug endpoint (T2 ticket 04 — extracted from DefaultApplicationService).

use std::collections::HashMap;

use serde::Serialize;

use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageEntry;
use crate::domain::model::trigger::NpcEncounterState;

#[derive(Clone, Serialize)]
pub struct DebugStateView {
    pub current_room_id: String,
    pub npcs_in_area: Vec<String>,
    pub generation_status: GenerationStatus,
    pub generation_phase: GenerationPhase,
    pub npc_encounter_log: HashMap<String, NpcEncounterState>,
    pub narration_history_tail: Vec<MessageEntry>,
    pub narration_history_length: usize,
    pub dynamic_rooms: Vec<String>,
    pub dynamic_room_count: usize,
    pub last_error: Option<String>,
    pub quantifier_confidence: Option<String>,
    pub backend_name: Option<String>,
    pub model_name: Option<String>,
}
