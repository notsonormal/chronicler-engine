use std::sync::{Arc, RwLock};

use crate::cli::{Args, list_available_worlds, resolve_engine_data_path};
use crate::model::character::NpcCard;
use crate::model::settings::AppSettings;
use crate::model::state::GameState;
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

    // Initialise npc_encounter_log and npcs_in_area from scenario NPCs
    if let Some(scenario) = manifest.default_scenario() {
        state.init_scenario_npcs(scenario);
    }

    let nearby_npcs = state.scene.npcs_in_area.clone();
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
    let storage = Arc::new(crate::storage::snapshot_storage::SqliteGameStorage::new(
        db_pool.clone(),
        1,
    ));
    let snapshot_storage: Arc<dyn crate::storage::snapshot_storage::SnapshotStorage> =
        storage.clone();
    let message_storage: Arc<dyn crate::storage::message_storage::MessageStorage> = storage.clone();
    let llm_message_storage: Arc<dyn crate::storage::llm_message_storage::LlmMessageStorage> =
        Arc::new(crate::storage::llm_message_storage::SqliteLlmMessageStorage::new(db_pool));

    // [DOC: docs/system/startup.md]
    let initial_snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let snapshot_id = snapshot_storage.save(&initial_snapshot)?;
    for msg in state.narrative.messages.iter_mut() {
        if msg.id == crate::model::message::UNPERSISTED_ID {
            msg.snapshot_id = Some(snapshot_id);
            message_storage.insert_message(msg)?;
        }
    }

    let world_arc: Arc<crate::model::world::WorldCard> = Arc::new(manifest.clone().into());
    let map_arc: Arc<crate::model::map::MapDef> = map;
    let player_arc: Arc<crate::model::character::PlayerCard> = player;
    let npcs_map: std::collections::HashMap<String, NpcCard> = state.npcs.clone();
    let npcs_arc = Arc::new(npcs_map);

    // [DOC: docs/architecture/system.md]
    let settings = Arc::new(RwLock::new(
        crate::settings::load_settings().unwrap_or_else(|_| AppSettings::default()),
    ));
    let config = ServerConfig { port: args.port };
    let runtime = tokio::runtime::Runtime::new().map_err(|e| {
        crate::error::EngineError::Io(format!("runtime_new {}: {e}", "tokio_runtime"))
    })?;

    // [DOC: docs/system/narration_engine.md]
    let has_scenario = world_arc
        .default_scenario()
        .is_some_and(|s| !s.text.is_empty());
    if !has_scenario {
        let snapshot_storage_for_task = Arc::clone(&snapshot_storage);
        let message_storage_for_task = Arc::clone(&message_storage);
        let llm_storage_for_task = Arc::clone(&llm_message_storage);
        let world_for_task = Arc::clone(&world_arc);
        let map_for_task = Arc::clone(&map_arc);
        let player_for_task = Arc::clone(&player_arc);
        let npcs_for_task = Arc::clone(&npcs_arc);
        let settings_for_task = Arc::clone(&settings);
        // [DOC: docs/architecture/invariants.md#INV-004]
        let _handle = runtime.spawn_blocking(move || {
            let mut state = match snapshot_storage_for_task.load_latest() {
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
                    world_for_task.starting_room_id.clone(),
                ),
            };
            if let Ok(msgs) = message_storage_for_task.load_messages() {
                state.narrative.messages = msgs;
            }

            state.narrative.input_buffer.status = crate::model::state::GenerationStatus::Generating;

            let room = map_for_task
                .overworld
                .regions
                .iter()
                .flat_map(|r| r.rooms.iter())
                .find(|r| r.id == room_id);

            if let Some(room) = room {
                let settings_guard = settings_for_task.read().unwrap_or_else(|e| e.into_inner());
                let backend = crate::narrative::llm::get_llm_backend_for(
                    &settings_guard.narration_connection(),
                    Some(Arc::clone(&llm_storage_for_task)),
                    Some(Arc::clone(&settings_for_task)),
                );
                drop(settings_guard);
                let context = PromptContext {
                    world: &world_for_task,
                    room,
                    all_npcs: &all_npcs,
                    npcs_in_area: &nearby_npcs,
                    player: &player_for_task,
                    user_message: "",
                    history: &history,
                };
                let narration = backend
                    .narrate_arrival(crate::narrative::llm::backend::AGENT_NARRATOR, &context);
                match narration {
                    Ok(result) => {
                        state.add_log(result.text, None, crate::model::state::LogType::Narration);
                        state.narrative.input_buffer.status =
                            crate::model::state::GenerationStatus::Idle;
                    }
                    Err(e) => {
                        state.narrative.input_buffer.status =
                            crate::model::state::GenerationStatus::Error(format!("LLM Error: {e}"));
                    }
                }
                let snapshot =
                    crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
                if let Err(e) = snapshot_storage_for_task.save(&snapshot) {
                    log::error!("Failed to save arrival snapshot: {e}");
                }
            }
        });
    } // end if !has_scenario

    let resources = crate::server::ServerResources {
        world: world_arc,
        map: map_arc,
        player: player_arc,
        npcs: npcs_arc,
        snapshot_storage,
        message_storage,
        llm_message_storage,
        settings,
    };
    runtime.block_on(crate::server::run_server_with_config(resources, config))?;

    Ok(())
}
