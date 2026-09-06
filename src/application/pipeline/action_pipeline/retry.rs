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
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;

/// How a retry should reinterpret the last message.
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
        let messages = match self.message_service.load_messages() {
            Ok(m) => m,
            Err(e) => {
                self.persist_generation_error(format!("Retry failed: {e}"));
                return;
            }
        };

        let Some(target) = self.resolve_retry_target(&messages) else {
            self.persist_generation_error("Retry failed: no anchor message");
            return;
        };

        let snapshot = match self.storage.load_snapshot_by_id(target.snapshot_id) {
            Ok(Some(s)) => s,
            Ok(None) => {
                self.persist_generation_error(format!(
                    "Retry failed: no snapshot found for id {}",
                    target.snapshot_id
                ));
                return;
            }
            Err(e) => {
                self.persist_generation_error(format!("Retry failed: {e}"));
                return;
            }
        };

        let mut state = Self::reconstruct_retry_state(snapshot, messages, &target);

        match target.mode {
            RetryMode::ReNarrate => {
                // A guided turn carries its input on the swipe replay, not in
                // an Input row — redo it with the same empty input the
                // original generation used, not with an older turn's input.
                let target_is_guided = target
                    .old_target
                    .as_ref()
                    .is_some_and(|m| m.replay().is_some_and(|r| r.guide.is_some()));
                let input_text = if target_is_guided {
                    String::new()
                } else {
                    match state.narrative.history.last_input_text() {
                        Some(text) => text,
                        None => {
                            self.persist_generation_error("Retry failed: no input to retry");
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
                // Redo = fresh Impersonate from the Swipe's stored inputs, with the
                // full tail (quantifier → engine commit → trigger).
                let outcome = self.retry_main_narration(state, String::new());
                self.log_cancellation(outcome);
            }
            RetryMode::UserRegen => {
                let outcome = self.retry_user_regen(state);
                self.log_cancellation(outcome);
            }
        }
    }

    pub(crate) fn resolve_retry_target(&self, messages: &[Message]) -> Option<RetryTarget> {
        let (anchor_idx, _anchor_msg, snapshot_id) =
            self.message_service.find_retry_anchor(messages)?;

        let old_target = messages
            .iter()
            .rev()
            .find(|m| {
                m.message_type == MessageType::Narration || m.message_type == MessageType::Input
            })
            .cloned()?;
        let is_event = old_target.event_header().is_some();

        let mode = if is_event || old_target.message_type == MessageType::Narration {
            RetryMode::ReNarrate
        } else {
            // The find filter above yields only Narration | Input targets, so
            // this arm is Input (non-event). The debug_assert pins that
            // invariant instead of a dead `else => return None` arm.
            debug_assert_eq!(old_target.message_type, MessageType::Input);
            if old_target.replay().is_some_and(|replay| replay.impersonate) {
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

    pub(crate) fn reconstruct_retry_state(
        snapshot: GameStateSnapshot,
        messages: Vec<Message>,
        target: &RetryTarget,
    ) -> GameState {
        let mut state = GameState::from_snapshot(&snapshot);
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
            self.persist_generation_error("Retry failed: no anchor message");
            return Err(ApplicationError::internal(
                "Retry failed: no anchor message",
            ));
        };
        let Some(snapshot_id) = anchor_msg.snapshot_id() else {
            let msg = "Retry failed: anchor message has no snapshot_id";
            self.persist_generation_error(msg);
            return Err(ApplicationError::internal(msg));
        };
        match self.storage.load_snapshot_by_id(snapshot_id) {
            Ok(Some(_)) => Ok(()),
            Ok(None) => {
                let msg = format!("Retry failed: no snapshot found for id {snapshot_id}");
                self.persist_generation_error(msg.clone());
                Err(ApplicationError::internal(msg))
            }
            Err(e) => {
                let msg = format!("Retry failed: {e}");
                self.persist_generation_error(msg.clone());
                Err(ApplicationError::internal(msg))
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
                PhaseError::NarratorFailed("Retry failed: missing user-regen target".to_string()),
            );
            return Ok(());
        };
        let instruction = self.build_user_regen_instruction(target.text());

        // The message lands as a Swipe on the retry target (no tail — a plain
        // user input never gets one).
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
