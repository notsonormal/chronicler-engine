use std::{fs, path::Path, sync::Arc, thread};

use chronicler_engine::error::EngineError;
use chronicler_engine::model::character::{NpcCard, PlayerCard};
use chronicler_engine::model::map::MapDef;
use chronicler_engine::model::state::{GameState, GeneratingGuard};
use chronicler_engine::model::world::WorldManifest;
use chronicler_engine::narrative::llm::get_llm_backend;
use chronicler_engine::narrative::prompt::PromptContext;
use chronicler_engine::server::ServerConfig;

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
    // Load from data/worlds/<world_id>/
    let world_dir = Path::new("data/worlds").join(world_id);

    if !world_dir.exists() {
        return Err(EngineError::WorldNotFound(world_id.to_string()));
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

fn main() -> chronicler_engine::Result<()> {
    dotenv::dotenv().ok();

    // Initialize logging to both stdout and file
    init_logging();

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
        Arc::new(player.clone()),
        npcs,
        manifest.starting_room_id.clone(),
    );

    // Check for starting scenario - use scenario text instead of LLM if available
    let use_scenario = if let Some(scenario) = manifest.default_scenario() {
        !scenario.text.is_empty()
    } else {
        false
    };

    // If scenario exists with text, add it to the log (skip LLM call)
    if use_scenario {
        if let Some(scenario) = manifest.default_scenario() {
            // Look up room name from map (not raw ID)
            let room_name = chronicler_engine::engine::logic::get_room_by_id(
                &state,
                &manifest.starting_room_id,
            )
            .map(|r| r.name.clone())
            .unwrap_or_else(|| manifest.starting_room_id.clone());

            // Add location entry first (sender + empty text = is_location detection)
            state.add_log(
                String::new(),
                Some(room_name),
                chronicler_engine::model::state::LogType::Narration,
            );
            // Then add scenario text
            let text = scenario.text.replace("{{user}}", &player.sheet.name);
            state.add_log(
                text,
                None,
                chronicler_engine::model::state::LogType::Narration,
            );
        }
    }

    let current_room = chronicler_engine::engine::logic::get_current_room(&state)
        .map_err(|e| chronicler_engine::EngineError::RoomNotFound(e.to_string()))?;

    // Fetch NPCs from room's NPC IDs via state.npcs HashMap (before mutating state)
    let room_npc_ids = current_room.npcs.clone();

    // For initial arrival, use static NPCs from map.json.
    // The quantifier is most useful during room transitions (handled in fragments.rs)
    // where conversation history provides context about NPC following behavior.
    let nearby_npcs: Vec<NpcCard> = room_npc_ids
        .iter()
        .filter_map(|id| state.npcs.get(id).cloned())
        .collect();

    // Get ALL NPCs from game state for prompt context
    let all_npcs: Vec<NpcCard> = state.npcs.values().cloned().collect();

    // Clone data for the background thread
    let world = state.world.clone();
    let map = state.map.clone();
    let player = state.player.clone();
    let room_id = state.current_room_id.clone();
    let history: Vec<chronicler_engine::model::state::LogEntry> = Vec::new();

    // Create shared state for the HTMX server
    let state = Arc::new(std::sync::Mutex::new(state));

    // Trigger LLM narration in background ONLY if no scenario was used
    if !use_scenario {
        let state_for_thread = state.clone();
        thread::spawn(move || {
            // RAII guard ensures is_generating is reset even if room is None
            let _guard = GeneratingGuard::new(state_for_thread.clone());

            let room = map
                .overworld
                .regions
                .iter()
                .flat_map(|r| r.rooms.iter())
                .find(|r| r.id == room_id);

            if let Some(room) = room {
                let backend = get_llm_backend();
                let context = PromptContext {
                    world: &world,
                    room,
                    all_npcs: &all_npcs,
                    npcs_in_area: &nearby_npcs,
                    player: &player,
                    user_message: "", // narrate_arrival creates its own message
                    history: &history,
                };
                let narration = backend.narrate_arrival(&context);
                match narration {
                    Ok(text) => {
                        if let Ok(mut state) = state_for_thread.lock() {
                            state.add_log(
                                text,
                                None,
                                chronicler_engine::model::state::LogType::Narration,
                            );
                            // Note: Guard will reset is_generating on drop
                        }
                    }
                    Err(e) => {
                        if let Ok(mut state) = state_for_thread.lock() {
                            state.generation_state.error_message = Some(format!("LLM Error: {e}"));
                            // Note: Guard will reset is_generating on drop
                        }
                    }
                }
            }
            // Guard drops here, resetting is_generating = false
        });
    } // end if !use_scenario

    // Run the HTTP server on the specified port
    let config = ServerConfig { port: args.port };
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(chronicler_engine::server::run_server_with_config(
        state, config,
    ))?;

    Ok(())
}

/// Initialize logging to both stdout and a daily rotating log file
fn init_logging() {
    use chrono::Local;
    use std::io::Write;

    // Create logs directory if it doesn't exist
    let log_dir = Path::new("logs");
    if !log_dir.exists() {
        if let Err(e) = fs::create_dir_all(log_dir) {
            eprintln!("Warning: Could not create logs directory: {e}");
        }
    }

    // Generate log filename with current date
    let timestamp = Local::now().format("%Y%m%d");
    let log_file_path = log_dir.join(format!("chronicler_{timestamp}.log"));

    // Write initial message to log file
    let init_msg = format!(
        "[{}] [INFO] [chronicler_engine] Logging initialized. Log file: {:?}\n",
        Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
        log_file_path
    );
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
    {
        let _ = file.write_all(init_msg.as_bytes());
    }

    // Initialize env_logger to stdout with debug level by default
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    log::info!("Logging initialized. Log file: {log_file_path:?}");
}
