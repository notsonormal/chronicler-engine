//! [DOC: docs/system/game_flow.md]
//! Retry logic for action pipeline operations

use std::collections::HashMap;
use std::sync::Arc;

use tracing::instrument;
use crate::application::action_pipeline::pipeline::ActionOutcome;
use crate::application::application_service::DefaultApplicationService;
use crate::domain::model::character::NpcCard;
use crate::domain::model::map::MapDef;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;

#[instrument(skip(app))]
pub fn retry_last_response_impl(app: &DefaultApplicationService) {
    let messages = match app.load_messages() {
        Ok(msgs) => msgs,
        Err(e) => {
            tracing::error!("Failed to load messages: {e}");
            return;
        }
    };

    let Some((anchor_idx, _anchor_msg, snapshot_id)) = app.find_retry_anchor(&messages) else {
        tracing::error!("No anchor message found for retry");
        save_retry_error(app, "Retry failed: no anchor message");
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

    let snapshot = match app.storage().load_snapshot_by_id(snapshot_id) {
        Ok(Some(s)) => s,
        Ok(None) => {
            tracing::error!("No snapshot found for id {snapshot_id}");
            save_retry_error(
                app,
                format!("Retry failed: no snapshot found for id {snapshot_id}"),
            );
            return;
        }
        Err(e) => {
            tracing::error!("Failed to load snapshot: {e}");
            save_retry_error(app, format!("Retry failed: {e}"));
            return;
        }
    };

    let mut state = GameState::from_snapshot(&snapshot);

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
        retry_event_continuation(app, state)
    } else {
        retry_main_narration(app, state, input_text)
    };

    if let ActionOutcome::Cancelled = outcome {
        let mut state = app.load_or_fresh();
        state.narrative.input_buffer.status = GenerationStatus::Idle;
        state.narrative.input_buffer.phase = GenerationPhase::default();
        let _ = app.save_state(&state);
    }
}

pub(crate) fn save_retry_error(app: &DefaultApplicationService, message: impl Into<String>) {
    let mut state = app.load_or_fresh();
    state.narrative.input_buffer.status = GenerationStatus::Error(message.into());
    if let Err(e) = app.save_state(&state) {
        tracing::error!("Critical: failed to persist retry error state: {e}");
    }
}

/// Side effect: persists error state on fetch failure (callers early-return `ActionOutcome::Completed`).
pub(crate) fn fetch_world_bundle_for_retry(
    app: &DefaultApplicationService,
) -> Option<(Arc<MapDef>, HashMap<String, NpcCard>)> {
    let game_id = app.storage().current_game_id();
    let game = match app.storage().get_game(game_id) {
        Ok(Some(g)) => g,
        Ok(None) => {
            tracing::error!("No game found for current_game_id {game_id}");
            save_retry_error(app, format!("Retry failed: no game for id {game_id}"));
            return None;
        }
        Err(e) => {
            tracing::error!("Failed to load game: {e}");
            save_retry_error(app, format!("Retry failed: {e}"));
            return None;
        }
    };
    let world_with_map = match app.storage().get_world(&game.world_key) {
        Ok(Some(w)) => w,
        Ok(None) => {
            tracing::error!("World '{}' not found", game.world_key);
            save_retry_error(
                app,
                format!("Retry failed: world '{}' not found", game.world_key),
            );
            return None;
        }
        Err(e) => {
            tracing::error!("Failed to load world '{}': {e}", game.world_key);
            save_retry_error(app, format!("Retry failed: {e}"));
            return None;
        }
    };
    let chars = match app.storage().list_characters(world_with_map.world_id) {
        Ok(list) => list,
        Err(e) => {
            tracing::error!("Failed to list characters: {e}");
            save_retry_error(app, format!("Retry failed: {e}"));
            return None;
        }
    };
    let npcs_map: HashMap<String, NpcCard> = chars.into_iter().map(|n| (n.id.clone(), n)).collect();
    let map: Arc<MapDef> = Arc::new(world_with_map.map);
    Some((map, npcs_map))
}

pub(crate) fn retry_event_continuation(
    app: &DefaultApplicationService,
    state: GameState,
) -> ActionOutcome {
    let Some(trigger) = state.narrative.last_trigger.clone() else {
        tracing::error!("Missing trigger context for event retry");
        save_retry_error(app, "Retry failed: missing trigger context");
        return ActionOutcome::Completed;
    };
    let input_text = match state.narrative.history.last_input_text() {
        Some((_sender, text)) => text,
        None => String::new(),
    };
    let (map, npcs_map) = match fetch_world_bundle_for_retry(app) {
        Some(bundle) => bundle,
        None => return ActionOutcome::Completed,
    };
    let persona: Arc<crate::domain::model::character::PersonaCard> = {
        let game_id = app.current_game_id();
        let game = match app.storage().get_game(game_id) {
            Ok(Some(g)) => g,
            Ok(None) => {
                tracing::error!("No game found for current_game_id {game_id}");
                save_retry_error(app, format!("Retry failed: no game for id {game_id}"));
                return ActionOutcome::Completed;
            }
            Err(e) => {
                tracing::error!("Failed to load game: {e}");
                save_retry_error(app, format!("Retry failed: {e}"));
                return ActionOutcome::Completed;
            }
        };
        match app.storage().get_persona(&game.persona_key) {
            Ok(Some(p)) => Arc::new(p),
            Ok(None) => {
                tracing::error!("Persona '{}' not found", game.persona_key);
                save_retry_error(
                    app,
                    format!("Retry failed: persona '{}' not found", game.persona_key),
                );
                return ActionOutcome::Completed;
            }
            Err(e) => {
                tracing::error!("Failed to load persona '{}': {e}", game.persona_key);
                save_retry_error(app, format!("Retry failed: {e}"));
                return ActionOutcome::Completed;
            }
        }
    };
    let pipeline = app.game_service().pipeline();
    let mut state = match pipeline.phase_trigger_continuation(state, &trigger, app, &map, &npcs_map)
    {
        Ok((s, continuation_text)) => {
            if !continuation_text.is_empty() {
                let started_for = app.current_game_id();
                let run = crate::application::action_pipeline::phases::PipelineRun::new(
                    &pipeline,
                    app,
                    started_for,
                );
                run.reconcile_post_trigger_npcs(
                    s,
                    &input_text,
                    &continuation_text,
                    &map,
                    &persona,
                    &npcs_map,
                )
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
        let started_for = app.current_game_id();
        let run = crate::application::action_pipeline::phases::PipelineRun::new(
            &pipeline,
            app,
            started_for,
        );
        run.phase_finalize(&mut state);
    }
    ActionOutcome::Completed
}

pub(crate) fn retry_main_narration(
    app: &DefaultApplicationService,
    state: GameState,
    input_text: String,
) -> ActionOutcome {
    let pipeline = app.game_service().pipeline();
    ActionOutcome::from_pipeline_result(pipeline.run_from_input(app, state, input_text))
}

#[instrument(skip(app))]
pub fn retrigger_event_impl(app: &DefaultApplicationService) {
    let state = app.load_or_fresh();
    let outcome = retry_event_continuation(app, state);
    if let ActionOutcome::Cancelled = outcome {
        let mut state = app.load_or_fresh();
        state.narrative.input_buffer.status = GenerationStatus::Idle;
        state.narrative.input_buffer.phase = GenerationPhase::default();
        let _ = app.save_state(&state);
    }
}
