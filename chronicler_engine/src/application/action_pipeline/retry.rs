use std::sync::Arc;

use crate::application::action_pipeline::pipeline::{
    ActionOutcome, ActionPipeline, ActionPipelineBackend,
};
use crate::application::context::{GameServiceContext, load_state, save_state};
use crate::model::state::{GameState, GenerationStatus, LogType};

/// [DOC: docs/architecture/system.md]
pub fn retry_last_response_impl<B: ActionPipelineBackend>(backend: &B, ctx: GameServiceContext) {
    let mut messages = match ctx.message_storage.load_messages() {
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

    // Delete messages after the anchor.
    let to_delete: Vec<u64> = messages.iter().skip(anchor_idx + 1).map(|m| m.id).collect();
    for id in &to_delete {
        if let Err(e) = ctx.message_storage.delete_message(*id) {
            log::error!("Failed to delete message {id} during retry: {e}");
            save_retry_error(&ctx, format!("Retry failed: could not delete message {id}"));
            return;
        }
    }
    messages.truncate(anchor_idx + 1);

    // Load and apply the anchor snapshot.
    let snapshot = match ctx.snapshot_storage.load_by_id(snapshot_id) {
        Ok(Some(s)) => s,
        Ok(None) => {
            log::error!("No snapshot found for id {snapshot_id}");
            return;
        }
        Err(e) => {
            log::error!("Failed to load snapshot: {e}");
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
    state.narrative.history.replace(messages);

    let input_text = match state.narrative.history.last_input_text() {
        Some((_sender, text)) => text,
        None => {
            log::error!("No input to retry");
            return;
        }
    };

    if is_event {
        retry_event_continuation(backend, &ctx, state);
    } else {
        retry_main_narration(backend, &ctx, state, input_text);
    }
}

pub(crate) fn save_retry_error(ctx: &GameServiceContext, message: impl Into<String>) {
    let mut state = load_state(ctx);
    state.narrative.input_buffer.status = GenerationStatus::Error(message.into());
    if let Err(e) = save_state(ctx, &mut state) {
        log::error!("Critical: failed to persist retry error state: {e}");
    }
}

pub(crate) fn retry_event_continuation<B: ActionPipelineBackend>(
    backend: &B,
    ctx: &GameServiceContext,
    state: GameState,
) {
    let Some(trigger) = state.narrative.last_trigger.clone() else {
        log::error!("Missing trigger context for event retry");
        save_retry_error(ctx, "Retry failed: missing trigger context");
        return;
    };

    let input_text = match state.narrative.history.last_input_text() {
        Some((_sender, text)) => text,
        None => String::new(),
    };

    let pipeline = ActionPipeline::new(backend, ctx);
    match pipeline.run_trigger_continuation(state, trigger, &input_text) {
        ActionOutcome::Completed => {}
        ActionOutcome::Error { message } => {
            save_retry_error(ctx, message);
        }
        ActionOutcome::Cancelled => {}
    }
}

pub(crate) fn retry_main_narration<B: ActionPipelineBackend>(
    backend: &B,
    ctx: &GameServiceContext,
    state: GameState,
    input_text: String,
) {
    let pipeline = ActionPipeline::new(backend, ctx);
    match pipeline.run_from_input(state, input_text) {
        ActionOutcome::Completed => {}
        ActionOutcome::Error { message } => {
            save_retry_error(ctx, message);
        }
        ActionOutcome::Cancelled => {}
    }
}
