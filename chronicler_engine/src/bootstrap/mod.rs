// Bootstrap module is allowed to use stdout/stderr for CLI output.
#![allow(clippy::print_stdout, clippy::print_stderr)]

mod load;
mod logging;
mod run;
mod scenario;
mod state;
mod validate;
pub use load::{initialize_world_from_manifest, load_world_manifest};
pub use logging::init_logging;
pub use run::run;
pub use scenario::inject_scenario_logs;
pub use state::build_fresh_initial_state;
pub use validate::validate_loaded_data;
#[cfg(test)]
mod run_tests;
