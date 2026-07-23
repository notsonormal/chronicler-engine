//! Integration test binary root: wires shared helpers (`test_utils`, `fixtures`, `storage_ext`, `application_ext`) and re-exports factory helpers (`failing_service`, `working_service`, `SettingsTestGuard`) used by the application / storage / flow / model / adapter sub-suites.

#[path = "../test_utils/mod.rs"]
mod test_utils;

#[path = "../helpers/fixtures.rs"]
mod fixtures;

#[path = "../helpers/storage_ext.rs"]
mod storage_ext;

#[path = "../helpers/application_ext.rs"]
mod application_ext;

#[path = "../helpers/sqlite_test_app_builder.rs"]
mod sqlite_test_app_builder;

pub use sqlite_test_app_builder::SqliteTestAppBuilder;

use std::sync::Arc;

use chronicler_engine::application::game_service::GameService;
use chronicler_engine::application::agents::registry::AgentRegistry;
use chronicler_engine::adapters::driven::llm::providers::MockBackend;

pub use test_utils::settings_guard::SettingsTestGuard;
pub use test_utils::make_test_recorder;
pub use test_utils::make_test_recorder_with_storage;
pub use test_utils::server::get_available_port;

pub fn failing_service() -> GameService {
    let quantifier: Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default());
    GameService::with_mock_quantifier(
        make_test_recorder(Arc::new(MockBackend::default().with_fail())),
        quantifier,
    )
}

pub fn working_service() -> GameService {
    GameService::with_backends(
        make_test_recorder(Arc::new(MockBackend::default())),
        AgentRegistry::default(),
    )
}

mod model;

#[path = "bootstrap/run_branches.rs"]
mod bootstrap;

mod storage;

#[path = "application/application_service.rs"]
mod application_service;
#[path = "application/game_service.rs"]
mod game_service;
#[path = "application/lifecycle.rs"]
mod lifecycle;
#[path = "adapters/driven/llm/llm_client.rs"]
mod llm_client;

#[path = "flow/arrival_persistence.rs"]
mod flow_arrival_persistence;
#[path = "flow/retry_event.rs"]
mod flow_retry_event;
#[path = "flow/retry_main.rs"]
mod flow_retry_main;
#[path = "flow/sequence.rs"]
mod flow_sequence;
#[path = "application/action_pipeline/actions.rs"]
mod pipeline_actions;
#[path = "application/action_pipeline/retry.rs"]
mod pipeline_retry;
#[path = "application/action_pipeline/pipeline.rs"]
mod pipeline_tests;
