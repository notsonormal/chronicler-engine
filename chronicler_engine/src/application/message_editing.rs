use std::sync::Arc;

use chrono::Utc;

use crate::application::ApplicationError;
use crate::application::context::{GameServiceContext, load_or_fresh};
use crate::application::game_service::DefaultGameService;
use crate::error::{EngineError, internal_error};
use crate::model::state_snapshot::GameStateSnapshot;

pub struct MessageEditingService {
    game_service: Arc<DefaultGameService>,
}

impl MessageEditingService {
    pub fn new(game_service: Arc<DefaultGameService>) -> Self {
        Self { game_service }
    }

    fn app_err_internal(msg: impl Into<String>) -> ApplicationError {
        ApplicationError::Engine(EngineError::Internal(internal_error(msg)))
    }

    fn prepare_retry_state(
        ctx: &GameServiceContext,
        mut game_state: crate::model::state::GameState,
        status: crate::model::state::GenerationStatus,
        phase: crate::model::state::GenerationPhase,
    ) -> Result<(crate::model::state::GameState, bool), ApplicationError> {
        game_state.narrative.input_buffer.status = status;
        game_state.narrative.input_buffer.phase = phase;
        let snapshot = GameStateSnapshot::from_game_state(&game_state);
        ctx.storage.save_snapshot(&snapshot)?;
        let cancelled = ctx.cancel_token.is_cancelled();
        Ok((game_state, cancelled))
    }

    pub fn switch_swipe(
        &self,
        ctx: GameServiceContext,
        message_id: u64,
        swipe_index: usize,
    ) -> Result<(), ApplicationError> {
        if ctx.is_generating.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(ApplicationError::ConcurrentGeneration);
        }

        let messages = ctx.load_messages()?;
        let is_last = messages.last().map(|m| m.id == message_id).unwrap_or(false);
        if !is_last {
            return Err(ApplicationError::validation(
                "Only the last message can be swiped",
            ));
        }

        ctx.storage.update_active_swipe(message_id, swipe_index)?;

        let target_msg = messages
            .iter()
            .find(|m| m.id == message_id)
            .ok_or_else(|| Self::app_err_internal("Message not found"))?;

        let target_swipe = target_msg
            .swipes
            .get(swipe_index)
            .ok_or_else(|| Self::app_err_internal("Swipe index out of bounds"))?;

        let snapshot_id = target_swipe
            .snapshot_id
            .ok_or_else(|| Self::app_err_internal("Swipe has no associated snapshot"))?;

        let mut snapshot = ctx
            .storage
            .load_snapshot_by_id(snapshot_id)?
            .ok_or_else(|| Self::app_err_internal("Snapshot not found"))?;

        snapshot.created_at = Utc::now();
        ctx.storage.save_snapshot(&snapshot)?;

        Ok(())
    }

    pub fn edit_history(
        &self,
        ctx: GameServiceContext,
        id: u64,
        text: String,
    ) -> Result<(), ApplicationError> {
        let latest = ctx.storage.load_latest_snapshot()?;
        let mut guard = load_or_fresh(&ctx);
        guard.narrative.history.edit(id, text.clone())?;

        if latest.is_some() {
            let snapshot = GameStateSnapshot::from_game_state(&guard);
            ctx.storage.save_snapshot(&snapshot)?;
            ctx.update_message_text(id, &text)?;
        }

        Ok(())
    }

    pub fn delete_last(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        let mut guard = load_or_fresh(&ctx);
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
        ctx.storage.save_snapshot(&snapshot)?;
        ctx.storage.delete_message(last_id)?;

        Ok(())
    }

    pub fn retry(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        let game_state = load_or_fresh(&ctx);

        if game_state.narrative.history.last_input_text().is_none() {
            return Err(ApplicationError::validation("No input to retry"));
        }

        let (_, cancelled) = Self::prepare_retry_state(
            &ctx,
            game_state,
            crate::model::state::GenerationStatus::Generating,
            crate::model::state::GenerationPhase::Narrating,
        )?;
        if cancelled {
            return Err(ApplicationError::ShuttingDown);
        }

        let game_service = Arc::clone(&self.game_service);
        let ctx_clone = ctx.clone();

        // [DOC: docs/architecture/invariants.md#INV-004]
        tokio::task::spawn_blocking(move || {
            if ctx_clone.cancel_token.is_cancelled() {
                return;
            }
            game_service.retry_last_response(ctx_clone);
        });

        Ok(())
    }

    pub fn retrigger(&self, ctx: GameServiceContext) -> Result<(), ApplicationError> {
        let game_state = load_or_fresh(&ctx);

        if game_state.narrative.last_trigger.is_none() {
            return Err(ApplicationError::validation("No trigger context available"));
        }

        let messages = ctx.load_messages()?;
        let Some(last_msg) = messages.last() else {
            return Err(ApplicationError::validation("No messages to retrigger"));
        };

        let is_narration = last_msg.message_type == crate::model::state::MessageType::Narration
            || last_msg.message_type == crate::model::state::MessageType::Dialogue;

        if !is_narration || last_msg.event_header().is_some() {
            return Err(ApplicationError::validation(
                "Last message must be a narration to retrigger",
            ));
        }

        let (_, cancelled) = Self::prepare_retry_state(
            &ctx,
            game_state,
            crate::model::state::GenerationStatus::Generating,
            crate::model::state::GenerationPhase::Narrating,
        )?;
        if cancelled {
            return Err(ApplicationError::ShuttingDown);
        }

        let game_service = Arc::clone(&self.game_service);
        let ctx_clone = ctx.clone();

        // [DOC: docs/architecture/invariants.md#INV-004]
        tokio::task::spawn_blocking(move || {
            if ctx_clone.cancel_token.is_cancelled() {
                return;
            }
            game_service.retrigger_event(ctx_clone);
        });

        Ok(())
    }
}
