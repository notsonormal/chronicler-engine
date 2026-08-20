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

    /// `process_action` carrying a guided-generation instruction. The guide is
    /// staged as a `pending_replay` blob by [`execute_action_with_replay`] and
    /// never enters history; a guided turn runs the continue path with empty input.
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

    /// `process_action` carrying a turn-conditions replay blob (guided
    /// generation or impersonate). The blob is staged as `pending_replay` by
    /// [`execute_action_with_replay`] and never enters history itself; it is
    /// recorded on the generated swipe so retry re-applies the steering.
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
                pipeline.execute_action_with_replay(spawn_input, spawn_replay);
            },
        )
    }

    pub fn execute_action(&self, input: String) {
        self.execute_action_with_replay(input, None)
    }

    /// `execute_action` carrying an optional turn-conditions replay blob. Stages
    /// it as `pending_replay` so the assembler renders the steering layer and
    /// the generated swipe records it for retry.
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

    /// Guided generation: a transient steering instruction on the next narration.
    /// Runs the continue path (no player input); the guide is the final prompt
    /// layer and is recorded on the swipe replay blob so retry re-applies it.
    pub fn guide_narration(
        &self,
        generation_gate: &GenerationGate,
        guide: String,
    ) -> Result<ProcessActionResult, EngineError> {
        self.process_action_with_guide(generation_gate, String::new(), Some(guide))
    }

    pub fn narrator_action(
        &self,
        generation_gate: &GenerationGate,
        text: String,
    ) -> Result<ProcessActionResult, EngineError> {
        drop(text);
        self.continue_narration(generation_gate)
    }

    /// Impersonate: force the next narration to be written as the player's
    /// persona. Runs the continue path (no player input); the impersonate
    /// preset replaces the system preset, the player-character layer is dropped,
    /// and the output is a player-voiced `Dialogue` message. Retry re-applies it.
    pub fn impersonate(
        &self,
        generation_gate: &GenerationGate,
        direction: Option<String>,
    ) -> Result<ProcessActionResult, EngineError> {
        let preset_id = self
            .settings
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .active_impersonate_prompt_preset_id
            .clone();
        let replay = GenerationReplay {
            impersonate: true,
            impersonate_direction: direction,
            impersonate_preset_id: Some(preset_id),
            ..Default::default()
        };
        self.process_action_with_replay(generation_gate, String::new(), Some(replay))
    }
}
