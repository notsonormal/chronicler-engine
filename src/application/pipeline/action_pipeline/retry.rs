//! [DOC: docs/diataxis/reference/game_flow.md]
//! Retry entry path for the pipeline.

use tracing::instrument;

use crate::application::errors::{ApplicationError, ProcessActionResult};
use crate::application::generation::gate::GenerationGate;
use crate::application::pipeline::PhaseError;
use crate::application::pipeline::pipeline_run::PipelineRun;
use super::core::ActionPipeline;
use crate::domain::model::game::GameId;
use crate::domain::model::message::Message;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::state::generation_status::{
    GenerationFailure, GenerationFailureKind, GenerationStatus, Step,
};
use crate::domain::model::state::message_types::MessageType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DurableWorldAfterFailedRedo {
    /// The state the click found: no Swipe for the target matches the story on
    /// screen, so the rewound world the attempt left would sit behind it.
    ClickTime,
    /// The world the attempt left: a Swipe for the target is the narration on
    /// screen, and the failing write already saved the movement and trigger mark
    /// it holds. A failure between the narration persist and the post-engine
    /// Snapshot is this variant too, on purpose — too rare to special-case.
    Attempt,
}

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
        game_id: GameId,
    ) -> Result<ProcessActionResult, ApplicationError> {
        self.claim_and_spawn(
            generation_gate,
            game_id,
            |_game_id, game_state| {
                if !game_state.narrative.history.last_turn_is_retryable() {
                    return Err(ApplicationError::validation("No input to retry"));
                }
                Ok(())
            },
            move || self.check_retry_anchor(game_id),
            move |pipeline| {
                pipeline.retry_last_response(game_id);
            },
        )
    }

    #[instrument(skip(self))]
    pub fn retry_last_response(&self, retry_game_id: GameId) {
        let messages = match self.message_service.load_messages(retry_game_id) {
            Ok(m) => m,
            Err(e) => {
                self.persist_generation_error(
                    retry_game_id,
                    GenerationFailure::from_engine_error(&e).with_context("Retry failed"),
                );
                return;
            }
        };

        let Some(target) = self.resolve_retry_target(&messages) else {
            self.persist_generation_error(
                retry_game_id,
                GenerationFailure::new(
                    GenerationFailureKind::Other,
                    "Retry failed: no anchor message",
                ),
            );
            return;
        };

        let state_at_click = match self.storage.load_latest_snapshot(retry_game_id) {
            Ok(snapshot) => snapshot,
            Err(e) => {
                self.persist_generation_error(
                    retry_game_id,
                    GenerationFailure::from_engine_error(&e).with_context("Retry failed"),
                );
                return;
            }
        };

        // An event continuation regenerates from the state its own Message was
        // committed onto: that Swipe's Snapshot holds the trigger context, the
        // post-movement room and the scene before the post-trigger reconciliation.
        let restore_snapshot_id = if target.is_event {
            let Some(snapshot_id) = target.old_target.as_ref().and_then(|m| m.snapshot_id()) else {
                self.persist_generation_error(
                    retry_game_id,
                    GenerationFailure::new(
                        GenerationFailureKind::Other,
                        "Retry failed: event message has no snapshot",
                    ),
                );
                return;
            };
            snapshot_id
        } else {
            target.snapshot_id
        };

        let snapshot = match self
            .storage
            .load_snapshot_by_id(retry_game_id, restore_snapshot_id)
        {
            Ok(Some(s)) => s,
            Ok(None) => {
                self.persist_generation_error(
                    retry_game_id,
                    GenerationFailure::new(
                        GenerationFailureKind::Other,
                        format!("Retry failed: no snapshot found for id {restore_snapshot_id}"),
                    ),
                );
                return;
            }
            Err(e) => {
                self.persist_generation_error(
                    retry_game_id,
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
                            self.persist_generation_error(
                                retry_game_id,
                                GenerationFailure::new(
                                    GenerationFailureKind::Other,
                                    "Retry failed: no input to retry",
                                ),
                            );
                            return;
                        }
                    }
                };
                if target.is_event {
                    let outcome = self.retry_event_continuation(retry_game_id, &mut state);
                    self.log_cancellation(outcome);
                } else {
                    let outcome = self.retry_main_narration(retry_game_id, state, input_text);
                    self.log_cancellation(outcome);
                }
            }
            RetryMode::ReImpersonate => {
                let outcome = self.retry_main_narration(retry_game_id, state, String::new());
                self.log_cancellation(outcome);
            }
            RetryMode::UserRegen => {
                let outcome = self.retry_user_regen(retry_game_id, state);
                self.log_cancellation(outcome);
            }
        }

        self.restore_click_time_state_after_failed_redo(retry_game_id, state_at_click, &target);
    }

    /// A failed redo leaves the rewound world of its anchor's Snapshot behind
    /// the Messages it wrote, so the world the click found comes back while
    /// those Messages stay.
    fn restore_click_time_state_after_failed_redo(
        &self,
        retry_game_id: GameId,
        state_at_click: Option<GameStateSnapshot>,
        target: &RetryTarget,
    ) {
        let Some(state_at_click) = state_at_click else {
            return;
        };
        if self.game_catalogue.active_game_id() != Some(retry_game_id) {
            tracing::warn!("Retry: the game changed while the redo ran; leaving it alone");
            return;
        }
        let attempt = match self.storage.load_latest_snapshot(retry_game_id) {
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
        match self.durable_world_after_failed_redo(retry_game_id, target) {
            DurableWorldAfterFailedRedo::Attempt => return,
            DurableWorldAfterFailedRedo::ClickTime => {}
        }
        let mut state = GameState::from_snapshot(&state_at_click);
        state.narrative.input_buffer.status = GenerationStatus::Error(failure.clone());
        state.narrative.input_buffer.step = attempt.narrative.input_buffer.step;
        if let Err(e) = self.message_service.save_state(retry_game_id, &state) {
            tracing::error!("Failed to put the game state back after a failed retry: {e}");
        }
    }

    /// A redo persists its anchor's older world while it runs; only its finalize
    /// makes the new world durable.
    ///
    /// Messages that cannot be read answer `Attempt` too: an unknown outcome is
    /// not a licence to overwrite a game.
    fn durable_world_after_failed_redo(
        &self,
        game_id: GameId,
        target: &RetryTarget,
    ) -> DurableWorldAfterFailedRedo {
        let Some(target) = target.old_target.as_ref() else {
            return DurableWorldAfterFailedRedo::Attempt;
        };
        let messages = match self.message_service.load_messages(game_id) {
            Ok(messages) => messages,
            Err(e) => {
                tracing::error!("Retry: cannot read the Messages a failed redo left: {e}");
                return DurableWorldAfterFailedRedo::Attempt;
            }
        };
        let persisted_a_swipe = messages
            .iter()
            .find(|m| m.id == target.id)
            .map(|m| m.swipes.len())
            .is_some_and(|count| count > target.swipes.len());
        if persisted_a_swipe {
            DurableWorldAfterFailedRedo::Attempt
        } else {
            DurableWorldAfterFailedRedo::ClickTime
        }
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

    pub(crate) fn reconstruct_retry_state(
        snapshot: GameStateSnapshot,
        messages: Vec<Message>,
        target: &RetryTarget,
    ) -> GameState {
        let mut state = GameState::from_snapshot(&snapshot);
        // The tail re-appends the target from `retry_target`, so it must not stay
        // in history: an anchor that IS the target truncates exclusively.
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

    fn check_retry_anchor(&self, game_id: GameId) -> Result<(), ApplicationError> {
        let messages = self.message_service.load_messages(game_id)?;
        let Some((_, anchor_msg)) = self.message_service.find_retry_anchor_msg(&messages) else {
            self.persist_generation_error(
                game_id,
                GenerationFailure::new(
                    GenerationFailureKind::Other,
                    "Retry failed: no anchor message",
                ),
            );
            return Err(ApplicationError::internal(
                "Retry failed: no anchor message",
            ));
        };
        let Some(snapshot_id) = anchor_msg.snapshot_id() else {
            let msg = "Retry failed: anchor message has no snapshot_id";
            self.persist_generation_error(
                game_id,
                GenerationFailure::new(GenerationFailureKind::Other, msg),
            );
            return Err(ApplicationError::internal(msg));
        };
        match self.storage.load_snapshot_by_id(game_id, snapshot_id) {
            Ok(Some(_)) => Ok(()),
            Ok(None) => {
                let msg = format!("Retry failed: no snapshot found for id {snapshot_id}");
                self.persist_generation_error(
                    game_id,
                    GenerationFailure::new(GenerationFailureKind::Other, msg.clone()),
                );
                Err(ApplicationError::internal(msg))
            }
            Err(e) => {
                self.persist_generation_error(
                    game_id,
                    GenerationFailure::from_engine_error(&e).with_context("Retry failed"),
                );
                Err(ApplicationError::internal(format!("Retry failed: {e}")))
            }
        }
    }

    pub(crate) fn retry_user_regen(
        &self,
        game_id: GameId,
        mut state: GameState,
    ) -> Result<(), PhaseError> {
        let run = PipelineRun::new(self, game_id);

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

        let inputs = crate::application::pipeline::narration_generation::GenerationInputs {
            input: instruction,
            guide: None,
            impersonate: None,
        };
        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.step = Step::Narrating;

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
