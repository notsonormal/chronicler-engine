use std::{fs, path::Path, sync::Arc};

use chronicler_engine::model::character::{NpcCard, PlayerCard};
use chronicler_engine::model::map::MapDef;
use chronicler_engine::model::state::GameState;
use chronicler_engine::model::world::WorldManifest;
use chronicler_engine::server::{Hub, ServerConfig};

use clap::Parser;

/// Command-line arguments for the Chronicler Engine
#[derive(Parser, Debug)]
#[command(name = "chronicler-engine")]
#[command(version = "0.1.0")]
#[command(about = "Text adventure engine with HTMX dashboard")]
struct Args {
    /// Specify which world to load
    #[arg(long, default_value = "redmist_estate")]
    world: String,

    /// List all available worlds and exit
    #[arg(long)]
    list_worlds: bool,

    /// Port to run the HTTP server on
    #[arg(long, default_value = "3000")]
    port: u16,
}

/// Load world manifest from file
fn load_world_manifest(world_id: &str) -> chronicler_engine::Result<WorldManifest> {
    let path = Path::new("data/worlds").join(world_id).join("world.json");
    let json = fs::read_to_string(&path)?;
    let manifest: WorldManifest = serde_json::from_str(&json)?;
    Ok(manifest)
}

/// List all available worlds
fn list_available_worlds() -> chronicler_engine::Result<()> {
    let worlds_dir = Path::new("data/worlds");
    if !worlds_dir.exists() {
        println!("No worlds found in data/worlds/");
        return Ok(());
    }

    let mut worlds = Vec::new();
    for entry in fs::read_dir(worlds_dir)? {
        let entry = entry?;
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

/// Verify that a world can be loaded (for testing purposes)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_redmist_estate_world() {
        let result = load_world("redmist_estate");
        assert!(
            result.is_ok(),
            "Failed to load redmist_estate: {:?}",
            result
        );
        let (manifest, _map, _player, npcs) = result.unwrap();
        assert_eq!(manifest.id, "redmist_estate");
        assert_eq!(manifest.name, "Redmist Estate");
        assert_eq!(manifest.starting_room_id, "front_gates");
        assert!(!npcs.is_empty(), "Should have NPCs");
    }

    #[test]
    fn test_load_test_world() {
        let result = load_world("test");
        assert!(result.is_ok(), "Failed to load test world: {:?}", result);
        let (manifest, _map, player, npcs) = result.unwrap();
        assert_eq!(manifest.id, "test");
        assert_eq!(manifest.name, "Test Realm");
        assert_eq!(player.sheet.name, "Test Player");
        // Test world has 3 NPCs: ranger, shopkeeper, bartender
        assert_eq!(npcs.len(), 3, "Test world should have 3 NPCs");
    }

    #[test]
    fn test_list_worlds() {
        let result = list_available_worlds();
        assert!(result.is_ok(), "list_available_worlds should not fail");
    }
}

/// Load a complete game world from the worlds directory structure
fn load_world(
    world_id: &str,
) -> chronicler_engine::Result<(WorldManifest, MapDef, PlayerCard, Vec<NpcCard>)> {
    // Try new directory structure first: data/worlds/<world_id>/
    let world_dir = Path::new("data/worlds").join(world_id);

    if !world_dir.exists() {
        // Fall back to legacy structure for backward compatibility
        return load_world_legacy(world_id);
    }

    // Load world manifest
    let manifest = load_world_manifest(world_id)?;

    // Load map
    let map_path = world_dir.join(&manifest.map_file);
    let map_json = fs::read_to_string(&map_path)?;
    let map: MapDef = serde_json::from_str(&map_json)?;

    // Load player
    let player_path = world_dir.join(&manifest.player_file);
    let player_json = fs::read_to_string(&player_path)?;
    let player: PlayerCard = serde_json::from_str(&player_json)?;

    // Load NPCs from characters directory
    let mut npcs = Vec::new();
    let chars_dir = world_dir.join("characters");
    if chars_dir.is_dir() {
        for entry in fs::read_dir(&chars_dir)?.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let char_json = fs::read_to_string(&path)?;
                match serde_json::from_str::<NpcCard>(&char_json) {
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

/// Load world using legacy file structure (backward compatibility)
fn load_world_legacy(
    world_id: &str,
) -> chronicler_engine::Result<(WorldManifest, MapDef, PlayerCard, Vec<NpcCard>)> {
    // Load world from data/world/<id>.json
    let world_path = Path::new("data/world").join(format!("{world_id}.json"));
    let world_json = fs::read_to_string(&world_path)?;
    let mut manifest: WorldManifest = serde_json::from_str(&world_json)?;
    manifest.id = world_id.to_string();

    // Load map from data/maps/<id>.json
    let map_path = Path::new("data/maps").join(format!("{world_id}.json"));
    let map_json = fs::read_to_string(&map_path)?;
    let map: MapDef = serde_json::from_str(&map_json)?;

    // For player, try personas/<id>.json or default to Julian
    let player_path = Path::new("data/personas").join(format!("{world_id}.json"));
    let player_json = if player_path.exists() {
        fs::read_to_string(&player_path)?
    } else {
        // Default to Julian
        fs::read_to_string("data/personas/julian.json")?
    };
    let player: PlayerCard = serde_json::from_str(&player_json)?;

    // Load NPCs from data/characters/
    let mut npcs = Vec::new();
    let chars_dir = Path::new("data/characters");
    if chars_dir.is_dir() {
        for entry in fs::read_dir(chars_dir)?.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let char_json = fs::read_to_string(&path)?;
                match serde_json::from_str::<NpcCard>(&char_json) {
                    Ok(npc) => npcs.push(npc),
                    Err(e) => {
                        eprintln!("Warning: Failed to parse NPC file {path:?}: {e}");
                    }
                }
            }
        }
    }

    // Set defaults for legacy worlds
    if manifest.starting_room_id.is_empty() {
        manifest.starting_room_id = "front_gates".to_string();
    }

    Ok((manifest, map, player, npcs))
}

fn main() -> chronicler_engine::Result<()> {
    dotenv::dotenv().ok();

    let args = Args::parse();

    // Handle --list-worlds flag
    if args.list_worlds {
        list_available_worlds()?;
        return Ok(());
    }

    // Load the world
    let (manifest, map, player, npcs) = load_world(&args.world)?;

    // Create game state with the starting room from manifest
    let mut state = GameState::new(
        Arc::new(manifest.clone().into()),
        Arc::new(map),
        Arc::new(player),
        npcs,
        manifest.starting_room_id.clone(),
    );

    // Initialize game state with welcome messages
    state.add_log(
        format!("Welcome to {}.", state.world.name),
        None,
        chronicler_engine::model::state::LogType::System,
    );
    state.add_log(
        format!("Logged in as: {}", state.player.sheet.name),
        None,
        chronicler_engine::model::state::LogType::System,
    );

    let current_room = chronicler_engine::engine::logic::get_current_room(&state)
        .map_err(|e| chronicler_engine::EngineError::RoomNotFound(e.to_string()))?;
    state.add_log(
        current_room.description.clone(),
        Some(current_room.name.clone()),
        chronicler_engine::model::state::LogType::Narration,
    );

    // Create shared state and WebSocket hub
    let state = Arc::new(std::sync::Mutex::new(state));
    let hub = Hub::new();

    // Run the HTTP server on the specified port
    let config = ServerConfig { port: args.port };
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(chronicler_engine::server::run_server_with_config(
        state, hub, config,
    ))?;

    Ok(())
}
