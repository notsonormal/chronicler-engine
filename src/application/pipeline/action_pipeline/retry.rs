//! [DOC: docs/diataxis/reference/game_flow.md]
//! Retry entry path for the pipeline.

use tracing::instrument;

use crate::application::errors::{ApplicationError, ProcessActionResult};
use crate::application::generation::gate::GenerationGate;
use crate::application::pipeline::PhaseError;
use crate::application::pipeline::pipeline_run::PipelineRun;
use crate::application::prompting::{NpcContext, PromptContext};
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
                if game_state.narrative.history.last_input_text().is_none() {
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
                let input_text = match state.narrative.history.last_input_text() {
                    Some(text) => text,
                    None => {
                        self.persist_generation_error("Retry failed: no input to retry");
                        return;
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
                let outcome = self.retry_reimpersonate(state);
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
        } else if old_target.message_type == MessageType::Input {
            if old_target.replay().is_some_and(|replay| replay.impersonate) {
                RetryMode::ReImpersonate
            } else {
                RetryMode::UserRegen
            }
        } else {
            return None;
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
        let mut truncated = messages;
        match target.mode {
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

    pub(crate) fn retry_reimpersonate(&self, mut state: GameState) -> Result<(), PhaseError> {
        let started_for = self.storage.current_game_id();
        let run = PipelineRun::new(self, started_for);

        let bundle = match self.load_world_bundle(started_for) {
            Ok(b) => b,
            Err(e) => {
                Self::finalize_phase_error(
                    &run,
                    Some(&mut state),
                    PhaseError::FetchFailed(e.to_string()),
                );
                return Ok(());
            }
        };

        let Some(room) = bundle
            .map
            .get_room_by_id(&state.movement.current_room_id)
            .or_else(|| {
                state
                    .movement
                    .dynamic_rooms
                    .get(&state.movement.current_room_id)
            })
        else {
            Self::finalize_phase_error(
                &run,
                Some(&mut state),
                PhaseError::NarratorFailed("Room not found".to_string()),
            );
            return Ok(());
        };

        let Some(target) = state.narrative.retry_target.as_ref() else {
            Self::finalize_phase_error(
                &run,
                Some(&mut state),
                PhaseError::NarratorFailed("Retry failed: missing impersonate target".to_string()),
            );
            return Ok(());
        };

        let Some(replay) = target.replay().cloned() else {
            Self::finalize_phase_error(
                &run,
                Some(&mut state),
                PhaseError::NarratorFailed(
                    "Retry failed: impersonate target has no replay".to_string(),
                ),
            );
            return Ok(());
        };

        let (preset, response_length) = match run
            .load_impersonate_preset_and_response_length(replay.impersonate_preset_id.as_deref())
        {
            Ok(p) => p,
            Err(msg) => {
                Self::finalize_phase_error(&run, Some(&mut state), PhaseError::NarratorFailed(msg));
                return Ok(());
            }
        };

        let impersonate_direction = replay.impersonate_direction.unwrap_or_default();
        let all_npcs: Vec<_> = bundle.npcs.values().cloned().collect();
        let history = state.narrative.history();
        let context = PromptContext::new(
            &bundle.world,
            room,
            NpcContext {
                all_npcs: &all_npcs,
                npcs_in_area: &state.scene.npcs_in_area,
            },
            &bundle.persona,
            &impersonate_direction,
            &history,
        )
        .with_impersonate(true);

        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::Narrating;

        let (narration_text, _backend, _model) =
            match run.call_narrator(&context, &preset, &response_length) {
                Ok(triple) => triple,
                Err(msg) => {
                    Self::finalize_phase_error(
                        &run,
                        Some(&mut state),
                        PhaseError::NarratorFailed(msg),
                    );
                    return Ok(());
                }
            };

        run.check_game_unchanged(started_for)?;

        state.add_message(narration_text, MessageType::Input);

        if let Err(source) = self.message_service.save_message_and_snapshot(&mut state) {
            Self::finalize_phase_error(
                &run,
                Some(&mut state),
                PhaseError::PersistFailed {
                    label: "re-impersonate swipe",
                    source,
                },
            );
            return Ok(());
        }

        if let Some(target) = state.narrative.retry_target.take() {
            state.narrative.history.append(target);
        }

        run.phase_finalize(&mut state);
        Ok(())
    }

    pub(crate) fn retry_user_regen(&self, mut state: GameState) -> Result<(), PhaseError> {
        let started_for = self.storage.current_game_id();
        let run = PipelineRun::new(self, started_for);

        let bundle = match self.load_world_bundle(started_for) {
            Ok(b) => b,
            Err(e) => {
                Self::finalize_phase_error(
                    &run,
                    Some(&mut state),
                    PhaseError::FetchFailed(e.to_string()),
                );
                return Ok(());
            }
        };

        let Some(room) = bundle
            .map
            .get_room_by_id(&state.movement.current_room_id)
            .or_else(|| {
                state
                    .movement
                    .dynamic_rooms
                    .get(&state.movement.current_room_id)
            })
        else {
            Self::finalize_phase_error(
                &run,
                Some(&mut state),
                PhaseError::NarratorFailed("Room not found".to_string()),
            );
            return Ok(());
        };

        let Some(target) = state.narrative.retry_target.as_ref() else {
            Self::finalize_phase_error(
                &run,
                Some(&mut state),
                PhaseError::NarratorFailed("Retry failed: missing user-regen target".to_string()),
            );
            return Ok(());
        };

        let (preset, response_length) = match run.load_preset_and_response_length() {
            Ok(p) => p,
            Err(msg) => {
                Self::finalize_phase_error(&run, Some(&mut state), PhaseError::NarratorFailed(msg));
                return Ok(());
            }
        };

        let user_message = self.build_user_regen_instruction(target.text());
        let all_npcs: Vec<_> = bundle.npcs.values().cloned().collect();
        let history = state.narrative.history();
        let context = PromptContext::new(
            &bundle.world,
            room,
            NpcContext {
                all_npcs: &all_npcs,
                npcs_in_area: &state.scene.npcs_in_area,
            },
            &bundle.persona,
            &user_message,
            &history,
        );

        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::Narrating;

        let (narration_text, _backend, _model) =
            match run.call_narrator(&context, &preset, &response_length) {
                Ok(triple) => triple,
                Err(msg) => {
                    Self::finalize_phase_error(
                        &run,
                        Some(&mut state),
                        PhaseError::NarratorFailed(msg),
                    );
                    return Ok(());
                }
            };

        run.check_game_unchanged(started_for)?;

        state.add_message(narration_text, MessageType::Input);

        if let Err(source) = self.message_service.save_message_and_snapshot(&mut state) {
            Self::finalize_phase_error(
                &run,
                Some(&mut state),
                PhaseError::PersistFailed {
                    label: "user-regen swipe",
                    source,
                },
            );
            return Ok(());
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
