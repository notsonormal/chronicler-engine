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
    pub fn process_action(
        &self,
        generation_gate: &GenerationGate,
        action: Action,
    ) -> Result<ProcessActionResult, EngineError> {
        // The impersonate preset resolves at generation time from the game's active configuration.
        let (input, impersonated, guide_text, impersonate_direction) = match action {
            Action::FreeAction(input) => (input, false, None, None),
            // Steering rides on the Swipe's stored inputs, so a redo re-applies it.
            Action::Guide(guide) => (String::new(), false, Some(guide), None),
            Action::Impersonate(direction) => (String::new(), true, None, direction),
            Action::Options => return self.process_options(generation_gate),
        };

        let spawn_input = input.clone();
        self.claim_and_spawn(
            generation_gate,
            move |game_id, game_state| {
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
                    impersonated,
                    guide_text,
                    impersonate_direction,
                );
            },
        )
    }

    #[instrument(
        skip(self, impersonated, guide_text, impersonate_direction),
        fields(input_length)
    )]
    pub(crate) fn execute_action_with_inputs(
        &self,
        input: String,
        impersonated: bool,
        guide_text: Option<String>,
        impersonate_direction: Option<String>,
    ) {
        let mut state = self.message_service.load_or_fresh();
        state.narrative.last_trigger = None;
        if let Err(PhaseError::Cancelled) = self.run_from_input(
            state,
            input,
            impersonated,
            guide_text,
            impersonate_direction,
        ) {
            tracing::debug!("Pipeline cancelled");
        }
    }
}
