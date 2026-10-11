//! [DOC: docs/diataxis/reference/game_flow.md]
//! Retry entry path for the pipeline.

use tracing::instrument;

use crate::application::errors::{ApplicationError, ProcessActionResult};
use crate::application::generation::gate::GenerationGate;
use crate::application::pipeline::PhaseError;
use crate::application::pipeline::pipeline_run::PipelineRun;
use super::core::ActionPipeline;
use crate::domain::model::message::Message;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::state::generation_status::{
    GenerationFailure, GenerationFailureKind, GenerationPhase, GenerationStatus,
};
use crate::domain::model::state::message_types::MessageType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetryMode {
    /// Retry the last AI narration using the original input.
    ReNarrate,
    /// Retry an impersonated input by re-running the impersonate preset.
    ReImpersonate,
    /// Regenerate a plain user input as a new swipe.
    UserRegen,
}

pub(crate) struct RetryTarget {
    pub(crate) anchor_idx: usize,
    pub(crate) snapshot_id: u64,
    pub(crate) is_event: bool,
    pub(crate) mode: RetryMode,
    pub(crate) old_target: Option<Message>,
}

impl ActionPipeline {
    pub fn retry(
        &self,
        generation_gate: &GenerationGate,
    ) -> Result<ProcessActionResult, ApplicationError> {
        self.claim_and_spawn(
            generation_gate,
            |_game_id, game_state| {
                if !game_state.narrative.history.last_turn_is_retryable() {
                    return Err(ApplicationError::validation("No input to retry"));
                }
                Ok(())
            },
            || self.check_retry_anchor(),
            |pipeline| {
                pipeline.retry_last_response();
            },
        )
    }

    #[instrument(skip(self))]
    pub fn retry_last_response(&self) {
        let retry_game_id = self.storage.current_game_id();
        let messages = match self.message_service.load_messages() {
            Ok(m) => m,
            Err(e) => {
                self.persist_generation_error(
                    GenerationFailure::from_engine_error(&e).with_context("Retry failed"),
                );
                return;
            }
        };

        let Some(target) = self.resolve_retry_target(&messages) else {
            self.persist_generation_error(GenerationFailure::new(
                GenerationFailureKind::Other,
                "Retry failed: no anchor message",
            ));
            return;
        };

        let state_at_click = match self.storage.load_latest_snapshot() {
            Ok(snapshot) => snapshot,
            Err(e) => {
                self.persist_generation_error(
                    GenerationFailure::from_engine_error(&e).with_context("Retry failed"),
                );
                return;
            }
        };

        // An event continuation is regenerated on the state its own Message was
        // committed onto, not the latest snapshot: that Swipe's Snapshot holds the
        // trigger context, the post-movement room and the scene before the
        // post-trigger reconciliation.
        let restore_snapshot_id = if target.is_event {
            let Some(snapshot_id) = target.old_target.as_ref().and_then(|m| m.snapshot_id()) else {
                self.persist_generation_error(GenerationFailure::new(
                    GenerationFailureKind::Other,
                    "Retry failed: event message has no snapshot",
                ));
                return;
            };
            snapshot_id
        } else {
            target.snapshot_id
        };

        let snapshot = match self.storage.load_snapshot_by_id(restore_snapshot_id) {
            Ok(Some(s)) => s,
            Ok(None) => {
                self.persist_generation_error(GenerationFailure::new(
                    GenerationFailureKind::Other,
                    format!("Retry failed: no snapshot found for id {restore_snapshot_id}"),
                ));
                return;
            }
            Err(e) => {
                self.persist_generation_error(
                    GenerationFailure::from_engine_error(&e).with_context("Retry failed"),
                );
                return;
            }
        };

        let mut state = Self::reconstruct_retry_state(snapshot, messages, &target);

        match target.mode {
            RetryMode::ReNarrate => {
                // A guided turn carries its steering on the Swipe's stored inputs,
                // so a redo reuses the empty input the original generation used.
                let target_is_guided = target.old_target.as_ref().is_some_and(|m| m.is_guided());
                let input_text = if target_is_guided {
                    String::new()
                } else {
                    match state.narrative.history.last_input_text() {
                        Some(text) => text,
                        None => {
                            self.persist_generation_error(GenerationFailure::new(
                                GenerationFailureKind::Other,
                                "Retry failed: no input to retry",
                            ));
                            return;
                        }
                    }
                };
                if target.is_event {
                    let outcome = self.retry_event_continuation(&mut state);
                    self.log_cancellation(outcome);
                } else {
                    let outcome = self.retry_main_narration(state, input_text);
                    self.log_cancellation(outcome);
                }
            }
            RetryMode::ReImpersonate => {
                // Unlike user regen, an impersonate redo runs the full tail
                // (quantifier → engine commit → trigger).
                let outcome = self.retry_main_narration(state, String::new());
                self.log_cancellation(outcome);
            }
            RetryMode::UserRegen => {
                let outcome = self.retry_user_regen(state);
                self.log_cancellation(outcome);
            }
        }

        self.restore_click_time_state_after_failed_redo(retry_game_id, state_at_click, &target);
    }

    /// A redo replays from its anchor's snapshot, so it persists that older
    /// world while it runs and only its finalize makes the new world durable.
    ///
    /// A failure that never got as far as persisting a Swipe for the target
    /// would leave that older world behind the story: put the state the click
    /// found back and keep the failure on screen. A failure that did persist one has
    /// a narration to match, so its world is the one to keep.
    ///
    /// A persist that fails at or after the post-engine Snapshot leaves the
    /// movement on record, and one at or after the trigger commit leaves the
    /// trigger mark on record too: the failing write saves the state it holds,
    /// and `finalize_phase_error` saves it once more.
    ///
    /// Between the pre-quantifier narration persist, which saves the new Swipe,
    /// and the post-engine Snapshot, a failure — an engine-commit error, or a
    /// write that fails once and then recovers — keeps the rewound world that
    /// Swipe was written from.
    ///
    /// That world is kept on purpose: the case is too rare to special-case.
    fn restore_click_time_state_after_failed_redo(
        &self,
        retry_game_id: u64,
        state_at_click: Option<GameStateSnapshot>,
        target: &RetryTarget,
    ) {
        let Some(state_at_click) = state_at_click else {
            return;
        };
        if self.storage.current_game_id() != retry_game_id {
            tracing::warn!("Retry: the game changed while the redo ran; leaving it alone");
            return;
        }
        let attempt = match self.storage.load_latest_snapshot() {
            Ok(Some(attempt)) => attempt,
            Ok(None) => return,
            Err(e) => {
                tracing::error!("Retry: cannot read the state a failed redo left: {e}");
                return;
            }
        };
        let Some(failure) = attempt.narrative.input_buffer.status.failure() else {
            return;
        };
        match self.redo_persisted_a_swipe(target) {
            Some(true) | None => return,
            Some(false) => {}
        }
        let mut state = GameState::from_snapshot(&state_at_click);
        state.narrative.input_buffer.status = GenerationStatus::Error(failure.clone());
        state.narrative.input_buffer.phase = attempt.narrative.input_buffer.phase;
        if let Err(e) = self.message_service.save_state(&state) {
            tracing::error!("Failed to put the game state back after a failed retry: {e}");
        }
    }

    /// The Swipe it finds is the narration on screen, so the world it was written
    /// from is the one to keep. `None` means the Messages cannot be read, which
    /// leaves the restore unsafe.
    fn redo_persisted_a_swipe(&self, target: &RetryTarget) -> Option<bool> {
        let target = target.old_target.as_ref()?;
        let messages = match self.message_service.load_messages() {
            Ok(messages) => messages,
            Err(e) => {
                tracing::error!("Retry: cannot read the Messages a failed redo left: {e}");
                return None;
            }
        };
        Some(
            messages
                .iter()
                .find(|m| m.id == target.id)
                .map(|m| m.swipes.len())
                .is_some_and(|count| count > target.swipes.len()),
        )
    }

    /// The target rule matches [`MessageService::find_retry_anchor_msg`], so the
    /// target and the anchor cannot drift apart.
    pub(crate) fn resolve_retry_target(&self, messages: &[Message]) -> Option<RetryTarget> {
        let (anchor_idx, _anchor_msg, snapshot_id) =
            self.message_service.find_retry_anchor(messages)?;

        let old_target = messages
            .iter()
            .rev()
            .find(|m| m.is_input_or_narration())
            .cloned()?;
        let is_event = old_target.event_header().is_some();

        let mode = if is_event || old_target.message_type == MessageType::Narration {
            RetryMode::ReNarrate
        } else {
            // The find filter above yields only Narration | Input targets, so
            // this arm is Input (non-event); the debug_assert pins that invariant.
            debug_assert_eq!(old_target.message_type, MessageType::Input);
            if old_target.impersonated() {
                RetryMode::ReImpersonate
            } else {
                RetryMode::UserRegen
            }
        };

        Some(RetryTarget {
            anchor_idx,
            snapshot_id,
            is_event,
            mode,
            old_target: Some(old_target),
        })
    }

    /// A Message's Swipes all come from the one turn that created it, so any
    /// Swipe that carries a room name answers for the Message.
    fn message_location_header(message: &Message) -> Option<String> {
        message
            .swipes
            .iter()
            .find_map(|swipe| swipe.location_header.clone())
    }

    pub(crate) fn reconstruct_retry_state(
        snapshot: GameStateSnapshot,
        messages: Vec<Message>,
        target: &RetryTarget,
    ) -> GameState {
        let mut state = GameState::from_snapshot(&snapshot);
        // An event continuation's own Snapshot is written after
        // `commit_trigger_narration` consumed the pending room, so the room name is
        // seeded back from the Message and the redo's Swipe reads like its siblings.
        if target.is_event {
            state.narrative.pending_location = target
                .old_target
                .as_ref()
                .and_then(Self::message_location_header);
        }
        // When the anchor IS the target (a guided Narration with no prior
        // Input), truncate exclusively so the target lands only in
        // `retry_target` — the same shape the ReImpersonate/UserRegen paths
        // use for Input targets.
        let anchor_is_target = messages
            .get(target.anchor_idx)
            .zip(target.old_target.as_ref())
            .is_some_and(|(anchor, old)| anchor.id == old.id);
        let mut truncated = messages;
        match target.mode {
            RetryMode::ReNarrate if anchor_is_target => truncated.truncate(target.anchor_idx),
            RetryMode::ReNarrate => truncated.truncate(target.anchor_idx + 1),
            RetryMode::ReImpersonate | RetryMode::UserRegen => {
                truncated.truncate(target.anchor_idx)
            }
        }
        state.narrative.history.replace(truncated);
        state.narrative.retry_target = target.old_target.clone();
        state
    }

    fn check_retry_anchor(&self) -> Result<(), ApplicationError> {
        let messages = self.message_service.load_messages()?;
        let Some((_, anchor_msg)) = self.message_service.find_retry_anchor_msg(&messages) else {
            self.persist_generation_error(GenerationFailure::new(
                GenerationFailureKind::Other,
                "Retry failed: no anchor message",
            ));
            return Err(ApplicationError::internal(
                "Retry failed: no anchor message",
            ));
        };
        let Some(snapshot_id) = anchor_msg.snapshot_id() else {
            let msg = "Retry failed: anchor message has no snapshot_id";
            self.persist_generation_error(GenerationFailure::new(
                GenerationFailureKind::Other,
                msg,
            ));
            return Err(ApplicationError::internal(msg));
        };
        match self.storage.load_snapshot_by_id(snapshot_id) {
            Ok(Some(_)) => Ok(()),
            Ok(None) => {
                let msg = format!("Retry failed: no snapshot found for id {snapshot_id}");
                self.persist_generation_error(GenerationFailure::new(
                    GenerationFailureKind::Other,
                    msg.clone(),
                ));
                Err(ApplicationError::internal(msg))
            }
            Err(e) => {
                self.persist_generation_error(
                    GenerationFailure::from_engine_error(&e).with_context("Retry failed"),
                );
                Err(ApplicationError::internal(format!("Retry failed: {e}")))
            }
        }
    }

    pub(crate) fn retry_user_regen(&self, mut state: GameState) -> Result<(), PhaseError> {
        let started_for = self.storage.current_game_id();
        let run = PipelineRun::new(self, started_for);

        let Some(target) = state.narrative.retry_target.as_ref() else {
            Self::finalize_phase_error(
                &run,
                Some(&mut state),
                PhaseError::NarratorFailed(GenerationFailure::new(
                    GenerationFailureKind::Other,
                    "Retry failed: missing user-regen target",
                )),
            );
            return Ok(());
        };
        let instruction = self.build_user_regen_instruction(target.text());

        // The message lands as a Swipe on the retry target; a plain user input
        // never gets the tail.
        let inputs = crate::application::pipeline::narration_generation::GenerationInputs {
            input: instruction,
            guide: None,
            impersonate: None,
        };
        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::Narrating;

        match crate::application::pipeline::narration_generation::NarrationGeneration::new(
            &run, inputs,
        )
        .run(&mut state)
        {
            Err(PhaseError::Cancelled) => return Err(run.handle_cancellation()),
            Err(e) => {
                Self::finalize_phase_error(&run, Some(&mut state), e);
                return Ok(());
            }
            Ok(_) => {}
        }

        if let Some(target) = state.narrative.retry_target.take() {
            state.narrative.history.append(target);
        }

        run.phase_finalize(&mut state);
        Ok(())
    }

    fn build_user_regen_instruction(&self, original: &str) -> String {
        format!(
            "Regenerate the user's previous message as an alternate swipe.\n\
             Write only the replacement user message text.\n\
             Do not answer as the assistant, continue the assistant side, or describe what the assistant does next.\n\n\
             <original_user_message>\n\
             {original}\n\
             </original_user_message>"
        )
    }
}
