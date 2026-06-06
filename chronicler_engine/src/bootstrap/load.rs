//! [DOC: docs/system/startup.md]
//! Data loading and initialization routines
use std::{fs, path::Path};

use crate::error::EngineError;
use crate::model::character::{NpcCard, PlayerCard};
use crate::model::map::MapDef;
use crate::model::world::WorldManifest;

fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> crate::error::Result<T> {
    let json = fs::read_to_string(path).map_err(|e| EngineError::DataLoad {
        path: path.display().to_string(),
        source: Box::new(EngineError::Io(format!(
            "read_to_string {}: {e}",
            path.display()
        ))),
    })?;
    serde_json::from_str(&json).map_err(|e| EngineError::DataLoad {
        path: path.display().to_string(),
        source: Box::new(e.into()),
    })
}

pub fn load_world_manifest(world_id: &str, data_dir: &Path) -> crate::error::Result<WorldManifest> {
    let path = data_dir.join("worlds").join(world_id).join("world.json");
    read_json_file(&path)
}

pub fn initialize_world_from_manifest(
    world_id: &str,
    data_dir: &Path,
) -> crate::error::Result<(WorldManifest, MapDef, PlayerCard, Vec<NpcCard>)> {
    let world_dir = data_dir.join("worlds").join(world_id);

    if !world_dir.exists() {
        return Err(EngineError::WorldNotFound(world_id.to_string()));
    }

    let manifest = load_world_manifest(world_id, data_dir)?;

    let map_path = world_dir.join(&manifest.map_file);
    let map: MapDef = read_json_file(&map_path)?;

    let player_path = data_dir.join("personas").join(&manifest.player_file);
    let player: PlayerCard = read_json_file(&player_path)?;

    let mut npcs = Vec::new();
    let characters_group = if manifest.characters_dir.is_empty() {
        world_id
    } else {
        &manifest.characters_dir
    };
    let chars_dir = data_dir.join("characters").join(characters_group);
    if chars_dir.is_dir() {
        for entry in fs::read_dir(&chars_dir)
            .map_err(|e| EngineError::Io(format!("read_dir {}: {e}", chars_dir.display())))?
            .flatten()
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match read_json_file::<NpcCard>(&path) {
                    Ok(npc) => npcs.push(npc),
                    Err(e) => {
                        eprintln!("Warning: Failed to parse NPC file {path:?}: {e}");
                    }
                }
            }
        }
    }

    Ok((manifest, map, player, npcs))
}
