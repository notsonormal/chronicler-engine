//! [DOC: docs/diataxis/reference/game_flow.md]
//! Action entry path for the pipeline.

use tracing::instrument;

use crate::application::errors::ProcessActionResult;
use crate::application::generation::gate::GenerationGate;
use crate::application::pipeline::core::ActionPipeline;
use crate::application::pipeline::phase_error::PhaseError;
use crate::domain::model::state::message_types::MessageType;
use crate::error::EngineError;

impl ActionPipeline {
    pub fn process_action(
        &self,
        generation_gate: &GenerationGate,
        input: String,
    ) -> Result<ProcessActionResult, EngineError> {
        let spawn_input = input.clone();
        self.claim_and_spawn(
            generation_gate,
            move |game_id, game_state| {
                generation_gate.heal_stale(game_id, game_state);

                self.message_service.save_state(game_state)?;

                let game = self.storage.require_game(game_id)?;
                let persona = self.storage.require_persona(&game.persona_key)?;
                let player_name = persona.sheet.name.clone();
                if !input.is_empty() {
                    game_state.add_message(
                        input.clone(),
                        Some(player_name.clone()),
                        MessageType::Input,
                    );
                }
                Ok(())
            },
            || Ok(()),
            move |pipeline| {
                pipeline.execute_action(spawn_input);
            },
        )
    }

    #[instrument(skip(self), fields(input_length))]
    pub fn execute_action(&self, input: String) {
        let mut state = self.message_service.load_or_fresh();
        state.narrative.last_trigger = None;
        if let Err(PhaseError::Cancelled) = self.run_from_input(state, input) {
            tracing::debug!("Pipeline cancelled");
        }
    }

    pub fn continue_narration(
        &self,
        generation_gate: &GenerationGate,
    ) -> Result<ProcessActionResult, EngineError> {
        self.process_action(generation_gate, String::new())
    }
}
