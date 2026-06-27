#[path = "../helpers/pipeline_helpers.rs"]
mod pipeline_helpers;

#[path = "../helpers/fixtures.rs"]
mod fixtures;

use std::sync::Arc;

use chronicler_engine::application::game_service::GameService;
use chronicler_engine::narrative::agents::registry::AgentRegistry;
use chronicler_engine::narrative::llm::MockBackend;

#[path = "../test_utils/settings_guard.rs"]
mod settings_guard;
pub use settings_guard::SettingsTestGuard;

pub fn failing_service() -> GameService {
    GameService::with_mock_quantifier(
        Arc::new(MockBackend::failing()),
        Arc::new(MockBackend::default()),
    )
}

pub fn working_service() -> GameService {
    GameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default())
}

mod application_service;
mod game_service;
mod lifecycle;
mod llm_client;
mod model;
mod storage;

#[path = "flow/arrival_persistence_tests.rs"]
mod flow_arrival_persistence_tests;
#[path = "flow/retry_event.rs"]
mod flow_retry_event;
#[path = "flow/retry_main.rs"]
mod flow_retry_main;
#[path = "flow/sequence.rs"]
mod flow_sequence;
#[path = "pipeline/actions.rs"]
mod pipeline_actions;
#[path = "pipeline/retry.rs"]
mod pipeline_retry;
#[path = "pipeline/pipeline.rs"]
mod pipeline_tests;
