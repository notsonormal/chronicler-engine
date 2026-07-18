//! [DOC: docs/system/startup.md]
//! Game data seeding and initialization routines

use std::{fs, path::Path};

use crate::error::EngineError;
use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::map::MapDef;
use crate::domain::model::world::{WorldCard, WorldManifest};
use crate::adapters::driven::storage::Storage;

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

pub fn seed_game_data(storage: &Storage, data_dir: &std::path::Path) -> crate::error::Result<()> {
    let worlds_dir = data_dir.join("worlds");
    if worlds_dir.is_dir() {
        seed_worlds(storage, data_dir, &worlds_dir)?;
    }

    let personas_dir = data_dir.join("personas");
    if personas_dir.is_dir() {
        seed_personas(storage, &personas_dir)?;
    }

    Ok(())
}

fn seed_worlds(
    storage: &Storage,
    data_dir: &std::path::Path,
    worlds_dir: &std::path::Path,
) -> crate::error::Result<()> {
    for entry in std::fs::read_dir(worlds_dir)? {
        let entry = entry?;
        let world_dir = entry.path();
        if !world_dir.is_dir() {
            continue;
        }
        process_world_dir(storage, &world_dir, data_dir)?;
    }
    Ok(())
}

fn process_world_dir(
    storage: &Storage,
    world_dir: &Path,
    data_dir: &Path,
) -> crate::error::Result<()> {
    let world_json = world_dir.join("world.json");
    if !world_json.exists() {
        return Ok(());
    }

    let manifest: WorldManifest = match read_json_file(&world_json) {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!(
                "Failed to parse world manifest {}: {e}",
                world_json.display()
            );
            return Ok(());
        }
    };
    let world_key = manifest.id.clone();
    let characters_dir = manifest.characters_dir.clone();

    let world_with_map = storage.get_world(&world_key)?;
    let world_id = match world_with_map {
        Some(existing) => existing.world_id,
        None => {
            let map_file = manifest.map_file.clone();
            let world_card: WorldCard = manifest.into();
            let map_path = world_dir.join(&map_file);
            let map: MapDef = read_json_file(&map_path)?;
            storage.seed_world(&world_card, &map)?
        }
    };

    let chars_group = if characters_dir.is_empty() {
        world_key.as_str()
    } else {
        characters_dir.as_str()
    };
    let chars_dir = data_dir.join("characters").join(chars_group);
    if !chars_dir.is_dir() {
        return Ok(());
    }
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
    Ok(())
}

fn seed_personas(storage: &Storage, personas_dir: &std::path::Path) -> crate::error::Result<()> {
    for entry in std::fs::read_dir(personas_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let key = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if key.is_empty() {
            continue;
        }
        if storage.get_persona(&key)?.is_none() {
            let persona: PersonaCard = read_json_file(&path)?;
            storage.seed_persona(&key, &persona)?;
            tracing::info!("Seeded persona: {key}");
        }
    }
    Ok(())
}
