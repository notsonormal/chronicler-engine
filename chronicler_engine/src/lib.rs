// [DOC: chronicler_engine/docs/diataxis/reference/coding_standards/guardrails.md]
// AI Guardrails: These lint attributes enforce the project's coding standards
// at compile time. Any violation will fail the build.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::dbg_macro,
    clippy::todo,
    clippy::unimplemented,
    clippy::print_stdout,
    clippy::print_stderr,
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
pub mod settings;

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
mod cli_tests;
#[cfg(test)]
mod error_tests;
#[cfg(test)]
mod settings_tests;
