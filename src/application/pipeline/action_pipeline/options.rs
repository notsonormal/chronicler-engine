//! [DOC: docs/diataxis/reference/narrative/options.md]
//! Options pipeline slice — on-demand entry, agent dispatch, and the
//! always-on turn-end rewrite.

use std::collections::HashMap;
use std::sync::Arc;

use tracing::instrument;

use crate::application::errors::ProcessActionResult;
use crate::application::generation::gate::GenerationGate;
use crate::application::pipeline::phase_error::PhaseError;
use crate::application::pipeline::pipeline_run::PipelineRun;
use super::core::ActionPipeline;
use crate::adapters::driven::storage::worlds::WorldBundle;
use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::game::GameId;
use crate::domain::model::map::MapDef;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::error::EngineError;

impl ActionPipeline {
    pub fn process_options(
        &self,
        generation_gate: &GenerationGate,
        game_id: GameId,
    ) -> Result<ProcessActionResult, EngineError> {
        self.claim_and_spawn(
            generation_gate,
            game_id,
            |_game_id, game_state| {
                if game_state.narrative.history().is_empty() {
                    return Err(EngineError::Validation(
                        "No scene to generate options for yet".to_string(),
                    ));
                }
                Ok(())
            },
            || Ok(()),
            move |pipeline| {
                pipeline.execute_options_refresh(game_id);
            },
        )
    }

    /// The Options hook of one Game: the values its Agent reads, built from the
    /// state the Job holds. `Ok(None)` means no Options Agent is registered.
    pub(crate) fn run_options_generation(
        &self,
        game_id: GameId,
        state: &GameState,
        map: &Arc<MapDef>,
        persona: &Arc<PersonaCard>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<Option<Vec<String>>, EngineError> {
        let run = PipelineRun::new(self, game_id);
        let kind = PipelineRun::scene_kind(state);
        PipelineRun::with_hook_input(state, map, persona, npcs, None, "", kind, |input| {
            self.agent_runner.options(input, &run.options_settings())
        })
    }

    /// A failed refresh keeps the prior set plus a system message — the scene did not change.
    #[instrument(skip(self))]
    pub(crate) fn execute_options_refresh(&self, game_id: GameId) {
        let run = PipelineRun::new(self, game_id);
        let mut state = self.message_service.load_or_fresh(game_id);

        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::Options;
        if let Err(e) = run.persist_snapshot_or_err(&mut state, "pre-options snapshot") {
            Self::finalize_phase_error(&run, Some(&mut state), e);
            return;
        }

        match run.check_game_unchanged(game_id) {
            Ok(()) => {}
            Err(PhaseError::Cancelled) => {
                let _ = run.handle_cancellation();
                return;
            }
            Err(e) => {
                Self::finalize_phase_error(&run, Some(&mut state), e);
                return;
            }
        }

        let WorldBundle {
            map, persona, npcs, ..
        } = match self.load_world_bundle(game_id) {
            Ok(bundle) => bundle,
            Err(e) => {
                self.finalize_options_failure(
                    &mut state,
                    &run,
                    format!("[System] Options generation failed: {e}"),
                );
                return;
            }
        };

        match self.run_options_generation(game_id, &state, &map, &persona, &npcs) {
            Ok(Some(options)) => {
                state.narrative.current_options = options;
                if let Err(e) = self
                    .message_service
                    .save_message_and_snapshot(game_id, &mut state)
                {
                    tracing::error!("Failed to save options set: {e}");
                }
            }
            Ok(None) => {
                self.finalize_options_failure(
                    &mut state,
                    &run,
                    "[System] Options agent is not available".to_string(),
                );
                return;
            }
            Err(e) => {
                self.finalize_options_failure(
                    &mut state,
                    &run,
                    format!("[System] Options generation failed: {e}"),
                );
                return;
            }
        }

        run.phase_finalize(&mut state);
    }

    fn finalize_options_failure(
        &self,
        state: &mut GameState,
        run: &PipelineRun<'_>,
        message: String,
    ) {
        tracing::warn!("Options generation failed: {message}");
        state.add_message(message, MessageType::System);
        if let Err(e) = self
            .message_service
            .save_message_and_snapshot(run.started_for, state)
        {
            tracing::error!("Failed to persist options failure message: {e}");
        }
        run.phase_finalize(state);
    }

    /// A failed regeneration empties the set: it must always describe the current scene.
    pub(crate) fn rewrite_options_after_turn(
        &self,
        game_id: GameId,
        state: &mut GameState,
        map: &Arc<MapDef>,
        persona: &Arc<PersonaCard>,
        npcs: &HashMap<String, NpcCard>,
        options_enabled: bool,
    ) {
        state.narrative.current_options.clear();
        if options_enabled {
            match self.run_options_generation(game_id, state, map, persona, npcs) {
                Ok(Some(options)) => {
                    state.narrative.current_options = options;
                }
                Ok(None) => {
                    state.add_message(
                        "[System] Options agent is not available".to_string(),
                        MessageType::System,
                    );
                }
                Err(e) => {
                    state.add_message(
                        format!("[System] Options generation failed: {e}"),
                        MessageType::System,
                    );
                }
            }
        }
        if let Err(e) = self
            .message_service
            .save_message_and_snapshot(game_id, state)
        {
            tracing::warn!("Failed to save post-turn options rewrite: {e}");
        }
    }
}
