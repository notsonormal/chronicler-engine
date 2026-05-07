// CLI module is allowed to use stdout/stderr for CLI output.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::{fs, path::{Path, PathBuf}};

use clap::Parser;

use crate::error::EngineError;
use crate::model::world::WorldManifest;

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

pub fn parse_args() -> Args {
    Args::parse()
}

/// [DOC: docs/architecture/system.md]
pub fn resolve_engine_data_path() -> PathBuf {
    // [DOC: docs/system/startup.md]
    if let Ok(data_dir) = std::env::var("CHRONICLER_DATA") {
        return PathBuf::from(data_dir);
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let data_dir = exe_dir.join("data");
            if data_dir.exists() {
                return data_dir;
            }
        }
    }

    PathBuf::from("data")
}

/// [DOC: docs/architecture/system.md]
/// Scan the given data directory for available worlds.
/// Returns a list of (id, name) tuples for each valid world found.
pub fn scan_worlds(data_dir: &Path) -> crate::error::Result<Vec<(String, String)>> {
    let worlds_dir = data_dir.join("worlds");
    if !worlds_dir.exists() {
        return Ok(vec![]);
    }

    let mut worlds = Vec::new();
    for entry in fs::read_dir(&worlds_dir)
        .map_err(|e| EngineError::Io(format!("read_dir {}: {e}", worlds_dir.display())))?
    {
        let entry = entry.map_err(|e| {
            EngineError::Io(format!("read_dir_entry {}: {e}", worlds_dir.display()))
        })?;
        let path = entry.path();
        if path.is_dir() {
            let world_file = path.join("world.json");
            if world_file.exists() {
                if let Ok(json) = fs::read_to_string(&world_file) {
                    if let Ok(manifest) = serde_json::from_str::<WorldManifest>(&json) {
                        worlds.push((manifest.id.clone(), manifest.name.clone()));
                    }
                }
            }
        }
    }

    Ok(worlds)
}

/// [DOC: docs/architecture/system.md]
pub fn list_available_worlds() -> crate::error::Result<()> {
    let worlds = scan_worlds(&resolve_engine_data_path())?;

    if worlds.is_empty() {
        println!("No worlds found in data/worlds/");
    } else {
        println!("Available worlds:");
        for (id, name) in &worlds {
            println!("  {id} - {name}");
        }
    }

    Ok(())
}
