//! [DOC: docs/diataxis/reference/game_flow.md]
//! Retry entry path for the pipeline.

use tracing::instrument;

use crate::application::errors::{ApplicationError, ProcessActionResult};
use crate::application::generation::gate::GenerationGate;
use crate::application::pipeline::core::ActionPipeline;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::message_types::MessageType;

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

        let Some((anchor_idx, _anchor_msg, snapshot_id)) =
            self.message_service.find_retry_anchor(&messages)
        else {
            self.persist_generation_error("Retry failed: no anchor message");
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

        let snapshot = match self.storage.load_snapshot_by_id(snapshot_id) {
            Ok(Some(s)) => s,
            Ok(None) => {
                self.persist_generation_error(format!(
                    "Retry failed: no snapshot found for id {snapshot_id}"
                ));
                return;
            }
            Err(e) => {
                self.persist_generation_error(format!("Retry failed: {e}"));
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
                self.persist_generation_error("Retry failed: no input to retry");
                return;
            }
        };

        if is_event {
            let outcome = self.retry_event_continuation(&mut state);
            self.log_cancellation(outcome);
        } else {
            let outcome = self.retry_main_narration(state, input_text);
            self.log_cancellation(outcome);
        };
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
}
