//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Retry/retrigger top-level orchestrators (shuttle work to background via spawn_pipeline_task).

use std::sync::Arc;

use crate::application::application_service::DefaultApplicationService;
use crate::application::ApplicationError;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;

pub fn retry(app: Arc<DefaultApplicationService>) -> Result<(), ApplicationError> {
    let game_state = app.load_or_fresh();

    if game_state.narrative.history.last_input_text().is_none() {
        return Err(ApplicationError::validation("No input to retry"));
    }

    let (_, cancelled) = app.prepare_retry_state(
        game_state,
        GenerationStatus::Generating,
        GenerationPhase::Narrating,
    )?;
    if cancelled {
        return Err(ApplicationError::ShuttingDown);
    }

    crate::application::spawn_pipeline_task(app, move |app_inner| {
        if app_inner.is_shutting_down() {
            return;
        }
        app_inner.retry_last_response();
    });

    Ok(())
}

pub fn retrigger(app: Arc<DefaultApplicationService>) -> Result<(), ApplicationError> {
    let game_state = app.load_or_fresh();

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

    let (_, cancelled) = app.prepare_retry_state(
        game_state,
        GenerationStatus::Generating,
        GenerationPhase::Narrating,
    )?;
    if cancelled {
        return Err(ApplicationError::ShuttingDown);
    }

    crate::application::spawn_pipeline_task(app, move |app_inner| {
        if app_inner.is_shutting_down() {
            return;
        }
        app_inner.retrigger_event();
    });

    Ok(())
}

impl DefaultApplicationService {
    /// Returns `cancelled=true` if shutdown was requested mid-call.
    pub(crate) fn prepare_retry_state(
        &self,
        mut game_state: GameState,
        status: GenerationStatus,
        phase: GenerationPhase,
    ) -> Result<(GameState, bool), ApplicationError> {
        game_state.narrative.input_buffer.status = status;
        game_state.narrative.input_buffer.phase = phase;
        let snapshot = GameStateSnapshot::from_game_state(&game_state);
        self.storage().save_snapshot(&snapshot)?;
        let cancelled = self.is_shutting_down();
        Ok((game_state, cancelled))
    }
}
