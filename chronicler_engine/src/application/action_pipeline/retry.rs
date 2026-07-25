//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Retry logic for action pipeline operations

use crate::application::action_pipeline::phase_error::PhaseError;
use crate::application::action_pipeline::phases::PipelineRun;
use crate::application::action_pipeline::pipeline::ActionPipeline;
use crate::application::application_service::DefaultApplicationService;
use crate::adapters::driven::storage::worlds::WorldBundle;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};

pub(crate) fn retry_persist_error(app: &DefaultApplicationService, message: impl Into<String>) {
    let mut state = app.load_or_fresh();
    state.narrative.input_buffer.status = GenerationStatus::Error(message.into());
    if let Err(e) = app.save_state(&state) {
        tracing::error!("Critical: failed to persist retry error state: {e}");
    }
}

pub(crate) fn handle_retry_outcome(
    app: &DefaultApplicationService,
    outcome: Result<(), PhaseError>,
) {
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
