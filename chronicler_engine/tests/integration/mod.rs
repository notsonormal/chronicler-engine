//! Integration test binary root.

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

use chronicler_engine::application::agents::registry::AgentRegistry;
use chronicler_engine::application::pipeline::pipeline::ActionPipeline;
use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driven::storage::Storage;

pub use test_utils::settings_guard::SettingsTestGuard;
pub use test_utils::make_test_recorder;
pub use test_utils::make_test_recorder_with_storage;
pub use test_utils::server::get_available_port;

pub fn failing_pipeline() -> ActionPipeline {
    let storage = Arc::new(Storage::new_in_memory());
    let quantifier: Arc<dyn chronicler_engine::application::ports::llm_provider::LlmProvider> =
        Arc::new(MockBackend::default());
    chronicler_engine::test_support::make_test_pipeline_with_mock_quantifier(
        storage,
        make_test_recorder(Arc::new(MockBackend::default().with_fail())),
        quantifier,
    )
}

pub fn working_pipeline() -> ActionPipeline {
    let storage = Arc::new(Storage::new_in_memory());
    chronicler_engine::test_support::make_test_pipeline_with_backends(
        storage,
        make_test_recorder(Arc::new(MockBackend::default())),
        AgentRegistry::default(),
    )
}

mod model;

#[path = "bootstrap/run_branches.rs"]
mod bootstrap;

mod storage;

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
#[path = "application/action_pipeline/retry.rs"]
mod pipeline_retry;
