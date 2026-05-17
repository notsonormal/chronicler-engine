use std::sync::Arc;

use crate::engine::game_service::actions::{
    execute_freeaction_pipeline, finish_action, reconcile_post_trigger_npcs,
};
use crate::model::state::{GameState, GenerationPhase, GenerationStatus, LogType};

use super::context::GameServiceContext;
use super::helpers::{load_state, save_state};
use super::service::DefaultGameService;

/// [DOC: docs/architecture/system.md]
pub fn retry_last_response_impl(service: &DefaultGameService, ctx: GameServiceContext) {
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
    let snapshot_id = match anchor_msg.snapshot_id {
        Some(id) => id,
        None => {
            log::error!("Anchor message has no snapshot_id");
            save_retry_error(&ctx, "Retry failed: missing snapshot_id");
            return;
        }
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
    state.narrative.messages = messages;

    let input_text = match state.get_last_input_text() {
        Some((_sender, text)) => text,
        None => {
            log::error!("No input to retry");
            return;
        }
    };

    if is_event {
        retry_event_continuation(service, &ctx, state);
    } else {
        retry_main_narration(service, &ctx, state, input_text);
    }
}

pub(crate) fn save_retry_error(ctx: &GameServiceContext, message: impl Into<String>) {
    let mut state = load_state(ctx);
    state.narrative.generation.status = GenerationStatus::Error(message.into());
    if let Err(e) = save_state(ctx, &mut state) {
        log::error!("Critical: failed to persist retry error state: {e}");
    }
}

pub(crate) fn retry_event_continuation(
    service: &DefaultGameService,
    ctx: &GameServiceContext,
    mut state: GameState,
) {
    let trigger = match state.narrative.last_trigger.clone() {
        Some(t) => t,
        None => {
            log::error!("Missing trigger context for event retry");
            save_retry_error(ctx, "Retry failed: missing trigger context");
            return;
        }
    };

    state.narrative.generation.status = GenerationStatus::Generating;
    state.narrative.generation.phase = GenerationPhase::GeneratingEvent;

    let backend = Arc::clone(&service.llm_backend);
    let continuation_result = match backend.complete(
        crate::narrative::llm::backend::AGENT_TRIGGER,
        &trigger.system_prompt,
        &trigger.user_prompt,
        trigger.max_tokens,
    ) {
        Ok(result) => result,
        Err(e) => {
            log::error!("Trigger narration retry failed: {e}");
            save_retry_error(ctx, format!("Retry failed: {e}"));
            return;
        }
    };
    let continuation_text = continuation_result.text;

    if continuation_text.trim().is_empty() {
        save_retry_error(ctx, "LLM Error: empty response");
        return;
    }

    let request = crate::engine::action_processing::TriggerContinuationRequest { stored: trigger };

    let mut committed_state = match crate::engine::action_processing::commit_trigger_narration(
        state,
        &request,
        &continuation_text,
    ) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Trigger commit failed on retry: {e}");
            save_retry_error(ctx, format!("Trigger error: {e}"));
            return;
        }
    };

    let input_text = match committed_state.get_last_input_text() {
        Some((_sender, text)) => text,
        None => String::new(),
    };

    match reconcile_post_trigger_npcs(
        service,
        committed_state.clone(),
        &input_text,
        &continuation_text,
    ) {
        Ok(updated) => committed_state = updated,
        Err(e) => {
            log::error!("Failed to apply post-trigger NPC events on retry: {e}");
            committed_state.narrative.generation.status =
                GenerationStatus::Error(format!("NPC event error: {e}"));
            if let Err(e) = save_state(ctx, &mut committed_state) {
                log::error!("Critical: failed to persist retry NPC error state: {e}");
            }
            return;
        }
    }

    if let Err(e) = finish_action(ctx, committed_state) {
        log::error!("Failed to persist finished retry action: {e}");
    }
}

pub(crate) fn retry_main_narration(
    service: &DefaultGameService,
    ctx: &GameServiceContext,
    state: GameState,
    input_text: String,
) {
    execute_freeaction_pipeline(service, ctx, state, input_text);
}
