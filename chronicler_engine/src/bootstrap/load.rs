//! [DOC: docs/system/startup.md]
//! Data loading and initialization routines
#![allow(dead_code)] // Legacy functions used by bootstrap/load_tests.rs
use std::{fs, path::Path};

use crate::error::EngineError;
use crate::model::character::{NpcCard, PlayerCard};
use crate::model::map::MapDef;
use crate::model::world::{WorldCard, WorldManifest, derive_player_key};
use crate::storage::Storage;

pub(crate) fn read_json_file<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> crate::error::Result<T> {
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

pub(crate) fn seed_game_data(
    storage: &Storage,
    data_dir: &std::path::Path,
) -> crate::error::Result<()> {
    let worlds_dir = data_dir.join("worlds");
    if !worlds_dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(&worlds_dir)? {
        let entry = entry?;
        let world_dir = entry.path();
        if !world_dir.is_dir() {
            continue;
        }
        let world_json = world_dir.join("world.json");
        if !world_json.exists() {
            continue;
        }

        let manifest: WorldManifest = match read_json_file(&world_json) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(
                    "Failed to parse world manifest {}: {e}",
                    world_json.display()
                );
                continue;
            }
        };
        let world_key = manifest.id.clone();

        let player_key = derive_player_key(&manifest.player_file);

        let world_id = match storage.get_world(&world_key)? {
            Some(existing) => existing.world_id,
            None => {
                let world_card: WorldCard = manifest.clone().into();
                let map_path = world_dir.join(&manifest.map_file);
                let map: MapDef = read_json_file(&map_path)?;
                storage.seed_world(&world_card, &map)?;

                storage.get_world_id(&world_key)?.ok_or_else(|| {
                    EngineError::Config(format!("World '{world_key}' not found after seeding"))
                })?
            }
        };

        if storage.get_persona(&player_key)?.is_none() {
            let player_path = data_dir.join("personas").join(&manifest.player_file);
            let player: PlayerCard = read_json_file(&player_path)?;
            storage.seed_persona(&player_key, &player)?;
            tracing::info!("Seeded persona: {player_key}");
        }

        let chars_group = if manifest.characters_dir.is_empty() {
            world_key.as_str()
        } else {
            manifest.characters_dir.as_str()
        };
        let chars_dir = data_dir.join("characters").join(chars_group);
        if chars_dir.is_dir() {
            let existing_chars: std::collections::HashSet<String> = storage
                .list_characters(world_id)?
                .into_iter()
                .map(|c| c.id)
                .collect();
            for char_entry in std::fs::read_dir(&chars_dir)? {
                let char_entry = char_entry?;
                let path = char_entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                match read_json_file::<NpcCard>(&path) {
                    Ok(npc) => {
                        if !existing_chars.contains(&npc.id) {
                            storage.seed_character(world_id, &npc)?;
                            tracing::info!("Seeded character: {}", npc.id);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse NPC {}: {e}", path.display())
                    }
                }
            }
        }
    }
    Ok(())
}
