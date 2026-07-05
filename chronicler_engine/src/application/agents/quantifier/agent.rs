//! [DOC: docs/system/agent_system.md]
//! Quantifier agent implementation
//! arch-lint: storage-direct — deferred to T2, see ADR-027

use std::sync::{Arc, RwLock};

use crate::error::EngineError;
use crate::domain::model::agent::{
    AgentConfig, AgentContext, AgentResult, BackendSelector, Confidence, ExecutionPhase, StatePatch,
};
use crate::domain::model::settings::AppSettings;

use crate::application::agents::Agent;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::narrative_prompt::assembler::assemble_prompt_text;
#[cfg(feature = "testing")]
use crate::application::ports::llm_provider::LlmProvider;
use crate::adapters::driven::storage::Storage;

use super::determine_npcs_in_room;

pub struct QuantifierAgent {
    name: String,
    recorder: Arc<LlmCallRecorder>,
    preset_storage: Option<Arc<Storage>>,
    settings: Arc<RwLock<AppSettings>>,
}

impl std::fmt::Debug for QuantifierAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuantifierAgent")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl QuantifierAgent {
    pub fn from_config_with_storage(
        _config: &AgentConfig,
        recorder: Arc<LlmCallRecorder>,
        preset_storage: Option<Arc<Storage>>,
        settings: Arc<RwLock<AppSettings>>,
    ) -> Result<Self, EngineError> {
        Ok(Self {
            name: "quantifier".to_string(),
            recorder,
            preset_storage,
            settings: Arc::clone(&settings),
        })
    }

    #[cfg(feature = "testing")]
    pub fn with_provider(name: String, provider: Arc<dyn LlmProvider>) -> Self {
        use crate::test_support::noop_forensics::NoopForensics;
        Self {
            name,
            recorder: Arc::new(LlmCallRecorder::new(provider, Arc::new(NoopForensics))),
            preset_storage: None,
            settings: Arc::new(RwLock::new(AppSettings::default())),
        }
    }
}

impl Agent for QuantifierAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn phase(&self) -> ExecutionPhase {
        ExecutionPhase::PostGeneration
    }

    fn backend_selector(&self) -> BackendSelector {
        BackendSelector::UseNamed("quantifier".to_string())
    }

    fn execute(&self, ctx: &AgentContext) -> Result<AgentResult, EngineError> {
        let main_response = ctx
            .main_response
            .ok_or_else(|| EngineError::Config("Quantifier requires main_response".into()))?;

        let state = ctx.state;
        let previous_room_npcs: Vec<_> = state.scene.npcs_in_area.clone();

        let current_room = ctx
            .current_room
            .ok_or_else(|| EngineError::RoomNotFound("current room not found".to_string()))?;

        let quantifier_prompt_override = {
            let settings = self.settings.read().unwrap_or_else(|e| e.into_inner());
            self.preset_storage
                .as_ref()
                .and_then(|s| {
                    s.get_preset(&settings.active_quantifier_prompt_preset_id)
                        .ok()
                        .flatten()
                })
                .map(|preset| assemble_prompt_text(&preset, &[], None))
        };

        let result = determine_npcs_in_room(
            state,
            current_room,
            &[],
            &previous_room_npcs,
            main_response,
            self.recorder.as_ref(),
            quantifier_prompt_override,
        );

        let confidence = Confidence::from(result.npcs.confidence);

        Ok(AgentResult::StatePatch(StatePatch {
            npc_ids: result.npcs.npc_ids,
            movement_destination: result.movement.destination,
            confidence,
        }))
    }
}
