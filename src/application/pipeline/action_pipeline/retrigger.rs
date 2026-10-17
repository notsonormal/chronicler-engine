//! [DOC: docs/diataxis/reference/game_flow.md]
//! Retrigger entry path for the pipeline.

use tracing::instrument;

use crate::application::errors::{ApplicationError, ProcessActionResult};
use crate::application::generation::gate::GenerationGate;
use super::core::ActionPipeline;
use crate::domain::model::game::GameId;
use crate::domain::model::state::message_types::MessageType;

impl ActionPipeline {
    pub fn retrigger(
        &self,
        generation_gate: &GenerationGate,
        game_id: GameId,
    ) -> Result<ProcessActionResult, ApplicationError> {
        self.claim_and_spawn(
            generation_gate,
            game_id,
            |_game_id, game_state| {
                if game_state.narrative.last_trigger.is_none() {
                    return Err(ApplicationError::validation("No trigger context available"));
                }
                let messages = self.message_service.load_messages(game_id)?;
                let Some(last_msg) = messages.last() else {
                    return Err(ApplicationError::validation("No messages to retrigger"));
                };

                let is_narration = last_msg.message_type == MessageType::Narration;

                if !is_narration || last_msg.event_header().is_some() {
                    return Err(ApplicationError::validation(
                        "Last message must be a narration to retrigger",
                    ));
                }
                Ok(())
            },
            || Ok(()),
            move |pipeline| {
                pipeline.retrigger_event(game_id);
            },
        )
    }

    #[instrument(skip(self))]
    pub fn retrigger_event(&self, game_id: GameId) {
        let mut state = self.message_service.load_or_fresh(game_id);
        let outcome = self.retry_event_continuation(game_id, &mut state);
        self.log_cancellation(outcome);
    }
}
