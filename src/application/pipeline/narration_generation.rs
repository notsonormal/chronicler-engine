//! [DOC: docs/diataxis/reference/game_flow.md]
//! Narration generation — the narrate-and-persist prefix of one generation.

use std::sync::Arc;

use crate::adapters::driven::storage::worlds::WorldBundle;
use crate::application::prompting::{NpcContext, PromptContext};
use crate::domain::model::character::NpcCard;
use crate::domain::model::map::{MapDef, Room};
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::message_types::MessageType;

use super::phase_error::PhaseError;
use super::pipeline_run::{PipelineRun, PresetKind};

/// The phase never reads `retry_target` — on a redo the caller reads the
/// Swipe's stored inputs and passes them here.
pub(crate) struct GenerationInputs {
    /// Empty for continues and slash-command generations; persisted for free
    /// actions by the entry path, not here.
    pub input: String,
    pub guide: Option<String>,
    pub impersonate: Option<ImpersonateInputs>,
}

#[derive(Debug, Clone)]
pub(crate) struct ImpersonateInputs {
    pub steering_instruction: Option<String>,
}

/// The tail (quantifier → engine commit → trigger) runs on the same bundle
/// the narration was generated from, so the loaded bundle rides on the outcome.
#[derive(Debug)]
pub(crate) struct NarrationOutcome {
    pub narration_text: String,
    pub backend_name: String,
    pub model_name: String,
    pub bundle: WorldBundle,
}

/// Errors return as `PhaseError`; the caller runs `finalize_phase_error`.
pub(crate) struct NarrationGeneration<'p, 'a> {
    run: &'p PipelineRun<'a>,
    inputs: GenerationInputs,
}

impl<'p, 'a> NarrationGeneration<'p, 'a> {
    pub(crate) fn new(run: &'p PipelineRun<'a>, inputs: GenerationInputs) -> Self {
        Self { run, inputs }
    }

    pub(crate) fn run(&self, state: &mut GameState) -> Result<NarrationOutcome, PhaseError> {
        let started_for = self.run.started_for;
        let bundle = match self.run.pipeline.load_world_bundle(started_for) {
            Ok(bundle) => bundle,
            Err(e) => {
                tracing::error!("narration_generation: {e}");
                return Err(PhaseError::FetchFailed(e.to_string()));
            }
        };

        let Some(room) = Self::resolve_room(state, &bundle.map) else {
            return Err(self.run.set_error(state, "Room not found".to_string()));
        };

        let (preset, response_length) = match self.resolve_preset_choice() {
            Ok(p) => p,
            Err(msg) => return Err(self.run.set_error(state, msg)),
        };

        let (user_message, message_type) = match &self.inputs.impersonate {
            Some(impersonate) => (
                impersonate.steering_instruction.clone().unwrap_or_default(),
                MessageType::Input,
            ),
            None => (self.inputs.input.clone(), MessageType::Narration),
        };

        let history = state.narrative.history();
        let all_npcs: Vec<NpcCard> = bundle.npcs.values().cloned().collect();
        let context = PromptContext::new(
            &bundle.world,
            room,
            NpcContext {
                all_npcs: &all_npcs,
                npcs_in_area: &state.scene.npcs_in_area,
            },
            &bundle.persona,
            &user_message,
            &history,
        );
        let context = match &self.inputs.impersonate {
            Some(_) => context.with_impersonate(true),
            None => context.with_guide(self.inputs.guide.clone()),
        };

        let (narration_text, backend_name, model_name) =
            match self.run.call_narrator(&context, &preset, &response_length) {
                Ok(triple) => triple,
                Err(msg) => return Err(self.run.set_error(state, msg)),
            };

        self.run.check_game_unchanged(started_for)?;

        let (impersonated, steering_instruction) = Self::stored_inputs_from(&self.inputs);
        state.add_message_with_inputs(
            narration_text.clone(),
            message_type,
            impersonated,
            steering_instruction,
        );
        self.run
            .persist_snapshot_or_err(state, "pre-quantifier narration")?;

        Ok(NarrationOutcome {
            narration_text,
            backend_name,
            model_name,
            bundle,
        })
    }

    fn resolve_room<'s>(state: &'s GameState, map: &'s Arc<MapDef>) -> Option<&'s Room> {
        map.get_room_by_id(&state.movement.current_room_id)
            .or_else(|| {
                state
                    .movement
                    .dynamic_rooms
                    .get(&state.movement.current_room_id)
            })
    }

    fn resolve_preset_choice(&self) -> Result<(PromptPreset, String), String> {
        // First run and redo alike resolve the game's current active preset —
        // a preset is game configuration, not swipe data.
        let (preset_id, kind) = match self.inputs.impersonate.as_ref() {
            Some(_) => {
                let settings = self
                    .run
                    .pipeline
                    .settings
                    .read()
                    .unwrap_or_else(|e| e.into_inner());
                (
                    self.run
                        .pipeline
                        .storage
                        .active_impersonate_preset_id(&settings),
                    PresetKind::Impersonate,
                )
            }
            None => {
                let settings = self
                    .run
                    .pipeline
                    .settings
                    .read()
                    .unwrap_or_else(|e| e.into_inner());
                (
                    self.run.pipeline.storage.active_system_preset_id(&settings),
                    PresetKind::System,
                )
            }
        };
        self.run.load_preset_and_response_length(&preset_id, kind)
    }

    fn stored_inputs_from(inputs: &GenerationInputs) -> (bool, Option<String>) {
        match &inputs.impersonate {
            Some(impersonate) => (true, impersonate.steering_instruction.clone()),
            None => (false, inputs.guide.clone()),
        }
    }
}
