// Bootstrap module is allowed to use stdout/stderr for CLI output.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::{fs, path::Path, sync::Arc};

use chrono::Local;

use crate::cli::{Args, list_available_worlds, resolve_engine_data_path};
use crate::error::EngineError;
use crate::model::character::{NpcCard, PlayerCard};
use crate::model::map::MapDef;
use crate::model::state::{GameState, GeneratingGuard};
use crate::model::world::WorldManifest;
use crate::narrative::llm::get_llm_backend;
use crate::narrative::prompt::PromptContext;
use crate::server::ServerConfig;

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

/// [DOC: docs/architecture/system.md]
pub fn initialize_world_from_manifest(
    world_id: &str,
    data_dir: &Path,
) -> crate::error::Result<(WorldManifest, MapDef, PlayerCard, Vec<NpcCard>)> {
    let world_dir = data_dir.join("worlds").join(world_id);

    if !world_dir.exists() {
        return Err(EngineError::WorldNotFound(world_id.to_string()));
    }

    // [DOC: docs/system/startup.md]
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

/// [DOC: docs/architecture/system.md]
pub fn validate_loaded_data(
    manifest: &WorldManifest,
    map: &MapDef,
    _player: &PlayerCard,
    npcs: &[NpcCard],
) -> Result<(), String> {
    let mut errors = Vec::new();

    let mut valid_room_ids = std::collections::HashSet::new();
    for region in &map.overworld.regions {
        for room in &region.rooms {
            valid_room_ids.insert(room.id.clone());
        }
    }

    // 1. Validate starting room exists
    if !valid_room_ids.contains(&manifest.starting_room_id) {
        errors.push(format!(
            "starting_room_id '{}' not found in map",
            manifest.starting_room_id
        ));
    }

    // 2. Validate all NPCs referenced in the map actually exist
    let loaded_npc_ids: std::collections::HashSet<_> = npcs.iter().map(|n| n.id.clone()).collect();
    for region in &map.overworld.regions {
        for room in &region.rooms {
            for npc_id in &room.npcs {
                if !loaded_npc_ids.contains(npc_id) {
                    errors.push(format!(
                        "Map room '{}' references missing NPC '{}'",
                        room.id, npc_id
                    ));
                }
            }
        }
    }

    // 3. Validate trigger room_ids exist in the map
    for npc in npcs {
        for (i, trigger) in npc.triggers.iter().enumerate() {
            if let Some(room_id) = &trigger.room_id {
                if !valid_room_ids.contains(room_id) {
                    errors.push(format!(
                        "NPC '{}' Trigger[{}] references non-existent room_id: '{}'",
                        npc.id, i, room_id
                    ));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

/// [DOC: docs/architecture/system.md]
pub fn inject_scenario_logs(state: &mut GameState, manifest: &WorldManifest, player: &PlayerCard) {
    let Some(scenario) = manifest.default_scenario() else {
        return;
    };
    if scenario.text.is_empty() {
        return;
    }

    let room_name = crate::engine::logic::find_room_in_world_map(state, &manifest.starting_room_id)
        .map(|r| r.name.clone())
        .unwrap_or_else(|| manifest.starting_room_id.clone());

    state.add_log(
        String::new(),
        Some(room_name),
        crate::model::state::LogType::Narration,
    );
    let text = scenario.text.replace("{{user}}", &player.sheet.name);
    state.add_log(text, None, crate::model::state::LogType::Narration);
}

/// [DOC: docs/architecture/system.md]
pub fn run(args: Args) -> crate::error::Result<()> {
    if args.list_worlds {
        list_available_worlds()?;
        return Ok(());
    }

    let data_dir = resolve_engine_data_path();
    let (manifest, map, player, npcs) = initialize_world_from_manifest(&args.world, &data_dir)?;

    if let Err(e) = validate_loaded_data(&manifest, &map, &player, &npcs) {
        log::error!("Data validation failed for world '{}':\n{}", args.world, e);
        eprintln!("Data validation failed for world '{}':\n{}", args.world, e);
        std::process::exit(1);
    }

    let mut state = GameState::new(
        Arc::new(manifest.clone().into()),
        Arc::new(map),
        Arc::new(player.clone()),
        npcs,
        manifest.starting_room_id.clone(),
    );

    inject_scenario_logs(&mut state, &manifest, &player);

    let current_room = crate::engine::logic::get_current_room(&state)?;

    let room_npc_ids = current_room.npcs.clone();

    let nearby_npcs: Vec<NpcCard> = room_npc_ids
        .iter()
        .filter_map(|id| state.npcs.get(id).cloned())
        .collect();

    let all_npcs: Vec<NpcCard> = state.npcs.values().cloned().collect();

    let world = state.world.clone();
    let map = state.map.clone();
    let player = state.player.clone();
    let room_id = state.movement.current_room_id.clone();
    let history: Vec<crate::model::state::LogEntry> = Vec::new();

    let state = Arc::new(std::sync::Mutex::new(state));

    // [DOC: docs/architecture/system.md]
    let config = ServerConfig { port: args.port };
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| EngineError::Io(format!("runtime_new {}: {e}", "tokio_runtime")))?;

    // [DOC: docs/system/narration_engine.md]
    let has_scenario = manifest
        .default_scenario()
        .is_some_and(|s| !s.text.is_empty());
    if !has_scenario {
        // [DOC: docs/architecture/invariants.md#INV-004]
        // Arrival narration runs off the async thread so the server starts immediately.
        let state_for_task = state.clone();
        let _handle = runtime.spawn_blocking(move || {
            let _guard = GeneratingGuard::new(state_for_task.clone());

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
                        if let Ok(mut state) = state_for_task.lock() {
                            state.add_log(text, None, crate::model::state::LogType::Narration);
                        }
                    }
                    Err(e) => {
                        if let Ok(mut state) = state_for_task.lock() {
                            state.narrative.generation.status =
                                crate::model::state::GenerationStatus::Error(format!(
                                    "LLM Error: {e}"
                                ));
                        }
                    }
                }
            }
        });
    } // end if !has_scenario

    runtime.block_on(crate::server::run_server_with_config(state, config))?;

    Ok(())
}

/// [DOC: docs/architecture/system.md]
pub fn init_logging() {
    let log_dir = Path::new("logs");
    if !log_dir.exists() {
        if let Err(e) = fs::create_dir_all(log_dir) {
            eprintln!("Warning: Could not create logs directory: {e}");
        }
    }

    let timestamp = Local::now().format("%Y%m%d");
    let log_file_path = log_dir.join(format!("chronicler_{timestamp}.log"));

    match fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
    {
        Ok(log_file) => {
            // Configure env_logger to write to the file
            env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Debug)
                .target(env_logger::Target::Pipe(Box::new(log_file)))
                .init();
        }
        Err(e) => {
            eprintln!("Warning: Could not open log file {log_file_path:?}: {e}");
            env_logger::Builder::from_default_env()
                .filter_level(log::LevelFilter::Debug)
                .init();
        }
    }

    // Also print to console so user sees output when running cargo run
    println!("Logging to file: {log_file_path:?}");

    log::info!("Logging initialized. Log file: {log_file_path:?}");
}
