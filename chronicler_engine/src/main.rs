// Binary entry point is allowed to use stdout/stderr for CLI output
// and expect for fatal bootstrap errors.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    clippy::print_stderr
)]

use chronicler_engine::bootstrap::{init_logging, run};
use chronicler_engine::utils::cli::parse_args;

fn main() -> chronicler_engine::Result<()> {
    dotenv::dotenv().ok();
    let _guard = init_logging();
    let args = parse_args();
    run(args)
}
