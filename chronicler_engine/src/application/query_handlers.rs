//! [DOC: docs/system/game_flow.md]
//! Read-only data access for game state and debug views

use crate::application::ApplicationError;
use crate::application::DebugStateView;
use crate::application::application_service::DefaultApplicationService;
use crate::error::EngineError;
use crate::application::ports::llm_message_repository::LlmMessage;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageEntry;

pub fn get_generating_status(
    app: &DefaultApplicationService,
) -> Result<(GenerationStatus, GenerationPhase), ApplicationError> {
    let game_state = app.load_or_fresh();
    Ok((
        game_state.narrative.input_buffer.status.clone(),
        game_state.narrative.input_buffer.phase.clone(),
    ))
}

pub fn reset_generating_status(app: &DefaultApplicationService) -> Result<(), ApplicationError> {
    let mut game_state = app.load_or_fresh();
    game_state.narrative.input_buffer.status = GenerationStatus::Idle;
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &game_state,
        );
    app.storage().save_snapshot(&snapshot)?;
    Ok(())
}

pub fn get_current_game_name(app: &DefaultApplicationService) -> Result<String, ApplicationError> {
    match app.storage().get_game(app.storage().current_game_id())? {
        Some(g) => Ok(g.name),
        None => Ok("Unknown".to_string()),
    }
}

pub fn list_latest_llm_messages(
    app: &DefaultApplicationService,
    limit: usize,
) -> Result<Vec<LlmMessage>, ApplicationError> {
    app.storage()
        .list_latest_llm_messages(limit)
        .map_err(Into::into)
}

pub fn get_story_log_entries(
    app: &DefaultApplicationService,
) -> Result<(Vec<MessageEntry>, bool), ApplicationError> {
    let game_state = app.load_or_fresh();
    let entries: Vec<_> = game_state.narrative.history().to_vec();
    let has_last_trigger = game_state.narrative.last_trigger.is_some();
    Ok((entries, has_last_trigger))
}

pub fn get_input_status(
    app: &DefaultApplicationService,
) -> Result<(GenerationStatus, GenerationPhase), ApplicationError> {
    get_generating_status(app)
}

pub fn get_current_room_view(
    app: &DefaultApplicationService,
) -> Result<(String, Option<String>), ApplicationError> {
    let game_state = app.load_or_fresh();
    let room = game_state
        .current_room()
        .ok_or_else(|| EngineError::RoomNotFound("current room not found".to_string()))?;

    let image_path = room
        .image_path
        .clone()
        .or_else(|| game_state.world.default_room_image.clone());

    Ok((room.name.clone(), image_path))
}

pub fn get_npc_headshots(
    app: &DefaultApplicationService,
    scene_only: bool,
) -> Result<Vec<(String, String)>, ApplicationError> {
    let game_state = app.load_or_fresh();

    let npc_ids: Vec<String> = if scene_only {
        game_state
            .scene
            .npcs_in_area
            .iter()
            .map(|npc| npc.id.clone())
            .collect()
    } else {
        game_state.npcs.keys().cloned().collect()
    };

    let npc_data: Vec<(String, String)> = npc_ids
        .iter()
        .filter_map(|id| {
            let npc = game_state.npcs.get(id)?;
            let image_path = npc.sheet.preferred_image()?.to_string();
            let name = npc.sheet.name.clone();
            Some((image_path, name))
        })
        .collect();

    Ok(npc_data)
}

pub fn get_debug_state(
    app: &DefaultApplicationService,
) -> Result<DebugStateView, ApplicationError> {
    let game_state = app.load_or_fresh();

    let history_tail: Vec<MessageEntry> = game_state
        .narrative
        .history()
        .iter()
        .rev()
        .take(5)
        .rev()
        .cloned()
        .collect();

    let npc_ids: Vec<String> = game_state
        .scene
        .npcs_in_area
        .iter()
        .map(|npc| npc.id.clone())
        .collect();

    let dynamic_rooms: Vec<String> = game_state.movement.dynamic_rooms.keys().cloned().collect();

    let last_error = match &game_state.narrative.input_buffer.status {
        GenerationStatus::Error(msg) => Some(msg.clone()),
        _ => None,
    };

    Ok(DebugStateView {
        current_room_id: game_state.movement.current_room_id.clone(),
        npcs_in_area: npc_ids,
        generation_status: game_state.narrative.input_buffer.status.clone(),
        generation_phase: game_state.narrative.input_buffer.phase.clone(),
        npc_encounter_log: game_state.npc_encounter_log.npcs.clone(),
        narration_history_tail: history_tail,
        narration_history_length: game_state.narrative.history().len(),
        dynamic_rooms,
        dynamic_room_count: game_state.movement.dynamic_rooms.len(),
        last_error,
        quantifier_confidence: game_state.scene.quantifier_confidence.clone(),
        backend_name: game_state.narrative.last_backend_name.clone(),
        model_name: game_state.narrative.last_model_name.clone(),
    })
}
