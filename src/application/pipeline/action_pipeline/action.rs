//! [DOC: docs/diataxis/reference/game_flow.md]
//! Action entry path for the pipeline.

use tracing::instrument;

use crate::application::errors::ProcessActionResult;
use crate::application::generation::gate::GenerationGate;
use super::core::ActionPipeline;
use crate::application::pipeline::phase_error::PhaseError;
use crate::domain::model::action::Action;
use crate::domain::model::game::GameId;
use crate::domain::model::state::message_types::MessageType;
use crate::error::EngineError;

impl ActionPipeline {
    pub fn process_action(
        &self,
        generation_gate: &GenerationGate,
        game_id: GameId,
        action: Action,
    ) -> Result<ProcessActionResult, EngineError> {
        // The impersonate preset resolves at generation time from the game's active configuration.
        let (input, impersonated, guide_text, impersonate_direction) = match action {
            Action::FreeAction(input) => (input, false, None, None),
            // Steering rides on the Swipe's stored inputs, so a redo re-applies it.
            Action::Guide(guide) => (String::new(), false, Some(guide), None),
            Action::Impersonate(direction) => (String::new(), true, None, direction),
            Action::Options => return self.process_options(generation_gate, game_id),
        };

        let spawn_input = input.clone();
        self.claim_and_spawn(
            generation_gate,
            game_id,
            move |game_id, game_state| {
                let game = self.storage.require_game(game_id.0)?;
                let _persona = self.storage.require_persona(&game.persona_key)?;
                if !input.is_empty() {
                    game_state.add_message(input.clone(), MessageType::Input);
                }
                Ok(())
            },
            || Ok(()),
            move |pipeline| {
                pipeline.execute_action_with_inputs(
                    game_id,
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
        game_id: GameId,
        input: String,
        impersonated: bool,
        guide_text: Option<String>,
        impersonate_direction: Option<String>,
    ) {
        let mut state = self.message_service.load_or_fresh(game_id);
        state.narrative.last_trigger = None;
        if let Err(PhaseError::Cancelled) = self.run_from_input(
            game_id,
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
