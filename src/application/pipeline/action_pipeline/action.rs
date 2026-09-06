//! [DOC: docs/diataxis/reference/game_flow.md]
//! Action entry path for the pipeline.

use tracing::instrument;

use crate::application::errors::ProcessActionResult;
use crate::application::generation::gate::GenerationGate;
use super::core::ActionPipeline;
use crate::application::pipeline::phase_error::PhaseError;
use crate::domain::model::action::Action;
use crate::domain::model::message::GenerationReplay;
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
        let (input, replay) = match action {
            Action::FreeAction(input) if input.is_empty() => (String::new(), None),
            Action::FreeAction(input) => (input, None),
            // The guide rides on the Swipe's stored inputs (not history), so a redo re-applies it.
            Action::Guide(guide) => (
                String::new(),
                Some(GenerationReplay {
                    guide: Some(guide),
                    ..Default::default()
                }),
            ),
            // Entry-time pinning: the active impersonate preset id is read here so the
            // Swipe records the preset that produced the generation (a redo re-applies it).
            Action::Impersonate(direction) => {
                let preset_id = {
                    let settings = self.settings.read().unwrap_or_else(|e| e.into_inner());
                    self.storage.active_impersonate_preset_id(&settings)
                };
                (
                    String::new(),
                    Some(GenerationReplay {
                        impersonate: true,
                        impersonate_direction: direction,
                        impersonate_preset_id: Some(preset_id),
                        ..Default::default()
                    }),
                )
            }
        };

        let spawn_input = input.clone();
        let spawn_replay = replay;
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
                pipeline.execute_action_with_replay(spawn_input, spawn_replay);
            },
        )
    }

    #[instrument(skip(self, replay), fields(input_length))]
    pub(crate) fn execute_action_with_replay(
        &self,
        input: String,
        replay: Option<GenerationReplay>,
    ) {
        let mut state = self.message_service.load_or_fresh();
        state.narrative.last_trigger = None;
        if let Err(PhaseError::Cancelled) = self.run_from_input(state, input, replay) {
            tracing::debug!("Pipeline cancelled");
        }
    }
}
