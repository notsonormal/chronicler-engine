// [DOC: docs/architecture/guardrails.md]
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

pub mod bootstrap;
pub mod cli;
pub mod engine;
pub mod error;
pub mod model;
pub mod narrative;
pub mod server;
pub mod settings;
pub mod storage;

pub use error::{EngineError, Result};

pub use model::settings::AppSettings;
pub use server::AppState;
pub use server::{create_app_for_testing, create_app_for_testing_with_settings};

pub mod test_support;

#[cfg(test)]
mod bootstrap_tests;
#[cfg(test)]
mod cli_tests;
#[cfg(test)]
mod error_tests;
#[cfg(test)]
mod settings_tests;
