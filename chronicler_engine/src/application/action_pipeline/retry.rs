use std::sync::Arc;

use crate::application::action_pipeline::pipeline::{
    ActionOutcome, ActionPipeline, ActionPipelineBackend,
};
use crate::application::context::{GameServiceContext, load_state, save_state};
use crate::model::message::Swipe;
use crate::model::state::{GameState, GenerationPhase, GenerationStatus, LogType};

/// [DOC: docs/architecture/system.md]
pub fn retry_last_response_impl<B: ActionPipelineBackend>(backend: &B, ctx: GameServiceContext) {
    let messages = match ctx.load_messages() {
        Ok(msgs) => msgs,
        Err(e) => {
            log::error!("Failed to load messages: {e}");
            return;
        }
    };

    if messages.is_empty() {
        log::error!("No messages to retry");
        return;
    }

    let is_event = messages
        .last()
        .map(|m| m.event_header.is_some())
        .unwrap_or(false);

    // Find the anchor message: the message whose snapshot_id we'll restore to.
    let anchor_idx = if is_event {
        // For event retry: find the last message before any event messages.
        messages.iter().rposition(|m| m.event_header.is_none())
    } else {
        // For main narration retry: find the last input message.
        messages.iter().rposition(|m| m.log_type == LogType::Input)
    };

    let Some(anchor_idx) = anchor_idx else {
        log::error!("No anchor message found for retry");
        save_retry_error(&ctx, "Retry failed: no anchor message");
        return;
    };

    let anchor_msg = &messages[anchor_idx];
    let Some(snapshot_id) = anchor_msg.snapshot_id else {
        log::error!("Anchor message has no snapshot_id");
        save_retry_error(&ctx, "Retry failed: missing snapshot_id");
        return;
    };

    // Extract swipes from the message being retried before soft-deleting.
    let old_target = match messages.last() {
        Some(m) => m,
        None => {
            log::error!("No messages to extract swipes from");
            return;
        }
    };
    let pending_swipes: Vec<Swipe> = old_target.swipes.clone();
    let to_delete: Vec<u64> = messages.iter().skip(anchor_idx + 1).map(|m| m.id).collect();

    for id in &to_delete {
        if let Err(e) = ctx.message_storage.soft_delete_message(*id) {
            log::error!("Failed to soft-delete message {id} during retry: {e}");
            save_retry_error(
                &ctx,
                format!("Retry failed: could not soft-delete message {id}"),
            );
            return;
        }
    }

    let snapshot = match ctx.snapshot_storage.load_by_id(snapshot_id) {
        Ok(Some(s)) => s,
        Ok(None) => {
            log::error!("No snapshot found for id {snapshot_id}");
            let _ = ctx.message_storage.restore_soft_deleted(&to_delete);
            save_retry_error(
                &ctx,
                format!("Retry failed: no snapshot found for id {snapshot_id}"),
            );
            return;
        }
        Err(e) => {
            log::error!("Failed to load snapshot: {e}");
            let _ = ctx.message_storage.restore_soft_deleted(&to_delete);
            save_retry_error(&ctx, format!("Retry failed: {e}"));
            return;
        }
    };

    let mut state = GameState::from_snapshot(
        &snapshot,
        Arc::clone(&ctx.world),
        Arc::clone(&ctx.map),
        Arc::clone(&ctx.player),
        (*ctx.npcs).clone(),
    );
    let mut truncated = messages;
    truncated.truncate(anchor_idx + 1);
    state.narrative.history.replace(truncated);

    let input_text = match state.narrative.history.last_input_text() {
        Some((_sender, text)) => text,
        None => {
            log::error!("No input to retry");
            let _ = ctx.message_storage.restore_soft_deleted(&to_delete);
            return;
        }
    };

    let outcome = if is_event {
        retry_event_continuation(backend, &ctx, state)
    } else {
        retry_main_narration(backend, &ctx, state, input_text)
    };

    match outcome {
        ActionOutcome::Completed => {
            if let Err(e) = post_retry_swipe_migration(&ctx, &to_delete, &pending_swipes, is_event)
            {
                log::error!("Failed to migrate swipes after retry: {e}");
                let _ = ctx.message_storage.restore_soft_deleted(&to_delete);
                save_retry_error(&ctx, format!("Retry failed: swipe migration error: {e}"));
            }
        }
        ActionOutcome::Error { message } => {
            let _ = ctx.message_storage.restore_soft_deleted(&to_delete);
            save_retry_error(&ctx, message);
        }
        ActionOutcome::Cancelled => {
            let _ = ctx.message_storage.restore_soft_deleted(&to_delete);
        }
    }
}

fn post_retry_swipe_migration(
    ctx: &GameServiceContext,
    to_delete: &[u64],
    pending_swipes: &[Swipe],
    is_event: bool,
) -> Result<(), crate::error::EngineError> {
    let new_messages = ctx.load_messages()?;
    let new_target = if is_event {
        new_messages.iter().rev().find(|m| m.event_header.is_some())
    } else {
        new_messages.iter().rev().find(|m| {
            (m.log_type == LogType::Narration || m.log_type == LogType::Dialogue)
                && m.event_header.is_none()
        })
    };

    if let Some(target) = new_target {
        let offset = pending_swipes.len();
        ctx.migrate_swipes(target.id, pending_swipes, offset, to_delete)?;
    } else {
        ctx.message_storage.purge_soft_deleted(to_delete)?;
    }

    Ok(())
}

pub(crate) fn save_retry_error(ctx: &GameServiceContext, message: impl Into<String>) {
    let mut state = load_state(ctx);
    state.narrative.input_buffer.status = GenerationStatus::Error(message.into());
    if let Err(e) = save_state(ctx, &state) {
        log::error!("Critical: failed to persist retry error state: {e}");
    }
}

pub(crate) fn retry_event_continuation<B: ActionPipelineBackend>(
    backend: &B,
    ctx: &GameServiceContext,
    state: GameState,
) -> ActionOutcome {
    let Some(trigger) = state.narrative.last_trigger.clone() else {
        log::error!("Missing trigger context for event retry");
        save_retry_error(ctx, "Retry failed: missing trigger context");
        return ActionOutcome::Error {
            message: "Retry failed: missing trigger context".to_string(),
        };
    };

    let input_text = match state.narrative.history.last_input_text() {
        Some((_sender, text)) => text,
        None => String::new(),
    };

    let pipeline = ActionPipeline::new(backend, ctx);
    pipeline.run_trigger_continuation(state, trigger, &input_text)
}

pub(crate) fn retry_main_narration<B: ActionPipelineBackend>(
    backend: &B,
    ctx: &GameServiceContext,
    state: GameState,
    input_text: String,
) -> ActionOutcome {
    let pipeline = ActionPipeline::new(backend, ctx);
    pipeline.run_from_input(state, input_text)
}

/// Assumes the caller has already verified `last_trigger` exists and saved a generating snapshot.
/// [DOC: docs/architecture/system.md]
pub fn retrigger_event_impl<B: ActionPipelineBackend>(backend: &B, ctx: &GameServiceContext) {
    let state = load_state(ctx);

    let outcome = retry_event_continuation(backend, ctx, state);

    match outcome {
        ActionOutcome::Completed => {}
        ActionOutcome::Error { message } => {
            save_retry_error(ctx, message);
        }
        ActionOutcome::Cancelled => {
            let mut state = load_state(ctx);
            state.narrative.input_buffer.status = GenerationStatus::Idle;
            state.narrative.input_buffer.phase = GenerationPhase::default();
            let _ = save_state(ctx, &state);
        }
    }
}
