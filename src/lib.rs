// [DOC: docs/diataxis/reference/coding_standards/guardrails.md]
// AI Guardrails: These lint attributes enforce the project's coding standards
// at compile time. Any violation will fail the build.
#![deny(
    // Prevent panics in production; propagate errors with `?`.
    clippy::unwrap_used,
    // No "should never happen" assumptions in production.
    clippy::expect_used,
    // No debug prints in committed code.
    clippy::dbg_macro,
    // No unfinished code in production.
    clippy::todo,
    // No stubs in production.
    clippy::unimplemented,
    // No `println!` in library code.
    clippy::print_stdout,
    // No `eprintln!` in library code.
    clippy::print_stderr,
    // No explicit panics in production.
    clippy::panic
)]
// Tests are allowed to panic on assertion failures — that's their purpose.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::print_stdout,
        clippy::print_stderr
    )
)]

pub mod adapters;
pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod error;
pub mod utils;

pub use error::{EngineError, Result};

pub use domain::model::settings::AppSettings;
pub use adapters::driving::http::AppState;

#[cfg(feature = "testing")]
pub use test_support::test_app_builder::TestAppBuilder;
#[cfg(feature = "testing")]
pub use test_support::test_data_builder::{TestData, TestDataBuilder};

#[cfg(feature = "testing")]
pub mod test_support;

#[cfg(test)]
mod error_tests;
