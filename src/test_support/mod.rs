//! Test fixtures + closure factories for the LLM recorder save seam.

pub mod context;
pub mod fixtures;
pub mod quantifier;
pub mod test_app_builder;
pub mod test_data_builder;

pub use context::{
    build_test_message_service, build_test_wired_app, build_test_wired_app_with_settings,
    make_test_app, make_test_app_without_snapshot, make_test_pipeline_app,
    make_test_pipeline_app_with_storage, make_test_pipeline_with_backends,
    make_test_pipeline_with_mock_quantifier, seed_default_impersonate_preset, seed_default_preset,
    seed_test_world_into_storage,
};
pub use fixtures::*;
pub use test_app_builder::TestAppBuilder;
pub use test_data_builder::{TestData, TestDataBuilder};
