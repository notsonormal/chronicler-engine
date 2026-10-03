//! [DOC: docs/diataxis/reference/startup.md]
//! Bootstrap initialization and startup sequences
#![allow(clippy::print_stdout, clippy::print_stderr)]

pub mod init_game;
mod load;
mod logging;
mod run;
pub mod wiring;

pub use logging::init_logging;
pub use run::run;
#[cfg(test)]
mod load_tests;
#[cfg(test)]
mod run_tests;
#[cfg(test)]
mod wiring_tests;
