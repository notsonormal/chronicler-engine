//! [DOC: docs/diataxis/reference/game_flow.md]
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
use crate::domain::model::map::MapDef;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::agent::{AgentContext, AgentResult, ExecutionPhase};
use crate::error::EngineError;

impl ActionPipeline {
    /// On-demand options generation (`/options`): claim the generation slot
    /// and refresh the offered set against the current scene, without
    /// narrating. Requires scene history — options need a scene to ground in.
    pub fn process_options(
        &self,
        generation_gate: &GenerationGate,
    ) -> Result<ProcessActionResult, EngineError> {
        self.claim_and_spawn(
            generation_gate,
            |_game_id, game_state| {
                if game_state.narrative.history().is_empty() {
                    return Err(EngineError::Validation(
                        "No scene to generate options for yet".to_string(),
                    ));
                }
                Ok(())
            },
            || Ok(()),
            |pipeline| {
                pipeline.execute_options_refresh();
            },
        )
    }

    /// Run the registered OptionsGeneration agents against the current scene
    /// and return the parsed option set.
    ///
    /// `Ok(empty)` means no options agent is registered (an `[agents]`
    /// settings section without an `options` entry); callers surface that as
    /// an unavailable-agent failure.
    pub(crate) fn run_options_generation(
        &self,
        state: &GameState,
        map: &Arc<MapDef>,
        persona: &Arc<PersonaCard>,
        npcs: &HashMap<String, NpcCard>,
    ) -> Result<Vec<String>, EngineError> {
        let current_room = map
            .get_room_by_id(&state.movement.current_room_id)
            .or_else(|| {
                state
                    .movement
                    .dynamic_rooms
                    .get(&state.movement.current_room_id)
            });
        let agent_ctx = AgentContext {
            state,
            main_response: None,
            player_input: "",
            current_room,
            map,
            persona,
            npcs,
        };

        let mut last_err = None;
        for agent in self
            .agent_registry
            .agents_for_phase(ExecutionPhase::OptionsGeneration)
        {
            match agent.execute(&agent_ctx) {
                Ok(AgentResult::Options(options)) => return Ok(options),
                Ok(_) => continue,
                Err(e) => {
                    tracing::warn!("Options agent {} failed: {e}", agent.name());
                    last_err = Some(e);
                }
            }
        }
        match last_err {
            Some(e) => Err(e),
            None => Ok(Vec::new()),
        }
    }

    /// On-demand options refresh (`/options`): runs the options agent against
    /// the current scene and replaces the offered set in place. Failures
    /// surface a system message and KEEP the prior set — the scene did not
    /// change, so the old options stay usable.
    #[instrument(skip(self))]
    pub(crate) fn execute_options_refresh(&self) {
        let started_for = self.storage.current_game_id();
        let run = PipelineRun::new(self, started_for);
        let mut state = self.message_service.load_or_fresh();

        state.narrative.input_buffer.status = GenerationStatus::Generating;
        state.narrative.input_buffer.phase = GenerationPhase::Options;
        if let Err(e) = run.persist_snapshot_or_err(&mut state, "pre-options snapshot") {
            Self::finalize_phase_error(&run, Some(&mut state), e);
            return;
        }

        match run.check_game_unchanged(started_for) {
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
        } = match self.load_world_bundle(started_for) {
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

        match self.run_options_generation(&state, &map, &persona, &npcs) {
            Ok(options) if !options.is_empty() => {
                state.narrative.current_options = options;
                if let Err(e) = self.message_service.save_message_and_snapshot(&mut state) {
                    tracing::error!("Failed to save options set: {e}");
                }
            }
            Ok(_) => {
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

    /// Surface an options failure as a system message, keep the prior offered
    /// set, and finish the run as a normal (non-error) turn.
    fn finalize_options_failure(
        &self,
        state: &mut GameState,
        run: &PipelineRun<'_>,
        message: String,
    ) {
        tracing::warn!("Options generation failed: {message}");
        state.add_message(message, MessageType::System);
        if let Err(e) = self.message_service.save_message_and_snapshot(state) {
            tracing::error!("Failed to persist options failure message: {e}");
        }
        run.phase_finalize(state);
    }

    /// Turn-end rewrite of the offered option set (always-on trigger):
    /// the prior set clears, then regenerates when `options_enabled`. A
    /// failed regeneration leaves the set empty plus a system message — the
    /// set must always describe the current scene, never a stale one.
    pub(crate) fn rewrite_options_after_turn(
        &self,
        state: &mut GameState,
        map: &Arc<MapDef>,
        persona: &Arc<PersonaCard>,
        npcs: &HashMap<String, NpcCard>,
        options_enabled: bool,
    ) {
        state.narrative.current_options.clear();
        if options_enabled {
            match self.run_options_generation(state, map, persona, npcs) {
                Ok(options) if !options.is_empty() => {
                    state.narrative.current_options = options;
                }
                Ok(_) => {
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
        if let Err(e) = self.message_service.save_message_and_snapshot(state) {
            tracing::warn!("Failed to save post-turn options rewrite: {e}");
        }
    }
}
