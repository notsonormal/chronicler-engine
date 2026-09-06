//! [DOC: docs/diataxis/reference/game_flow.md]
//! Action entry path for the pipeline.

use tracing::instrument;

use crate::application::errors::ProcessActionResult;
use crate::application::generation::gate::GenerationGate;
use super::core::ActionPipeline;
use crate::application::pipeline::phase_error::PhaseError;
use crate::domain::model::action::Action;
use crate::domain::model::state::message_types::MessageType;
use crate::error::EngineError;

impl ActionPipeline {
    /// Single gated entry. The empty free-action input is a continue —
    /// no message is persisted.
    pub fn process_action(
        &self,
        generation_gate: &GenerationGate,
        action: Action,
    ) -> Result<ProcessActionResult, EngineError> {
        // The impersonate preset resolves at generation time from the game's
        // active configuration — nothing is pinned at entry (ticket 15).
        let (input, impersonated, steering_instruction) = match action {
            Action::FreeAction(input) if input.is_empty() => (String::new(), false, None),
            Action::FreeAction(input) => (input, false, None),
            // The steering rides on the Swipe's stored inputs (not history), so a redo re-applies it.
            Action::Guide(guide) => (String::new(), false, Some(guide)),
            Action::Impersonate(direction) => (String::new(), true, direction),
        };

        let spawn_input = input.clone();
        let spawn_impersonated = impersonated;
        let spawn_steering_instruction = steering_instruction;
        self.claim_and_spawn(
            generation_gate,
            move |game_id, game_state| {
                generation_gate.heal_stale(game_id, game_state);

                self.message_service.save_state(game_state)?;

                let game = self.storage.require_game(game_id)?;
                let _persona = self.storage.require_persona(&game.persona_key)?;
                if !input.is_empty() {
                    game_state.add_message(input.clone(), MessageType::Input);
                }
                Ok(())
            },
            || Ok(()),
            move |pipeline| {
                pipeline.execute_action_with_inputs(
                    spawn_input,
                    spawn_impersonated,
                    spawn_steering_instruction,
                );
            },
        )
    }

    #[instrument(skip(self, impersonated, steering_instruction), fields(input_length))]
    pub(crate) fn execute_action_with_inputs(
        &self,
        input: String,
        impersonated: bool,
        steering_instruction: Option<String>,
    ) {
        let mut state = self.message_service.load_or_fresh();
        state.narrative.last_trigger = None;
        if let Err(PhaseError::Cancelled) =
            self.run_from_input(state, input, impersonated, steering_instruction)
        {
            tracing::debug!("Pipeline cancelled");
        }
    }
}
