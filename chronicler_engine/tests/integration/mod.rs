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

pub use test_utils::settings_guard::SettingsTestGuard;
pub use test_utils::make_test_recorder;
pub use test_utils::make_test_recorder_with_storage;
pub use test_utils::server::get_available_port;

mod model;

#[path = "bootstrap/run_branches.rs"]
mod bootstrap;

mod storage;

#[path = "adapters/driven/llm/llm_client.rs"]
mod llm_client;
