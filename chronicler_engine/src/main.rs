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

/// [DOC: docs/architecture/system.md]
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

use std::path::PathBuf;

/// [DOC: docs/architecture/system.md]
fn resolve_engine_data_path() -> PathBuf {
    // [DOC: docs/system/startup.md]
    if let Ok(data_dir) = std::env::var("CHRONICLER_DATA") {
        return PathBuf::from(data_dir);
    }

    // Deprecated: CHRONLER_DATA was a typo, kept for backward compatibility.
    if let Ok(data_dir) = std::env::var("CHRONLER_DATA") {
        eprintln!("Warning: CHRONLER_DATA is deprecated. Use CHRONICLER_DATA instead.");
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

fn load_world_manifest(world_id: &str) -> chronicler_engine::Result<WorldManifest> {
    let data_dir = resolve_engine_data_path();
    let path = data_dir.join("worlds").join(world_id).join("world.json");
    let json = fs::read_to_string(&path).map_err(|e| EngineError::DataLoad {
        path: path.display().to_string(),
        source: Box::new(e.into()),
    })?;
    let manifest: WorldManifest =
        serde_json::from_str(&json).map_err(|e| EngineError::DataLoad {
            path: path.display().to_string(),
            source: Box::new(e.into()),
        })?;
    Ok(manifest)
}

/// [DOC: docs/architecture/system.md]
fn list_available_worlds() -> chronicler_engine::Result<()> {
    let data_dir = resolve_engine_data_path();
    let worlds_dir = data_dir.join("worlds");
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

/// [DOC: docs/system/testing.md]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_redmist_estate_world() {
        let result = initialize_world_from_manifest("redmist_estate");
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
        let result = initialize_world_from_manifest("test");
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

/// [DOC: docs/architecture/system.md]
fn initialize_world_from_manifest(
    world_id: &str,
) -> chronicler_engine::Result<(WorldManifest, MapDef, PlayerCard, Vec<NpcCard>)> {
    let data_dir = resolve_engine_data_path();
    let world_dir = data_dir.join("worlds").join(world_id);

    if !world_dir.exists() {
        return Err(EngineError::WorldNotFound(world_id.to_string()));
    }

    // [DOC: docs/system/startup.md]
    let manifest = load_world_manifest(world_id)?;

    let map_path = world_dir.join(&manifest.map_file);
    let map_json = fs::read_to_string(&map_path).map_err(|e| EngineError::DataLoad {
        path: map_path.display().to_string(),
        source: Box::new(e.into()),
    })?;
    let map: MapDef = serde_json::from_str(&map_json).map_err(|e| EngineError::DataLoad {
        path: map_path.display().to_string(),
        source: Box::new(e.into()),
    })?;

    let player_path = world_dir.join(&manifest.player_file);
    let player_json = fs::read_to_string(&player_path).map_err(|e| EngineError::DataLoad {
        path: player_path.display().to_string(),
        source: Box::new(e.into()),
    })?;
    let player: PlayerCard =
        serde_json::from_str(&player_json).map_err(|e| EngineError::DataLoad {
            path: player_path.display().to_string(),
            source: Box::new(e.into()),
        })?;

    let mut npcs = Vec::new();
    let chars_dir = world_dir.join("characters");
    if chars_dir.is_dir() {
        for entry in fs::read_dir(&chars_dir)?.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let char_json = fs::read_to_string(&path).map_err(|e| EngineError::DataLoad {
                    path: path.display().to_string(),
                    source: Box::new(e.into()),
                })?;
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

    init_logging();

    let args = Args::parse();

    if args.list_worlds {
        list_available_worlds()?;
        return Ok(());
    }

    let (manifest, map, player, npcs) = initialize_world_from_manifest(&args.world)?;

    let mut state = GameState::new(
        Arc::new(manifest.clone().into()),
        Arc::new(map),
        Arc::new(player.clone()),
        npcs,
        manifest.starting_room_id.clone(),
    );

    if let Some(scenario) = manifest.default_scenario() {
        if !scenario.text.is_empty() {
            let room_name = chronicler_engine::engine::logic::find_room_in_world_map(
                &state,
                &manifest.starting_room_id,
            )
            .map(|r| r.name.clone())
            .unwrap_or_else(|| manifest.starting_room_id.clone());

            state.add_log(
                String::new(),
                Some(room_name),
                chronicler_engine::model::state::LogType::Narration,
            );
            let text = scenario.text.replace("{{user}}", &player.sheet.name);
            state.add_log(
                text,
                None,
                chronicler_engine::model::state::LogType::Narration,
            );
        }
    }

    let current_room = chronicler_engine::engine::logic::get_current_room(&state)?;

    let room_npc_ids = current_room.npcs.clone();

    let nearby_npcs: Vec<NpcCard> = room_npc_ids
        .iter()
        .filter_map(|id| state.npcs.get(id).cloned())
        .collect();

    let all_npcs: Vec<NpcCard> = state.npcs.values().cloned().collect();

    let world = state.world.clone();
    let map = state.map.clone();
    let player = state.player.clone();
    let room_id = state.current_room_id.clone();
    let history: Vec<chronicler_engine::model::state::LogEntry> = Vec::new();

    let state = Arc::new(std::sync::Mutex::new(state));

    // [DOC: docs/system/narration_engine.md]
    let has_scenario = manifest
        .default_scenario()
        .is_some_and(|s| !s.text.is_empty());
    if !has_scenario {
        let state_for_thread = state.clone();
        thread::spawn(move || {
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
                    user_message: "",
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
                        }
                    }
                    Err(e) => {
                        if let Ok(mut state) = state_for_thread.lock() {
                            state.generation_state.error_message = Some(format!("LLM Error: {e}"));
                        }
                    }
                }
            }
        });
    } // end if !has_scenario

    // [DOC: docs/architecture/system.md]
    let config = ServerConfig { port: args.port };
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(chronicler_engine::server::run_server_with_config(
        state, config,
    ))?;

    Ok(())
}

/// [DOC: docs/architecture/system.md]
fn init_logging() {
    use chrono::Local;

    let log_dir = Path::new("logs");
    if !log_dir.exists() {
        if let Err(e) = fs::create_dir_all(log_dir) {
            eprintln!("Warning: Could not create logs directory: {e}");
        }
    }

    let timestamp = Local::now().format("%Y%m%d");
    let log_file_path = log_dir.join(format!("chronicler_{timestamp}.log"));

    // Open file for writing all logs
    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
        .expect("Failed to open log file");

    // Configure env_logger to write to the file
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .init();

    // Also print to console so user sees output when running cargo run
    println!("Logging to file: {log_file_path:?}");

    log::info!("Logging initialized. Log file: {log_file_path:?}");
}
