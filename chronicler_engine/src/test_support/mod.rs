//! Test fixtures + closure factories for the LLM recorder save seam.

pub mod context;
pub mod fixtures;
pub mod quantifier;
pub mod test_app_builder;
pub mod test_data_builder;

pub use context::{
    default_test_preset_storage, make_test_app, make_test_app_without_snapshot,
    seed_test_world_into_storage,
};
pub use fixtures::*;
pub use test_app_builder::TestAppBuilder;
pub use test_data_builder::{TestData, TestDataBuilder};
