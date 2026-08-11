//! [DOC: docs/diataxis/reference/startup.md]
//! Bootstrap initialization and startup sequences
#![allow(clippy::print_stdout, clippy::print_stderr)]

pub mod init_game;
mod load;
mod logging;
mod run;
mod validate;
pub mod wiring;

pub use logging::init_logging;
pub use run::run;
pub use validate::validate_loaded_data;
#[cfg(test)]
mod load_tests;
#[cfg(test)]
mod run_tests;
#[cfg(test)]
mod validate_tests;
#[cfg(test)]
mod wiring_tests;
