//! [DOC: docs/system/startup.md]
//! Command-line interface definitions

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;

use crate::error::EngineError;
use crate::domain::model::world::WorldManifest;

#[derive(Parser, Debug)]
#[command(name = "chronicler-engine")]
#[command(version = "0.1.0")]
#[command(about = "Text adventure engine with HTMX dashboard")]
pub struct Args {
    #[arg(long, default_value = "redmist_estate")]
    pub world: String,

    #[arg(long, default_value = "julian")]
    pub persona: String,

    #[arg(long)]
    pub list_worlds: bool,

    #[arg(long, default_value = "3000")]
    pub port: u16,

    #[arg(long)]
    pub settings_path: Option<std::path::PathBuf>,
}

pub fn parse_args() -> Args {
    Args::parse()
}

pub fn resolve_engine_data_path() -> PathBuf {
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

pub fn scan_worlds(data_dir: &Path) -> crate::error::Result<Vec<(String, String)>> {
    let worlds_dir = data_dir.join("worlds");
    if !worlds_dir.exists() {
        return Ok(vec![]);
    }
    discover_worlds_in_dir(&worlds_dir)
}

fn discover_worlds_in_dir(dir: &Path) -> crate::error::Result<Vec<(String, String)>> {
    let entries = fs::read_dir(dir)
        .map_err(|e| EngineError::Io(format!("read_dir {}: {e}", dir.display())))?;
    let parsed: Vec<Option<(String, String)>> = entries
        .map(|entry_result| {
            let entry = entry_result
                .map_err(|e| EngineError::Io(format!("read_dir_entry {}: {e}", dir.display())))?;
            Ok(parse_world_at(&entry.path()))
        })
        .collect::<crate::error::Result<Vec<_>>>()?;
    Ok(parsed.into_iter().flatten().collect())
}

fn parse_world_at(path: &Path) -> Option<(String, String)> {
    if !path.is_dir() {
        return None;
    }
    let world_file = path.join("world.json");
    if !world_file.exists() {
        return None;
    }
    let json = fs::read_to_string(&world_file).ok()?;
    let manifest = serde_json::from_str::<WorldManifest>(&json).ok()?;
    Some((manifest.id.clone(), manifest.name.clone()))
}

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
