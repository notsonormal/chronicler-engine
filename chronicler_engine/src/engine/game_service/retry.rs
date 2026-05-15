use std::sync::Arc;

use crate::engine::game_service::actions::{
    execute_freeaction_pipeline, finish_action, reconcile_post_trigger_npcs,
};
use crate::model::state::{GameState, GenerationPhase, GenerationStatus};

use super::context::GameServiceContext;
use super::helpers::{load_state, save_state};
use super::service::DefaultGameService;

/// [DOC: docs/architecture/system.md]
pub fn retry_last_response_impl(service: &DefaultGameService, ctx: GameServiceContext) {
    let snapshot = match ctx.snapshot_storage.load_latest(None) {
        Ok(Some(s)) => s,
        _ => {
            log::error!("No snapshot to retry");
            return;
        }
    };

    let guard = GameState::from_snapshot(
        &snapshot,
        Arc::clone(&ctx.world),
        Arc::clone(&ctx.map),
        Arc::clone(&ctx.player),
        (*ctx.npcs).clone(),
    );

    let input_text = match guard.get_last_input_text() {
        Some((_sender, text)) => text,
        None => {
            log::error!("No input to retry");
            return;
        }
    };

    let turn_uuid = snapshot.turn_id.clone();
    let current_swipe = snapshot.swipe_index;
    let is_event = guard.is_last_ai_response_event_continuation();

    if is_event {
        retry_event_continuation(service, &ctx, &turn_uuid, current_swipe, &guard);
    } else {
        retry_main_narration(service, &ctx, &turn_uuid, current_swipe, input_text);
    }
}

fn save_retry_error(
    ctx: &GameServiceContext,
    turn_uuid: &str,
    swipe: u32,
    message: impl Into<String>,
) {
    let mut state = load_state(ctx);
    state.narrative.generation.status = GenerationStatus::Error(message.into());
    save_state(ctx, &state, turn_uuid.to_string(), swipe);
}

fn retry_event_continuation(
    service: &DefaultGameService,
    ctx: &GameServiceContext,
    turn_uuid: &str,
    current_swipe: u32,
    latest_state: &GameState,
) {
    let pre_event_id = format!("pre-event:{turn_uuid}");
    let pre_event_snapshot = match ctx.snapshot_storage.load_by_turn(&pre_event_id, 0) {
        Ok(Some(s)) => s,
        Ok(None) => {
            log::warn!(
                "No pre-event snapshot for {turn_uuid}; falling back to main narration retry"
            );
            let input_text = latest_state.get_last_input_text().map(|(_, t)| t);
            match input_text {
                Some(text) => retry_main_narration(service, ctx, turn_uuid, current_swipe, text),
                None => log::error!("No input to retry"),
            }
            return;
        }
        Err(e) => {
            log::error!("Failed to load pre-event snapshot: {e}");
            save_retry_error(
                ctx,
                turn_uuid,
                current_swipe + 1,
                format!("Retry failed: {e}"),
            );
            return;
        }
    };

    let mut pre_event_state = GameState::from_snapshot(
        &pre_event_snapshot,
        Arc::clone(&ctx.world),
        Arc::clone(&ctx.map),
        Arc::clone(&ctx.player),
        (*ctx.npcs).clone(),
    );

    let trigger = match pre_event_state.narrative.last_trigger.clone() {
        Some(t) => t,
        None => {
            log::error!("Pre-event snapshot missing trigger context");
            save_retry_error(
                ctx,
                turn_uuid,
                current_swipe + 1,
                "Retry failed: missing trigger context",
            );
            return;
        }
    };

    pre_event_state.narrative.generation.status = GenerationStatus::Generating;
    pre_event_state.narrative.generation.phase = GenerationPhase::GeneratingEvent;
    save_state(
        ctx,
        &pre_event_state,
        turn_uuid.to_string(),
        current_swipe + 1,
    );

    let backend = Arc::clone(&service.llm_backend);
    let continuation_result = match backend.narrate_action_from_prompt(
        crate::narrative::llm::backend::AGENT_TRIGGER,
        &trigger.system_prompt,
        &trigger.user_prompt,
        trigger.max_tokens,
    ) {
        Ok(result) => result,
        Err(e) => {
            log::error!("Trigger narration retry failed: {e}");
            save_retry_error(
                ctx,
                turn_uuid,
                current_swipe + 1,
                format!("Retry failed: {e}"),
            );
            return;
        }
    };
    let continuation_text = continuation_result.text;

    if continuation_text.trim().is_empty() {
        save_retry_error(
            ctx,
            turn_uuid,
            current_swipe + 1,
            "LLM Error: empty response",
        );
        return;
    }

    let request = crate::engine::action_processing::TriggerContinuationRequest { stored: trigger };

    let mut committed_state = match crate::engine::action_processing::commit_trigger_narration(
        pre_event_state,
        &request,
        &continuation_text,
    ) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Trigger commit failed on retry: {e}");
            save_retry_error(
                ctx,
                turn_uuid,
                current_swipe + 1,
                format!("Trigger error: {e}"),
            );
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
            save_state(
                ctx,
                &committed_state,
                turn_uuid.to_string(),
                current_swipe + 1,
            );
            return;
        }
    }

    finish_action(
        ctx,
        committed_state,
        turn_uuid.to_string(),
        current_swipe + 1,
    );
}

fn retry_main_narration(
    service: &DefaultGameService,
    ctx: &GameServiceContext,
    turn_uuid: &str,
    current_swipe: u32,
    input_text: String,
) {
    let pre_main_id = format!("pre-main:{turn_uuid}");
    let pre_main_snapshot = match ctx.snapshot_storage.load_by_turn(&pre_main_id, 0) {
        Ok(Some(s)) => s,
        Ok(None) => {
            log::error!("No pre-main snapshot for {turn_uuid}");
            save_retry_error(
                ctx,
                turn_uuid,
                current_swipe + 1,
                "Retry failed: no pre-generation snapshot found",
            );
            return;
        }
        Err(e) => {
            log::error!("Failed to load pre-main snapshot: {e}");
            save_retry_error(
                ctx,
                turn_uuid,
                current_swipe + 1,
                format!("Retry failed: {e}"),
            );
            return;
        }
    };

    let state = GameState::from_snapshot(
        &pre_main_snapshot,
        Arc::clone(&ctx.world),
        Arc::clone(&ctx.map),
        Arc::clone(&ctx.player),
        (*ctx.npcs).clone(),
    );

    execute_freeaction_pipeline(
        service,
        ctx,
        state,
        turn_uuid.to_string(),
        input_text,
        current_swipe + 1,
    );
}
