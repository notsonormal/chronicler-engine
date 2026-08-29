//! [DOC: docs/diataxis/reference/game_flow.md]
//! Action entry path for the pipeline.

use tracing::instrument;

use crate::application::errors::ProcessActionResult;
use crate::application::generation::gate::GenerationGate;
use super::core::ActionPipeline;
use crate::application::pipeline::phase_error::PhaseError;
use crate::domain::model::message::GenerationReplay;
use crate::domain::model::state::message_types::MessageType;
use crate::error::EngineError;

impl ActionPipeline {
    pub fn process_action(
        &self,
        generation_gate: &GenerationGate,
        input: String,
    ) -> Result<ProcessActionResult, EngineError> {
        self.process_action_with_replay(generation_gate, input, None)
    }

    pub(crate) fn process_action_with_guide(
        &self,
        generation_gate: &GenerationGate,
        input: String,
        guide: Option<String>,
    ) -> Result<ProcessActionResult, EngineError> {
        let replay = guide.map(|g| GenerationReplay {
            guide: Some(g),
            ..Default::default()
        });
        self.process_action_with_replay(generation_gate, input, replay)
    }

    pub(crate) fn process_action_with_replay(
        &self,
        generation_gate: &GenerationGate,
        input: String,
        replay: Option<GenerationReplay>,
    ) -> Result<ProcessActionResult, EngineError> {
        let spawn_input = input.clone();
        let spawn_replay = replay.clone();
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

    pub fn execute_action(&self, input: String) {
        self.execute_action_with_replay(input, None)
    }

    #[instrument(skip(self, replay), fields(input_length))]
    pub(crate) fn execute_action_with_replay(
        &self,
        input: String,
        replay: Option<GenerationReplay>,
    ) {
        let mut state = self.message_service.load_or_fresh();
        state.narrative.last_trigger = None;
        if let Some(replay) = replay {
            state.narrative.pending_replay = Some(replay);
        }
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

    /// Transient steering: the guide is the final prompt layer and rides on the swipe replay blob (not history) so retry re-applies it.
    pub fn guide_narration(
        &self,
        generation_gate: &GenerationGate,
        guide: String,
    ) -> Result<ProcessActionResult, EngineError> {
        self.process_action_with_guide(generation_gate, String::new(), Some(guide))
    }

    /// Persists a permanent narrator directive in history; retry re-reads it naturally (no replay blob).
    pub fn narrator_action(
        &self,
        generation_gate: &GenerationGate,
        text: String,
    ) -> Result<ProcessActionResult, EngineError> {
        self.process_action_with_narrator(generation_gate, text)
    }

    fn process_action_with_narrator(
        &self,
        generation_gate: &GenerationGate,
        text: String,
    ) -> Result<ProcessActionResult, EngineError> {
        self.claim_and_spawn(
            generation_gate,
            move |game_id, game_state| {
                generation_gate.heal_stale(game_id, game_state);
                self.message_service.save_state(game_state)?;
                game_state.add_message(text.clone(), MessageType::Narrator);
                Ok(())
            },
            || Ok(()),
            move |pipeline| {
                pipeline.execute_action_with_replay(String::new(), None);
            },
        )
    }

    /// Transient steering: impersonate rides on the swipe replay blob (not history) so retry re-applies it.
    pub fn impersonate(
        &self,
        generation_gate: &GenerationGate,
        direction: Option<String>,
    ) -> Result<ProcessActionResult, EngineError> {
        let preset_id = {
            let settings = self.settings.read().unwrap_or_else(|e| e.into_inner());
            self.storage.active_impersonate_preset_id(&settings)
        };
        let replay = GenerationReplay {
            impersonate: true,
            impersonate_direction: direction,
            impersonate_preset_id: Some(preset_id),
            ..Default::default()
        };
        self.process_action_with_replay(generation_gate, String::new(), Some(replay))
    }
}
