//! Builds `WiredApp` instances for integration tests.
#![allow(clippy::expect_used)]

use std::sync::Arc;
use std::sync::RwLock;

use tokio_util::sync::CancellationToken;

use crate::adapters::driven::storage::Storage;
use crate::application::agents::registry::AgentRegistry;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::pipeline::ActionPipeline;
use crate::application::message_service::MessageService;
use crate::bootstrap::wiring::{WiredApp, build_app_graph_for_tests};
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::character::NpcCard;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::settings::AppSettings;
use crate::error::Result;
use crate::test_support::TestData;

pub fn seed_default_preset(storage: &Storage) {
    storage
        .save_preset(&PromptPreset {
            id: "system_default".to_string(),
            name: "Default Test System".to_string(),
            role: Some("You are a test narrator.".to_string()),
            instructions: None,
            writing_style: None,
            output_format: None,
            is_default: true,
            preset_type: PresetType::System,
        })
        // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on storage failure"
        .expect("test setup: save_preset must succeed for default preset");
}

pub fn build_test_message_service(storage: Arc<Storage>) -> Arc<MessageService> {
    Arc::new(MessageService::new(storage))
}

pub fn make_test_pipeline_with_backends(
    storage: Arc<Storage>,
    recorder: Arc<LlmCallRecorder>,
    agent_registry: AgentRegistry,
) -> ActionPipeline {
    seed_default_preset(&storage);
    let message_service = build_test_message_service(Arc::clone(&storage));
    let settings = Arc::new(RwLock::new(AppSettings::default()));
    ActionPipeline::with_backends(
        CancellationToken::new(),
        recorder,
        agent_registry,
        message_service,
        Arc::clone(&storage),
        settings,
    )
}

pub fn make_test_pipeline_with_mock_quantifier(
    storage: Arc<Storage>,
    recorder: Arc<LlmCallRecorder>,
    quantifier_provider: Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
) -> ActionPipeline {
    seed_default_preset(&storage);
    let message_service = build_test_message_service(Arc::clone(&storage));
    let settings = Arc::new(RwLock::new(AppSettings::default()));
    ActionPipeline::with_mock_quantifier(
        CancellationToken::new(),
        recorder,
        quantifier_provider,
        message_service,
        Arc::clone(&storage),
        settings,
    )
}

/// Build a full `WiredApp` for the supplied pipeline.
pub fn build_test_wired_app(storage: Arc<Storage>, pipeline: ActionPipeline) -> Result<WiredApp> {
    build_app_graph_for_tests(
        Arc::new(RwLock::new(AppSettings::default())),
        storage,
        Some(pipeline),
    )
}

/// Build a full `WiredApp` with custom settings.
pub fn build_test_wired_app_with_settings(
    storage: Arc<Storage>,
    settings: Arc<RwLock<AppSettings>>,
    pipeline: ActionPipeline,
) -> Result<WiredApp> {
    build_app_graph_for_tests(settings, storage, Some(pipeline))
}

pub fn seed_test_world_into_storage(storage: &Storage, state: &GameState) {
    let data = TestData {
        world: Arc::new(crate::test_support::fixtures::TestWorld::minimal()),
        map: Arc::new(crate::test_support::fixtures::TestMap::single_room(
            &state.movement.current_room_id,
        )),
        persona: Arc::new(crate::test_support::fixtures::TestPersona::standard()),
        npcs: std::iter::empty::<NpcCard>().collect(),
        room_npcs: Vec::new(),
    };
    let _ = data.seed_into(storage);
}

fn build_test_app(storage: Arc<Storage>) -> Result<WiredApp> {
    seed_default_preset(&storage);
    let settings = Arc::new(RwLock::new(AppSettings::default()));
    build_app_graph_for_tests(settings, storage, None)
}

pub fn make_test_app(state: GameState) -> Result<WiredApp> {
    let snapshot = GameStateSnapshot::from_game_state(&state);
    let storage = Arc::new(Storage::new_in_memory());
    seed_test_world_into_storage(&storage, &state);
    storage
        .save_snapshot(&snapshot)
        // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on storage failure"
        .expect("test setup: save_snapshot must succeed for make_test_app fixture");
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        let id = storage
            .insert_message(&msg)
            // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on storage failure"
            .expect("test setup: insert_message must succeed to seed narrative history");
        for (idx, swipe) in msg.swipes.iter().enumerate() {
            storage
                .insert_swipe(id, swipe, idx)
                // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on storage failure"
                .expect("test setup: insert_swipe must succeed to seed narrative swipes");
        }
    }
    build_test_app(storage)
}

pub fn make_test_app_without_snapshot(state: GameState) -> Result<WiredApp> {
    let storage = Arc::new(Storage::new_in_memory());
    seed_test_world_into_storage(&storage, &state);
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        let id = storage
            .insert_message(&msg)
            // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on storage failure"
            .expect("test setup: insert_message must succeed to seed narrative history");
        for (idx, swipe) in msg.swipes.iter().enumerate() {
            storage
                .insert_swipe(id, swipe, idx)
                // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on storage failure"
                .expect("test setup: insert_swipe must succeed to seed narrative swipes");
        }
    }
    build_test_app(storage)
}
