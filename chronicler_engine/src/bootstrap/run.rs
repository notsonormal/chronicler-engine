use std::sync::Arc;

use crate::cli::{Args, list_available_worlds, resolve_engine_data_path};
use crate::model::character::NpcCard;
use crate::model::state::GameState;
use crate::narrative::llm::get_llm_backend;
use crate::narrative::prompt::PromptContext;
use crate::server::ServerConfig;

use super::{initialize_world_from_manifest, inject_scenario_logs, validate_loaded_data};

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

    let _world = state.world.clone();
    let map = state.map.clone();
    let player = state.player.clone();
    let room_id = state.movement.current_room_id.clone();
    let history: Vec<crate::model::state::LogEntry> = Vec::new();

    let db_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(std::path::PathBuf::from))
        .unwrap_or_else(|| data_dir.clone());
    let db_path = db_dir.join(format!("chronicler_{}.db", args.port));
    let db_pool = crate::storage::db::DbPool::new(db_path.to_str().unwrap_or("chronicler.db"))?;
    let snapshot_storage =
        Arc::new(crate::storage::snapshot_storage::SqliteSnapshotStorage::new(db_pool))
            as Arc<dyn crate::storage::snapshot_storage::SnapshotStorage>;

    // [DOC: docs/system/startup.md]
    let initial_snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
        &state,
        "initial".to_string(),
        0,
    );
    snapshot_storage.save(&initial_snapshot)?;

    let world_arc: Arc<crate::model::world::WorldCard> = Arc::new(manifest.clone().into());
    let map_arc: Arc<crate::model::map::MapDef> = map;
    let player_arc: Arc<crate::model::character::PlayerCard> = player;
    let npcs_map: std::collections::HashMap<String, NpcCard> = state.npcs.clone();
    let npcs_arc = Arc::new(npcs_map);
    let starting_room = manifest.starting_room_id.clone();

    // [DOC: docs/architecture/system.md]
    let config = ServerConfig { port: args.port };
    let runtime = tokio::runtime::Runtime::new().map_err(|e| {
        crate::error::EngineError::Io(format!("runtime_new {}: {e}", "tokio_runtime"))
    })?;

    // [DOC: docs/system/narration_engine.md]
    let has_scenario = manifest
        .default_scenario()
        .is_some_and(|s| !s.text.is_empty());
    if !has_scenario {
        let storage_for_task = Arc::clone(&snapshot_storage);
        let world_for_task = Arc::clone(&world_arc);
        let map_for_task = Arc::clone(&map_arc);
        let player_for_task = Arc::clone(&player_arc);
        let npcs_for_task = Arc::clone(&npcs_arc);
        let starting_room_for_task = starting_room.clone();
        // [DOC: docs/architecture/invariants.md#INV-004]
        let _handle = runtime.spawn_blocking(move || {
            let mut state = match storage_for_task.load_latest(None) {
                Ok(Some(snap)) => GameState::from_snapshot(
                    &snap,
                    Arc::clone(&world_for_task),
                    Arc::clone(&map_for_task),
                    Arc::clone(&player_for_task),
                    (*npcs_for_task).clone(),
                ),
                _ => GameState::new(
                    Arc::clone(&world_for_task),
                    Arc::clone(&map_for_task),
                    Arc::clone(&player_for_task),
                    (*npcs_for_task).values().cloned().collect(),
                    starting_room_for_task.clone(),
                ),
            };

            state.narrative.generation.status = crate::model::state::GenerationStatus::Generating;

            let room = map_for_task
                .overworld
                .regions
                .iter()
                .flat_map(|r| r.rooms.iter())
                .find(|r| r.id == room_id);

            if let Some(room) = room {
                let backend = get_llm_backend();
                let context = PromptContext {
                    world: &world_for_task,
                    room,
                    all_npcs: &all_npcs,
                    npcs_in_area: &nearby_npcs,
                    player: &player_for_task,
                    user_message: "",
                    history: &history,
                };
                let narration = backend.narrate_arrival(&context);
                match narration {
                    Ok(text) => {
                        state.add_log(text, None, crate::model::state::LogType::Narration);
                        state.narrative.generation.status =
                            crate::model::state::GenerationStatus::Idle;
                    }
                    Err(e) => {
                        state.narrative.generation.status =
                            crate::model::state::GenerationStatus::Error(format!("LLM Error: {e}"));
                    }
                }
                let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
                    &state,
                    "arrival".to_string(),
                    0,
                );
                let _ = storage_for_task.save(&snapshot);
            }
        });
    } // end if !has_scenario

    runtime.block_on(crate::server::run_server_with_config(
        world_arc,
        map_arc,
        player_arc,
        npcs_arc,
        starting_room,
        snapshot_storage,
        config,
    ))?;

    Ok(())
}
