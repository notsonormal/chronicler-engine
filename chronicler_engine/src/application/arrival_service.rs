//! [DOC: chronicler_engine/docs/diataxis/reference/startup.md]
//! Arrival narration use case — generates the opening scene when a player enters a room

use std::sync::Arc;

use tracing::instrument;

use crate::application::persistence_gate::PersistenceGate;
use crate::application::prompting::{NpcContext, PromptContext};
use crate::application::ports::llm_provider::AGENT_NARRATOR;
use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::map::MapDef;
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::state::generation_status::GenerationStatus;
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::world::WorldCard;
use crate::error::EngineError;

pub struct ArrivalTaskContext {
    pub(crate) persistence_gate: Arc<PersistenceGate>,
    pub(crate) room_id: String,
    pub(crate) arrival_preset: Option<PromptPreset>,
    pub(crate) response_length: String,
    pub(crate) max_context_tokens: u32,
    pub(crate) max_tokens: Option<u32>,
    pub(crate) nearby_npcs: Vec<NpcCard>,
    pub(crate) all_npcs: Vec<NpcCard>,
    pub(crate) recorder: Arc<crate::application::llm_recorder::LlmCallRecorder>,
}

impl ArrivalTaskContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        persistence_gate: Arc<PersistenceGate>,
        room_id: String,
        arrival_preset: Option<PromptPreset>,
        response_length: String,
        max_context_tokens: u32,
        max_tokens: Option<u32>,
        nearby_npcs: Vec<NpcCard>,
        all_npcs: Vec<NpcCard>,
        recorder: Arc<crate::application::llm_recorder::LlmCallRecorder>,
    ) -> Self {
        Self {
            persistence_gate,
            room_id,
            arrival_preset,
            response_length,
            max_context_tokens,
            max_tokens,
            nearby_npcs,
            all_npcs,
            recorder,
        }
    }

    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub fn new_for_test(
        persistence_gate: Arc<PersistenceGate>,
        room_id: String,
        nearby_npcs: Vec<NpcCard>,
        all_npcs: Vec<NpcCard>,
        arrival_preset: Option<PromptPreset>,
        response_length: String,
        max_context_tokens: u32,
        max_tokens: Option<u32>,
        recorder: Arc<crate::application::llm_recorder::LlmCallRecorder>,
    ) -> Self {
        Self::new(
            persistence_gate,
            room_id,
            arrival_preset,
            response_length,
            max_context_tokens,
            max_tokens,
            nearby_npcs,
            all_npcs,
            recorder,
        )
    }

    #[doc(hidden)]
    pub fn run_sync(self) {
        let _ = self.run();
    }

    #[instrument(err, skip(self), fields(room_id = %self.room_id))]
    pub(crate) fn run(self) -> Result<(), EngineError> {
        let storage = self.persistence_gate.storage();
        let mut state = match self.persistence_gate.load_expecting_valid_state() {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(
                    "load_expecting_valid_state failed in arrival task: {e}; falling back to fresh initial state"
                );
                match self.persistence_gate.build_fresh_initial_state() {
                    Ok(s) => s,
                    Err(e2) => {
                        tracing::error!("build_fresh_initial_state also failed: {e2}");
                        return Err(e2);
                    }
                }
            }
        };

        let current_game_id = storage.current_game_id();
        let game = storage.require_game(current_game_id)?;
        let world_with_map = storage.require_world(&game.world_key)?;
        let persona: Arc<PersonaCard> = Arc::new(storage.require_persona(&game.persona_key)?);
        let world: Arc<WorldCard> = Arc::new(world_with_map.world_card);
        let map: Arc<MapDef> = Arc::new(world_with_map.map);
        if state.narrative.history.is_empty() {
            state.inject_scenario_logs(&world, &persona, &map);
        }
        state.narrative.input_buffer.status = GenerationStatus::Generating;

        let Some(room) = map
            .overworld
            .regions
            .iter()
            .flat_map(|r| r.rooms.iter())
            .find(|r| r.id == self.room_id)
        else {
            return Ok(());
        };

        let prompt_context = PromptContext::new(
            &world,
            room,
            NpcContext {
                all_npcs: &self.all_npcs,
                npcs_in_area: &self.nearby_npcs,
            },
            &persona,
            "",
            &[],
        );

        let global_rules = &world.global_rules;
        let narration = match self.arrival_preset.as_ref() {
            Some(preset) => prompt_context
                .build_narration_prompt(
                    preset,
                    global_rules,
                    Some(&self.response_length),
                    self.max_context_tokens,
                    self.max_tokens,
                )
                .and_then(|assembled| {
                    self.recorder.complete(
                        AGENT_NARRATOR,
                        &assembled.system_prompt,
                        &assembled.user_prompt,
                        Some(assembled.max_tokens),
                    )
                }),
            None => Err(crate::error::EngineError::Config(
                "No active preset found for arrival narration".into(),
            )),
        };

        match narration {
            Ok(result) => {
                state.add_message(result.text, None, MessageType::Narration);
                state.narrative.input_buffer.status = GenerationStatus::Idle;
            }
            Err(e) => {
                state.narrative.input_buffer.status =
                    GenerationStatus::Error(format!("LLM Error: {e}"));
            }
        }

        if let Err(e) = self.persistence_gate.save_message_and_snapshot(&mut state) {
            tracing::error!("Failed to save arrival message and snapshot: {e}");
        }

        Ok(())
    }
}
