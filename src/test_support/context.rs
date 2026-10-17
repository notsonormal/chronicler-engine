//! Builds `WiredApp` instances for integration tests.
#![allow(clippy::expect_used)]

use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::adapters::driven::llm::providers::MockBackend;
use crate::adapters::driven::storage::Storage;
use crate::adapters::driving::http::AppState;
use crate::application::agents::AgentRunner;
use crate::application::games::active::ActiveGame;
use crate::application::games::catalogue::GameCatalogue;
use crate::application::llm_recorder::LlmCallRecorder;
use crate::application::message_service::MessageService;
use crate::application::pipeline::ActionPipeline;
use crate::bootstrap::wiring::{WiredApp, build_app_graph_for_tests};
use crate::domain::model::agent::AgentConfig;
use crate::domain::model::character::NpcCard;
use crate::domain::model::game::GameId;
use crate::domain::model::prompt_preset::{PresetType, PromptPreset};
use crate::domain::model::settings::NarratorMode;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::utils::settings_defaults;
use crate::error::Result;
use crate::test_support::TestData;
use crate::test_support::{make_test_recorder, TestAppBuilder, TestDataBuilder};

/// Mirrors the production system presets' macro-bearing writing style, so
/// posture macros resolve in recorded prompts.
const FIXTURE_WRITING_STYLE: &str = "{{narrative_perspective}}-person limited perspective, focused on the player character.\n{{narrative_tense}} tense narrative prose.";

pub fn seed_default_preset(storage: &Storage) {
    storage
        .save_preset(&PromptPreset {
            id: "system_default".to_string(),
            name: "Default Test System".to_string(),
            role: Some("You are a test narrator.".to_string()),
            instructions: None,
            writing_style: Some(FIXTURE_WRITING_STYLE.to_string()),
            output_format: None,
            allowed_modes: vec![NarratorMode::Novel],
            is_default: true,
            preset_type: PresetType::System,
        })
        // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on storage failure"
        .expect("test setup: save_preset must succeed for default preset");

    // The default Interactive Fiction bundle selects this id; without it an
    // IF-mode game built on this fixture fails preset resolution.
    storage
        .save_preset(&PromptPreset {
            id: "system_if_default".to_string(),
            name: "Test Interactive Fiction".to_string(),
            role: Some("You are a test interactive fiction narrator.".to_string()),
            instructions: None,
            writing_style: Some(FIXTURE_WRITING_STYLE.to_string()),
            output_format: None,
            allowed_modes: vec![NarratorMode::InteractiveFiction],
            is_default: true,
            preset_type: PresetType::System,
        })
        // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on storage failure"
        .expect("test setup: save_preset must succeed for IF default preset");

    seed_default_impersonate_preset(storage);
}

pub fn seed_default_impersonate_preset(storage: &Storage) {
    storage
        .save_preset(&PromptPreset {
            id: "impersonate_default".to_string(),
            name: "Default Test Impersonate".to_string(),
            role: Some("You are writing as {{user}}. {{persona_description}}".to_string()),
            instructions: Some("Write only as {{user}}.".to_string()),
            writing_style: Some("First person as {{user}}.".to_string()),
            output_format: Some("Write {{user}}'s next message.".to_string()),
            allowed_modes: settings_defaults::default_allowed_modes(),
            is_default: true,
            preset_type: PresetType::Impersonate,
        })
        // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on storage failure"
        .expect("test setup: save_preset must succeed for default impersonate preset");
}

pub fn build_test_message_service(storage: Arc<Storage>) -> Arc<MessageService> {
    Arc::new(MessageService::new(storage))
}

pub fn configured_runner(recorder: Arc<LlmCallRecorder>) -> AgentRunner {
    runner_from_configs(&AgentConfig::defaults(), recorder)
}

/// The same wiring without the Agents of `agent_type`, as an `[agents]` section
/// with no entry for that type leaves them.
pub fn configured_runner_without(recorder: Arc<LlmCallRecorder>, agent_type: &str) -> AgentRunner {
    let configs: Vec<AgentConfig> = AgentConfig::defaults()
        .into_iter()
        .filter(|config| config.agent_type != agent_type)
        .collect();
    runner_from_configs(&configs, recorder)
}

fn runner_from_configs(configs: &[AgentConfig], recorder: Arc<LlmCallRecorder>) -> AgentRunner {
    AgentRunner::from_configs(configs, Arc::clone(&recorder), recorder)
        // arch-lint: allow(no-unwrap-expect) reason="test wiring fails on an unknown agent config"
        .expect("configured Agents build from the default configs")
}

pub fn make_test_pipeline_with_backends(
    storage: Arc<Storage>,
    recorder: Arc<LlmCallRecorder>,
    agent_runner: AgentRunner,
) -> ActionPipeline {
    seed_default_preset(&storage);
    let message_service = build_test_message_service(Arc::clone(&storage));
    let game_catalogue = Arc::new(GameCatalogue::new(
        Arc::clone(&storage),
        Arc::clone(&message_service),
        Arc::new(ActiveGame::new(Arc::clone(&storage))),
        None,
    ));
    ActionPipeline::with_backends(
        CancellationToken::new(),
        recorder,
        agent_runner,
        message_service,
        storage,
        game_catalogue,
    )
}

pub fn make_test_pipeline_with_mock_quantifier(
    storage: Arc<Storage>,
    recorder: Arc<LlmCallRecorder>,
    quantifier_provider: Arc<dyn crate::application::ports::llm_provider::LlmProvider>,
) -> ActionPipeline {
    seed_default_preset(&storage);
    let message_service = build_test_message_service(Arc::clone(&storage));
    let game_catalogue = Arc::new(GameCatalogue::new(
        Arc::clone(&storage),
        Arc::clone(&message_service),
        Arc::new(ActiveGame::new(Arc::clone(&storage))),
        None,
    ));
    ActionPipeline::with_mock_quantifier(
        CancellationToken::new(),
        recorder,
        quantifier_provider,
        message_service,
        storage,
        game_catalogue,
    )
}

pub fn build_test_wired_app(
    storage: Arc<Storage>,
    pipeline: ActionPipeline,
    game_id: GameId,
) -> Result<WiredApp> {
    build_app_graph_for_tests(storage, Some(pipeline), Some(game_id))
}

pub fn seed_test_world_into_storage(storage: &Storage, state: &GameState) -> GameId {
    let data = TestData {
        world: Arc::new(crate::test_support::fixtures::TestWorld::minimal()),
        map: Arc::new(crate::test_support::fixtures::TestMap::single_room(
            &state.swipe.movement.current_room_id,
        )),
        persona: Arc::new(crate::test_support::fixtures::TestPersona::standard()),
        npcs: std::iter::empty::<NpcCard>().collect(),
        room_npcs: Vec::new(),
    };
    data.seed_into(storage)
}

pub fn make_test_pipeline_app() -> AppState {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_runner = AgentRunner::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_runner,
    );
    TestAppBuilder::with_data(data)
        .pipeline(service)
        .build_service()
}

pub fn make_test_pipeline_app_with_storage() -> (AppState, Arc<Storage>) {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_runner = AgentRunner::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_runner,
    );
    TestAppBuilder::with_data(data)
        .pipeline(service)
        .build_service_with_storage()
}

fn build_test_app(storage: Arc<Storage>, game_id: GameId) -> Result<WiredApp> {
    seed_default_preset(&storage);
    build_app_graph_for_tests(storage, None, Some(game_id))
}

pub fn make_test_app(state: GameState) -> Result<WiredApp> {
    let snapshot = GameStateSnapshot::from_game_state(&state);
    let storage = Arc::new(Storage::new_in_memory());
    let game_id = seed_test_world_into_storage(&storage, &state);
    storage
        .save_snapshot(game_id, &snapshot)
        // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on storage failure"
        .expect("test setup: save_snapshot must succeed for make_test_app fixture");
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        let id = storage
            .insert_message(game_id, &msg)
            // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on storage failure"
            .expect("test setup: insert_message must succeed to seed narrative history");
        for (idx, swipe) in msg.swipes.iter().enumerate() {
            storage
                .insert_swipe(id, swipe, idx)
                // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on storage failure"
                .expect("test setup: insert_swipe must succeed to seed narrative swipes");
        }
    }
    build_test_app(storage, game_id)
}

pub fn make_test_app_without_snapshot(state: GameState) -> Result<WiredApp> {
    let storage = Arc::new(Storage::new_in_memory());
    let game_id = seed_test_world_into_storage(&storage, &state);
    for msg in state.narrative.history.iter().cloned().collect::<Vec<_>>() {
        let id = storage
            .insert_message(game_id, &msg)
            // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on storage failure"
            .expect("test setup: insert_message must succeed to seed narrative history");
        for (idx, swipe) in msg.swipes.iter().enumerate() {
            storage
                .insert_swipe(id, swipe, idx)
                // arch-lint: allow(no-unwrap-expect) reason="test setup fixture panics on storage failure"
                .expect("test setup: insert_swipe must succeed to seed narrative swipes");
        }
    }
    build_test_app(storage, game_id)
}
