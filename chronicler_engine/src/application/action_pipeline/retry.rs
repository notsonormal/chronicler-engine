//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Retry logic for action pipeline operations

use tracing::instrument;
use crate::application::action_pipeline::phase_error::PhaseError;
use crate::application::action_pipeline::phases::PipelineRun;
use crate::application::action_pipeline::pipeline::ActionPipeline;
use crate::application::application_service::DefaultApplicationService;
use crate::adapters::driven::storage::worlds::WorldBundle;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;

fn retry_persist_error(app: &DefaultApplicationService, message: impl Into<String>) {
    let mut state = app.load_or_fresh();
    state.narrative.input_buffer.status = GenerationStatus::Error(message.into());
    if let Err(e) = app.save_state(&state) {
        tracing::error!("Critical: failed to persist retry error state: {e}");
    }
}

fn handle_retry_outcome(app: &DefaultApplicationService, outcome: Result<(), PhaseError>) {
    match outcome {
        Err(PhaseError::Cancelled) => {
            let mut state = app.load_or_fresh();
            state.narrative.input_buffer.status = GenerationStatus::Idle;
            state.narrative.input_buffer.phase = GenerationPhase::default();
            let _ = app.save_state(&state);
        }
        Err(e) => {
            let pipeline = app.game_service().pipeline();
            let started_for = app.current_game_id();
            let run = PipelineRun::new(&pipeline, app, started_for);
            ActionPipeline::finalize_phase_error(&run, e);
        }
        Ok(()) => {}
    }
}

#[instrument(skip(app))]
pub fn retry_last_response_impl(app: &DefaultApplicationService) {
    let messages = match app.load_messages() {
        Ok(m) => m,
        Err(e) => {
            retry_persist_error(app, format!("Retry failed: {e}"));
            return;
        }
    };

    let Some((anchor_idx, _anchor_msg, snapshot_id)) = app.find_retry_anchor(&messages) else {
        retry_persist_error(app, "Retry failed: no anchor message");
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
            retry_persist_error(
                app,
                format!("Retry failed: no snapshot found for id {snapshot_id}"),
            );
            return;
        }
        Err(e) => {
            retry_persist_error(app, format!("Retry failed: {e}"));
            return;
        }
    };

    let mut state = GameState::from_snapshot(&snapshot);
    let mut truncated = messages;
    truncated.truncate(anchor_idx + 1);
    state.narrative.history.replace(truncated);
    state.narrative.retry_target = old_target;

    let input_text = match state.narrative.history.last_input_text() {
        Some((_, text)) => text,
        None => {
            retry_persist_error(app, "Retry failed: no input to retry");
            return;
        }
    };

    let outcome = if is_event {
        retry_event_continuation(app, state)
    } else {
        retry_main_narration(app, state, input_text)
    };

    handle_retry_outcome(app, outcome);
}

pub(crate) fn retry_event_continuation(
    app: &DefaultApplicationService,
    state: GameState,
) -> Result<(), PhaseError> {
    let Some(trigger) = state.narrative.last_trigger.clone() else {
        return Err(PhaseError::TriggerMissing);
    };
    let input_text = match state.narrative.history.last_input_text() {
        Some((_, text)) => text,
        None => String::new(),
    };
    let WorldBundle {
        map,
        persona,
        npcs: npcs_map,
        ..
    } = match ActionPipeline::load_world_bundle(app, app.current_game_id()) {
        Ok(b) => b,
        Err(e) => return Err(PhaseError::FetchFailed(e.to_string())),
    };
    let pipeline = app.game_service().pipeline();
    let (mut state, continuation_text) =
        pipeline.phase_trigger_continuation(state, &trigger, app, &map, &npcs_map)?;
    if !continuation_text.is_empty() {
        let started_for = app.current_game_id();
        let run = PipelineRun::new(&pipeline, app, started_for);
        state = run.reconcile_post_trigger_npcs(
            state,
            &input_text,
            &continuation_text,
            &map,
            &persona,
            &npcs_map,
        );
    }
    if let Some(target) = state.narrative.retry_target.take() {
        state.narrative.history.append(target);
    }
    {
        let started_for = app.current_game_id();
        let run = PipelineRun::new(&pipeline, app, started_for);
        run.phase_finalize(&mut state);
    }
    Ok(())
}

pub(crate) fn retry_main_narration(
    app: &DefaultApplicationService,
    state: GameState,
    input_text: String,
) -> Result<(), PhaseError> {
    let pipeline = app.game_service().pipeline();
    pipeline.run_from_input(app, state, input_text)
}

#[instrument(skip(app))]
pub fn retrigger_event_impl(app: &DefaultApplicationService) {
    let state = app.load_or_fresh();
    let outcome = retry_event_continuation(app, state);
    handle_retry_outcome(app, outcome);
}
