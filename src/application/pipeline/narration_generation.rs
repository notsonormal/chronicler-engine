//! [DOC: docs/diataxis/reference/game_flow.md]
//! Narration generation — the narrate-and-persist prefix of one generation.

use std::sync::Arc;

use crate::adapters::driven::storage::worlds::WorldBundle;
use crate::application::prompting::{NpcContext, PromptContext};
use crate::domain::model::agent::Role;
use crate::domain::model::character::NpcCard;
use crate::domain::model::map::{MapDef, Room};
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::{GenerationFailure, GenerationFailureKind};
use crate::domain::model::state::message_types::MessageType;

use super::phase_error::PhaseError;
use super::pipeline_run::{PipelineRun, PresetKind};

pub(crate) struct GenerationInputs {
    /// Empty for continues and slash-command generations; the entry path
    /// persists free actions.
    pub input: String,
    pub guide: Option<String>,
    pub impersonate: Option<ImpersonateInputs>,
}

#[derive(Debug, Clone)]
pub(crate) struct ImpersonateInputs {
    pub direction: Option<String>,
}

/// The tail (quantifier → game rules → trigger) runs on the same bundle
/// the narration was generated from, so the loaded bundle rides on the outcome.
#[derive(Debug)]
pub(crate) struct NarrationOutcome {
    pub narration_text: String,
    pub backend_name: String,
    pub model_name: String,
    pub bundle: WorldBundle,
}

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
                return Err(PhaseError::FetchFailed(
                    GenerationFailure::from_engine_error(&e),
                ));
            }
        };

        let Some(room) = Self::resolve_room(state, &bundle.map) else {
            // The line names the room, so its raw text is the room id.
            return Err(self.run.set_error(
                state,
                GenerationFailure::new(
                    GenerationFailureKind::SceneMissing,
                    state.swipe.movement.current_room_id.clone(),
                ),
            ));
        };

        let (preset, response_length) = match self.resolve_preset_choice() {
            Ok(p) => p,
            Err(failure) => return Err(self.run.set_error(state, failure)),
        };

        let (user_message, message_type) = match &self.inputs.impersonate {
            Some(impersonate) => (
                impersonate.direction.clone().unwrap_or_default(),
                MessageType::Input,
            ),
            None => (self.inputs.input.clone(), MessageType::Narration),
        };

        let history = state.narrative.history();
        let all_npcs: Vec<NpcCard> = bundle.npcs.values().cloned().collect();
        let (narrative_perspective, narrative_tense) = self.run.resolve_posture(&bundle.world);
        let context = PromptContext::new(
            &bundle.world,
            narrative_perspective,
            narrative_tense,
            room,
            NpcContext {
                all_npcs: &all_npcs,
                npcs_in_area: &state.swipe.scene.npcs_in_area,
            },
            &bundle.persona,
            &user_message,
            &history,
        );
        let context = match &self.inputs.impersonate {
            Some(_) => context.with_impersonate(true),
            None => context.with_guide(self.inputs.guide.clone()),
        };

        let budget = match self.run.narration_budget() {
            Ok(budget) => budget,
            Err(failure) => return Err(self.run.set_error(state, failure)),
        };
        let narrated = match self.run.pipeline.narrator.narrate(
            Role::Narrator,
            &context,
            &preset,
            &response_length,
            budget,
        ) {
            Ok(narrated) => narrated,
            Err(failure) => return Err(self.run.set_error(state, failure)),
        };
        let narration_text = narrated.text;

        self.run.check_game_unchanged(started_for)?;

        let (impersonated, guide_text, impersonate_direction) =
            Self::stored_inputs_from(&self.inputs);
        state.add_message_with_inputs(
            narration_text.clone(),
            message_type,
            impersonated,
            guide_text,
            impersonate_direction,
        );
        self.run
            .persist_snapshot_or_err(state, "pre-quantifier narration")?;

        Ok(NarrationOutcome {
            narration_text,
            backend_name: narrated.backend_name,
            model_name: narrated.model_name,
            bundle,
        })
    }

    fn resolve_room<'s>(state: &'s GameState, map: &'s Arc<MapDef>) -> Option<&'s Room> {
        map.get_room_by_id(&state.swipe.movement.current_room_id)
            .or_else(|| {
                state
                    .swipe
                    .movement
                    .dynamic_rooms
                    .get(&state.swipe.movement.current_room_id)
            })
    }

    fn resolve_preset_choice(&self) -> Result<(PromptPreset, String), GenerationFailure> {
        // A redo resolves the game's current active preset, as a first run does:
        // a preset is game configuration, not swipe data.
        let settings = self.run.pipeline.storage.get_settings().map_err(|e| {
            GenerationFailure::new(
                GenerationFailureKind::Other,
                format!("Failed to read settings: {e}"),
            )
        })?;
        let (preset_id, kind) = match self.inputs.impersonate.as_ref() {
            Some(_) => (
                self.run
                    .pipeline
                    .storage
                    .active_impersonate_preset_id(self.run.started_for, &settings),
                PresetKind::Impersonate,
            ),
            None => (
                self.run
                    .pipeline
                    .storage
                    .active_system_preset_id(self.run.started_for, &settings),
                PresetKind::System,
            ),
        };
        self.run.load_preset_and_response_length(&preset_id, kind)
    }

    fn stored_inputs_from(inputs: &GenerationInputs) -> (bool, Option<String>, Option<String>) {
        match &inputs.impersonate {
            Some(impersonate) => (true, None, impersonate.direction.clone()),
            None => (false, inputs.guide.clone(), None),
        }
    }
}
