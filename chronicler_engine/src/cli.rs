use clap::Parser;

/// [DOC: docs/architecture/system.md]
#[derive(Parser, Debug)]
#[command(name = "chronicler-engine")]
#[command(version = "0.1.0")]
#[command(about = "Text adventure engine with HTMX dashboard")]
pub struct Args {
    /// Specify which world to load
    #[arg(long, default_value = "redmist_estate")]
    pub world: String,

    /// List all available worlds and exit
    #[arg(long)]
    pub list_worlds: bool,

    /// Port to run the HTTP server on
    #[arg(long, default_value = "3000")]
    pub port: u16,
}

/// Parse command-line arguments.
pub fn parse_args() -> Args {
    Args::parse()
}
