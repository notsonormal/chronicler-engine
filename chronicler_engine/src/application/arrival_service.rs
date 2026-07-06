//! [DOC: docs/system/startup.md]
//! Arrival narration use case — generates the opening scene when a player enters a room

use std::sync::Arc;

use crate::application::context::{self, OpContext};
use crate::application::narrative_prompt::{build_narration_prompt, make_prompt_context, NpcContext};
use crate::application::ports::llm_provider::AGENT_NARRATOR;
use crate::application::scenario::inject_scenario_logs;
use crate::domain::model::character::NpcCard;
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::GenerationStatus;
use crate::domain::model::state::message_types::MessageType;

pub struct ArrivalTaskContext {
    pub(crate) ctx: OpContext,
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
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub fn new_for_test(
        ctx: OpContext,
        room_id: String,
        nearby_npcs: Vec<NpcCard>,
        all_npcs: Vec<NpcCard>,
        arrival_preset: Option<PromptPreset>,
        response_length: String,
        max_context_tokens: u32,
        max_tokens: Option<u32>,
        recorder: Arc<crate::application::llm_recorder::LlmCallRecorder>,
    ) -> Self {
        Self {
            ctx,
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
    pub fn run_sync(self) {
        self.run();
    }

    pub(crate) fn run(self) {
        let mut state = match self.ctx.storage.load_latest_snapshot() {
            Ok(Some(snap)) => GameState::from_snapshot(
                &snap,
                Arc::clone(&self.ctx.world_snapshot.world),
                Arc::clone(&self.ctx.world_snapshot.map),
                Arc::clone(&self.ctx.world_snapshot.player),
                (*self.ctx.world_snapshot.npcs).clone(),
            ),
            _ => {
                tracing::warn!("No snapshot found in arrival task; starting fresh");
                let starting_room_id = self.ctx.world_snapshot.world.starting_room_id();
                let mut s = GameState::new(
                    Arc::clone(&self.ctx.world_snapshot.world),
                    Arc::clone(&self.ctx.world_snapshot.map),
                    Arc::clone(&self.ctx.world_snapshot.player),
                    (*self.ctx.world_snapshot.npcs).values().cloned().collect(),
                    starting_room_id,
                );
                inject_scenario_logs(
                    &mut s,
                    &self.ctx.world_snapshot.world,
                    &self.ctx.world_snapshot.player,
                );
                s
            }
        };
        if let Ok(msgs) = context::load_messages_with_swipes(&self.ctx.storage) {
            state.narrative.history.replace(msgs);
        }
        state.narrative.input_buffer.status = GenerationStatus::Generating;

        let room = match self
            .ctx
            .world_snapshot
            .map
            .overworld
            .regions
            .iter()
            .flat_map(|r| r.rooms.iter())
            .find(|r| r.id == self.room_id)
        {
            Some(r) => r,
            None => return,
        };

        let prompt_context = make_prompt_context(
            &self.ctx.world_snapshot.world,
            room,
            NpcContext {
                all_npcs: &self.all_npcs,
                npcs_in_area: &self.nearby_npcs,
            },
            &self.ctx.world_snapshot.player,
            "",
            &[],
        );

        let narration = match self.arrival_preset.as_ref() {
            Some(preset) => build_narration_prompt(
                &prompt_context,
                preset,
                &self.ctx.world_snapshot.world.global_rules,
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

        if let Err(e) = context::save_message_and_snapshot(&self.ctx, &mut state) {
            tracing::error!("Failed to save arrival message and snapshot: {e}");
        }
    }
}
