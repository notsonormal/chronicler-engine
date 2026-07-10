//! [DOC: docs/system/game_flow.md]
//! GenerationGate — owns `CancellationToken` + `is_generating: Arc<AtomicBool>`
//! (ADR-030 hot-path cache) + slot-orchestration. (T2 ticket 03.)

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::application::application_service::DefaultApplicationService;
use crate::application::application_service::ProcessActionResult;
use crate::application::generation_guard::GenerationGuard;
use crate::application::spawn_pipeline_task;
use crate::application::action_pipeline::execute_action_impl;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::message_types::MessageType;
use crate::error::EngineError;

#[derive(Clone)]
pub struct GenerationGate {
    cancel_token: CancellationToken,
    is_generating: Arc<AtomicBool>,
}

impl GenerationGate {
    pub fn new(cancel_token: CancellationToken, is_generating: Arc<AtomicBool>) -> Self {
        Self {
            cancel_token,
            is_generating,
        }
    }

    pub fn cancel_token(&self) -> &CancellationToken {
        &self.cancel_token
    }

    pub fn is_generating(&self) -> &Arc<AtomicBool> {
        &self.is_generating
    }

    pub fn start_action(
        &self,
        app: &DefaultApplicationService,
        input: String,
    ) -> Result<ProcessActionResult, EngineError> {
        let mut game_state = app.load_or_fresh();

        self.heal_stale_generating(app, &mut game_state);

        let player_name = game_state.persona.sheet.name.clone();
        if !input.is_empty() {
            game_state.add_message(input.clone(), Some(player_name.clone()), MessageType::Input);
        }

        match self.claim_generation_slot(app, &mut game_state)? {
            ProcessActionResult::ConcurrentGeneration => {
                return Ok(ProcessActionResult::ConcurrentGeneration);
            }
            ProcessActionResult::Started => {}
            ProcessActionResult::ShuttingDown => {
                return Ok(ProcessActionResult::ShuttingDown);
            }
        }

        if self.cancel_token.is_cancelled() {
            let mut gs = app.load_or_fresh();
            gs.narrative.input_buffer.status = GenerationStatus::Idle;
            if let Err(e) = app.persistence_gate.save_state(&gs) {
                tracing::error!("Failed to save shutdown snapshot: {e}");
            }
            self.release_generation_slot();
            return Ok(ProcessActionResult::ShuttingDown);
        }

        let is_generating = Arc::clone(&self.is_generating);
        spawn_pipeline_task(Arc::new(app.clone()), move |app| {
            tracing::debug!("spawn_blocking: task started");
            let _guard = GenerationGuard(Arc::clone(&is_generating));
            if app.cancel_token().is_cancelled() {
                tracing::debug!("spawn_blocking: cancelled before execute_action");
                return;
            }
            execute_action_impl(app, input);
            tracing::debug!("spawn_blocking: execute_action completed");
        });
        Ok(ProcessActionResult::Started)
    }

    pub fn heal_stale_generating(&self, app: &DefaultApplicationService, state: &mut GameState) {
        let _ = app;
        if !self.is_generating.load(Ordering::SeqCst)
            && state.narrative.input_buffer.status.is_generating()
        {
            tracing::warn!(
                "Found stale Generating status without active generation, resetting to Idle"
            );
            state.narrative.input_buffer.status = GenerationStatus::Idle;
            state.narrative.input_buffer.phase = GenerationPhase::default();
        }
    }

    pub fn claim_generation_slot(
        &self,
        app: &DefaultApplicationService,
        state: &mut GameState,
    ) -> Result<ProcessActionResult, EngineError> {
        if self
            .is_generating
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Ok(ProcessActionResult::ConcurrentGeneration);
        }

        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::Narrating;

        if let Err(e) = app.save_message_and_snapshot(state) {
            tracing::debug!("claim_generation_slot: save failed; releasing slot");
            self.release_generation_slot();
            return Err(e);
        }
        tracing::debug!("process_action: state saved, spawning blocking task");
        Ok(ProcessActionResult::Started)
    }

    pub fn release_generation_slot(&self) {
        self.is_generating.store(false, Ordering::SeqCst);
    }
}
