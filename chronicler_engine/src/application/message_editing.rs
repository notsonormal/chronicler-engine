//! [DOC: docs/system/game_flow.md]
//! Message editing and modification utilities

use std::sync::Arc;

use chrono::Utc;

use crate::application::action_pipeline::{retry_last_response_impl, retrigger_event_impl};
use crate::application::application_service::DefaultApplicationService;
use crate::application::ApplicationError;
use crate::error::{EngineError, internal_error};
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;

fn app_err_internal(msg: impl Into<String>) -> ApplicationError {
    ApplicationError::Engine(EngineError::Internal(internal_error(msg)))
}

fn prepare_retry_state(
    app: &DefaultApplicationService,
    mut game_state: GameState,
    status: GenerationStatus,
    phase: GenerationPhase,
) -> Result<(GameState, bool), ApplicationError> {
    game_state.narrative.input_buffer.status = status;
    game_state.narrative.input_buffer.phase = phase;
    let snapshot = GameStateSnapshot::from_game_state(&game_state);
    app.storage.save_snapshot(&snapshot)?;
    let cancelled = app.cancel_token.is_cancelled();
    Ok((game_state, cancelled))
}

pub fn switch_swipe(
    app: &DefaultApplicationService,
    message_id: u64,
    swipe_index: usize,
) -> Result<(), ApplicationError> {
    if app.is_generating.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(ApplicationError::ConcurrentGeneration);
    }

    let messages = app.load_messages()?;
    let is_last = messages.last().map(|m| m.id == message_id).unwrap_or(false);
    if !is_last {
        return Err(ApplicationError::validation(
            "Only the last message can be swiped",
        ));
    }

    app.storage.update_active_swipe(message_id, swipe_index)?;

    let target_msg = messages
        .iter()
        .find(|m| m.id == message_id)
        .ok_or_else(|| app_err_internal("Message not found"))?;

    let target_swipe = target_msg
        .swipes
        .get(swipe_index)
        .ok_or_else(|| app_err_internal("Swipe index out of bounds"))?;

    let snapshot_id = target_swipe
        .snapshot_id
        .ok_or_else(|| app_err_internal("Swipe has no associated snapshot"))?;

    let mut snapshot = app
        .storage
        .load_snapshot_by_id(snapshot_id)?
        .ok_or_else(|| app_err_internal("Snapshot not found"))?;

    snapshot.created_at = Utc::now();
    app.storage.save_snapshot(&snapshot)?;

    Ok(())
}

pub fn edit_history(
    app: &DefaultApplicationService,
    id: u64,
    text: String,
) -> Result<(), ApplicationError> {
    let latest = app.storage.load_latest_snapshot()?;
    let mut guard = app.load_or_fresh()?;
    guard.narrative.history.edit(id, text.clone())?;

    if latest.is_some() {
        let snapshot = GameStateSnapshot::from_game_state(&guard);
        app.storage.save_snapshot(&snapshot)?;
        app.update_message_text(id, &text)?;
    }

    Ok(())
}

pub fn delete_last(app: &DefaultApplicationService) -> Result<(), ApplicationError> {
    let mut guard = app.load_or_fresh()?;
    let last_id = guard
        .narrative
        .history
        .last()
        .map(|m| m.id)
        .ok_or_else(|| {
            ApplicationError::Engine(EngineError::Internal(internal_error("History is empty")))
        })?;

    guard.narrative.history.delete_last()?;
    let snapshot = GameStateSnapshot::from_game_state(&guard);
    app.storage.save_snapshot(&snapshot)?;
    app.storage.delete_message(last_id)?;

    Ok(())
}

pub fn retry(app: Arc<DefaultApplicationService>) -> Result<(), ApplicationError> {
    let game_state = app.load_or_fresh()?;

    if game_state.narrative.history.last_input_text().is_none() {
        return Err(ApplicationError::validation("No input to retry"));
    }

    let (_, cancelled) = prepare_retry_state(
        &app,
        game_state,
        GenerationStatus::Generating,
        GenerationPhase::Narrating,
    )?;
    if cancelled {
        return Err(ApplicationError::ShuttingDown);
    }

    crate::application::spawn_pipeline_task(app, move |app_inner| {
        if app_inner.cancel_token.is_cancelled() {
            return;
        }
        retry_last_response_impl(app_inner);
    });

    Ok(())
}

pub fn retrigger(app: Arc<DefaultApplicationService>) -> Result<(), ApplicationError> {
    let game_state = app.load_or_fresh()?;

    if game_state.narrative.last_trigger.is_none() {
        return Err(ApplicationError::validation("No trigger context available"));
    }

    let messages = app.load_messages()?;
    let Some(last_msg) = messages.last() else {
        return Err(ApplicationError::validation("No messages to retrigger"));
    };

    let is_narration = last_msg.message_type == MessageType::Narration
        || last_msg.message_type == MessageType::Dialogue;

    if !is_narration || last_msg.event_header().is_some() {
        return Err(ApplicationError::validation(
            "Last message must be a narration to retrigger",
        ));
    }

    let (_, cancelled) = prepare_retry_state(
        &app,
        game_state,
        GenerationStatus::Generating,
        GenerationPhase::Narrating,
    )?;
    if cancelled {
        return Err(ApplicationError::ShuttingDown);
    }

    crate::application::spawn_pipeline_task(app, move |app_inner| {
        if app_inner.cancel_token.is_cancelled() {
            return;
        }
        retrigger_event_impl(app_inner);
    });

    Ok(())
}