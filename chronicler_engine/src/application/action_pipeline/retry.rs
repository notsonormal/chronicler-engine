//! [DOC: docs/system/game_flow.md]
//! Retry logic for action pipeline operations

use std::sync::Arc;

use tracing::instrument;
use crate::application::action_pipeline::pipeline::ActionOutcome;
use crate::application::context::{GameServiceContext, load_or_fresh, save_state};
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::application::game_service::GameService;

#[instrument(skip(service, ctx))]
pub fn retry_last_response_impl(service: &GameService, ctx: GameServiceContext) {
    let messages = match ctx.load_messages() {
        Ok(msgs) => msgs,
        Err(e) => {
            tracing::error!("Failed to load messages: {e}");
            return;
        }
    };

    let Some((anchor_idx, _anchor_msg, snapshot_id)) = ctx.find_retry_anchor(&messages) else {
        tracing::error!("No anchor message found for retry");
        save_retry_error(&ctx, "Retry failed: no anchor message");
        return;
    };

    let is_event = messages
        .last()
        .map(|m| m.event_header().is_some())
        .unwrap_or(false);

    let old_target = messages
        .iter()
        .rev()
        .find(|m| {
            if is_event {
                m.event_header().is_some()
            } else {
                matches!(
                    m.message_type,
                    MessageType::Narration | MessageType::Dialogue
                ) && m.event_header().is_none()
            }
        })
        .cloned();

    let snapshot = match ctx.storage.load_snapshot_by_id(snapshot_id) {
        Ok(Some(s)) => s,
        Ok(None) => {
            tracing::error!("No snapshot found for id {snapshot_id}");
            save_retry_error(
                &ctx,
                format!("Retry failed: no snapshot found for id {snapshot_id}"),
            );
            return;
        }
        Err(e) => {
            tracing::error!("Failed to load snapshot: {e}");
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
    state.narrative.retry_target = old_target;

    let input_text = match state.narrative.history.last_input_text() {
        Some((_sender, text)) => text,
        None => {
            tracing::error!("No input to retry");
            return;
        }
    };

    let outcome = if is_event {
        retry_event_continuation(service, &ctx, state)
    } else {
        retry_main_narration(service, &ctx, state, input_text)
    };

    if let ActionOutcome::Cancelled = outcome {
        let mut state = load_or_fresh(&ctx);
        state.narrative.input_buffer.status = GenerationStatus::Idle;
        state.narrative.input_buffer.phase = GenerationPhase::default();
        let _ = save_state(&ctx, &state);
    }
}

pub(crate) fn save_retry_error(ctx: &GameServiceContext, message: impl Into<String>) {
    let mut state = load_or_fresh(ctx);
    state.narrative.input_buffer.status = GenerationStatus::Error(message.into());
    if let Err(e) = save_state(ctx, &state) {
        tracing::error!("Critical: failed to persist retry error state: {e}");
    }
}

pub(crate) fn retry_event_continuation(
    service: &GameService,
    ctx: &GameServiceContext,
    state: GameState,
) -> ActionOutcome {
    let Some(trigger) = state.narrative.last_trigger.clone() else {
        tracing::error!("Missing trigger context for event retry");
        save_retry_error(ctx, "Retry failed: missing trigger context");
        return ActionOutcome::Completed;
    };
    let input_text = match state.narrative.history.last_input_text() {
        Some((_sender, text)) => text,
        None => String::new(),
    };
    let pipeline = service.pipeline();
    let mut state = match pipeline.phase_trigger_continuation(state, &trigger, ctx) {
        Ok((s, continuation_text)) => {
            if !continuation_text.is_empty() {
                let run =
                    crate::application::action_pipeline::phases::PipelineRun::new(&pipeline, ctx);
                run.reconcile_post_trigger_npcs(s, &input_text, &continuation_text)
            } else {
                s
            }
        }
        Err(outcome) => return outcome,
    };
    if let Some(target) = state.narrative.retry_target.take() {
        state.narrative.history.append(target);
    }
    {
        let run = crate::application::action_pipeline::phases::PipelineRun::new(&pipeline, ctx);
        run.phase_finalize(&mut state);
    }
    ActionOutcome::Completed
}

pub(crate) fn retry_main_narration(
    service: &GameService,
    ctx: &GameServiceContext,
    state: GameState,
    input_text: String,
) -> ActionOutcome {
    let pipeline = service.pipeline();
    ActionOutcome::from_pipeline_result(pipeline.run_from_input(ctx, state, input_text))
}

#[instrument(skip(service, ctx))]
pub fn retrigger_event_impl(service: &GameService, ctx: &GameServiceContext) {
    let state = load_or_fresh(ctx);
    let outcome = retry_event_continuation(service, ctx, state);
    if let ActionOutcome::Cancelled = outcome {
        let mut state = load_or_fresh(ctx);
        state.narrative.input_buffer.status = GenerationStatus::Idle;
        state.narrative.input_buffer.phase = GenerationPhase::default();
        let _ = save_state(ctx, &state);
    }
}
