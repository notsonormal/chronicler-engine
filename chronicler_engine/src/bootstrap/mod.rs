//! [DOC: docs/system/startup.md]
//! Bootstrap initialization and startup sequences
#![allow(clippy::print_stdout, clippy::print_stderr)]

pub mod init_game;
pub mod llm_factory;
mod load;
mod logging;
mod run;
pub mod text_check_factory;
mod validate;
pub mod wiring;

pub use logging::init_logging;
pub use run::run;
pub use validate::validate_loaded_data;
#[cfg(test)]
mod llm_factory_tests;
#[cfg(test)]
mod load_tests;
#[cfg(test)]
mod run_tests;
#[cfg(test)]
mod text_check_factory_tests;
#[cfg(test)]
mod validate_tests;
