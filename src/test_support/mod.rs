//! Test fixtures + closure factories for the LLM recorder save seam.

// Fixtures deliberately ignore persistence results: their job is to arrange
// state, and a failed write surfaces in the test's own assertions.
#![allow(clippy::let_underscore_must_use)]

pub mod chat_completions_mock;
pub mod context;
pub mod env_guard;
pub mod fixtures;
pub mod http;
pub mod quantifier;
pub mod scripted_model;
pub mod test_app_builder;
pub mod test_data_builder;

pub use context::{
    configured_runner, configured_runner_without, make_test_job_deps,
    seed_default_impersonate_preset, seed_default_preset,
};
pub use fixtures::*;
pub use http::body_text;
pub use scripted_model::{analysis_report, ModelCall, ModelHold, ScriptedModel, STRAY_CALL_MARKER};
pub use test_app_builder::TestAppBuilder;
pub use test_data_builder::{TestData, TestDataBuilder};
