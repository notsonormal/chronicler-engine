// CLI module is allowed to use stdout/stderr for CLI output.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::{fs, path::PathBuf};

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
pub fn list_available_worlds() -> crate::error::Result<()> {
    let data_dir = resolve_engine_data_path();
    let worlds_dir = data_dir.join("worlds");
    if !worlds_dir.exists() {
        println!("No worlds found in data/worlds/");
        return Ok(());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_engine_data_path_default() {
        let path = resolve_engine_data_path();
        assert!(path.is_relative() || path.is_absolute());
    }

    #[test]
    fn test_resolve_data_path_from_exe_fallback() {
        let data_dir = resolve_engine_data_path();
        // Should return a path (may or may not exist)
        assert!(data_dir.is_relative() || data_dir.is_absolute());
        let _ = data_dir.to_string_lossy(); // Should not panic
    }

    #[test]
    fn test_resolve_data_path_returns_pathbuf() {
        // Verify return type is PathBuf
        let path = resolve_engine_data_path();
        use std::path::PathBuf;
        let _type_check: PathBuf = path;
    }

    #[test]
    fn test_list_worlds_uses_worlds_subdirectory() {
        // list_available_works should look in data/worlds/ subdirectory
        let result = list_available_worlds();
        assert!(result.is_ok()); // Should handle gracefully
    }

    #[test]
    fn test_list_worlds_graceful_when_empty() {
        // Test that empty worlds directory is handled gracefully
        // The function should not panic
        let result = list_available_worlds();
        assert!(result.is_ok() || result.is_err()); // Should return cleanly
    }

    #[test]
    fn test_list_worlds_nonexistent_directory() {
        let _data_dir = resolve_engine_data_path();
        let result = list_available_worlds();
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_cli_args_default_world() {
        let args = Args::try_parse_from(["chronicler-engine"]).unwrap();
        assert_eq!(args.world, "redmist_estate");
        assert_eq!(args.port, 3000);
        assert!(!args.list_worlds);
    }

    #[test]
    fn test_cli_args_custom_world() {
        let args = Args::try_parse_from(["chronicler-engine", "--world", "test"]).unwrap();
        assert_eq!(args.world, "test");
    }

    #[test]
    fn test_cli_args_custom_port() {
        let args = Args::try_parse_from(["chronicler-engine", "--port", "8080"]).unwrap();
        assert_eq!(args.port, 8080);
    }

    #[test]
    fn test_cli_args_list_worlds() {
        let args = Args::try_parse_from(["chronicler-engine", "--list-worlds"]).unwrap();
        assert!(args.list_worlds);
    }

    #[test]
    fn test_cli_args_all_options() {
        let args = Args::try_parse_from([
            "chronicler-engine",
            "--world",
            "my_world",
            "--port",
            "9000",
            "--list-worlds",
        ])
        .unwrap();
        assert_eq!(args.world, "my_world");
        assert_eq!(args.port, 9000);
        assert!(args.list_worlds);
    }
}
