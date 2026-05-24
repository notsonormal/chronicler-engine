use std::sync::{Arc, RwLock};

use crate::cli::{Args, list_available_worlds, resolve_engine_data_path};
use crate::model::character::NpcCard;
use crate::model::game::generate_game_name;
use crate::model::settings::AppSettings;
use crate::model::state::GameState;
use crate::narrative::prompt::PromptContext;
use crate::server::ServerConfig;
use crate::storage::prompt_preset_storage::PromptPresetStorage;

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

    if let Err(e) = ensure_defaults(&db_pool, &data_dir) {
        log::warn!("Failed to seed prompt presets: {e}");
    }

    let active_game_id = match find_latest_game_for_world(&db_pool, &manifest.name)? {
        Some((id, name)) => {
            log::info!("Loaded existing game '{name}' (id={id})");
            id
        }
        None => {
            let existing_names = list_game_names_for_world(&db_pool, &manifest.name)?;
            let name = generate_game_name(&manifest.name, &existing_names);
            let conn = db_pool.conn();
            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO games (world_name, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
                rusqlite::params![&manifest.name, &name, &now],
            )
            .map_err(|e| crate::error::EngineError::Config(format!("Failed to create game: {e}")))?;
            let id = conn.last_insert_rowid() as u64;
            log::info!("Created new game '{name}' (id={id})");
            id
        }
    };

    let snapshot_storage: Arc<dyn crate::storage::snapshot_storage::SnapshotStorage> = Arc::new(
        crate::storage::snapshot_storage::SqliteSnapshotRepository::new(
            db_pool.clone(),
            active_game_id,
        ),
    );
    let message_storage: Arc<dyn crate::storage::message_storage::MessageStorage> = Arc::new(
        crate::storage::message_storage::SqliteMessageRepository::new(
            db_pool.clone(),
            active_game_id,
        ),
    );
    let llm_message_storage: Arc<dyn crate::storage::llm_message_storage::LlmMessageStorage> =
        Arc::new(
            crate::storage::llm_message_storage::SqliteLlmMessageStorage::new(db_pool.clone()),
        );

    // [DOC: docs/system/startup.md]
    let initial_snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let snapshot_id = snapshot_storage.save(&initial_snapshot)?;
    if let Some(msg) = state.narrative.history.last_mut() {
        if msg.id == 0 {
            msg.snapshot_id = Some(snapshot_id);
            let id = message_storage.insert_message(&*msg)?;
            msg.id = id;
        }
    }

    let world_arc: Arc<crate::model::world::WorldCard> = Arc::new(manifest.clone().into());
    let map_arc: Arc<crate::model::map::MapDef> = map;
    let player_arc: Arc<crate::model::character::PlayerCard> = player;
    let npcs_map: std::collections::HashMap<String, NpcCard> = state.npcs.clone();
    let npcs_arc = Arc::new(npcs_map);

    // [DOC: docs/architecture/system.md]
    let mut settings = crate::settings::load_settings().unwrap_or_else(|_| AppSettings::default());

    let preset_storage =
        crate::storage::prompt_preset_storage::SqlitePromptPresetStorage::new(db_pool.clone());
    if let Ok(Some(preset)) = preset_storage.get(&settings.active_system_prompt_preset_id) {
        settings.active_system_prompt = Some(preset.prompt_text);
    }
    if let Ok(Some(preset)) = preset_storage.get(&settings.active_quantifier_prompt_preset_id) {
        settings.active_quantifier_prompt = Some(preset.prompt_text);
    }

    let settings = Arc::new(RwLock::new(settings));
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
                state.narrative.history.replace(msgs);
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
                    system_prompt_override: None,
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

    let prompt_preset_storage: Arc<dyn crate::storage::prompt_preset_storage::PromptPresetStorage> =
        Arc::new(crate::storage::prompt_preset_storage::SqlitePromptPresetStorage::new(db_pool));

    let resources = crate::server::ServerResources {
        world: world_arc,
        map: map_arc,
        player: player_arc,
        npcs: npcs_arc,
        snapshot_storage,
        message_storage,
        llm_message_storage,
        prompt_preset_storage,
        settings,
    };
    runtime.block_on(crate::server::run_server_with_config(resources, config))?;

    Ok(())
}

pub(crate) fn find_latest_game_for_world(
    db_pool: &crate::storage::db::DbPool,
    world_name: &str,
) -> Result<Option<(u64, String)>, crate::error::EngineError> {
    let conn = db_pool.conn();
    let mut stmt = conn
        .prepare(
            "SELECT g.id, g.name
             FROM games g
             LEFT JOIN (
                 SELECT game_id, MAX(timestamp) as last_message
                 FROM messages
                 GROUP BY game_id
             ) m ON g.id = m.game_id
             WHERE g.world_name = ?1
             ORDER BY COALESCE(m.last_message, g.updated_at) DESC
             LIMIT 1",
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Failed to prepare query: {e}")))?;

    let result = stmt.query_row(rusqlite::params![world_name], |row| {
        Ok((row.get::<_, i64>(0)? as u64, row.get::<_, String>(1)?))
    });

    match result {
        Ok(pair) => Ok(Some(pair)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(crate::error::EngineError::Config(format!(
            "Failed to query games: {e}"
        ))),
    }
}

pub(crate) fn list_game_names_for_world(
    db_pool: &crate::storage::db::DbPool,
    world_name: &str,
) -> Result<Vec<String>, crate::error::EngineError> {
    let conn = db_pool.conn();
    let mut stmt = conn
        .prepare("SELECT name FROM games WHERE world_name = ?1")
        .map_err(|e| crate::error::EngineError::Config(format!("Failed to prepare query: {e}")))?;

    let rows = stmt
        .query_map(rusqlite::params![world_name], |row| row.get::<_, String>(0))
        .map_err(|e| crate::error::EngineError::Config(format!("Failed to query games: {e}")))?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| crate::error::EngineError::Config(format!("Failed to read game names: {e}")))
}

fn ensure_defaults(
    db_pool: &crate::storage::db::DbPool,
    data_dir: &std::path::Path,
) -> crate::error::Result<()> {
    use crate::model::prompt_preset::{PresetType, PromptPreset};
    use crate::storage::prompt_preset_storage::PromptPresetStorage;

    let storage =
        crate::storage::prompt_preset_storage::SqlitePromptPresetStorage::new(db_pool.clone());

    for preset_type in [PresetType::System, PresetType::Quantifier] {
        let dir = data_dir.join("prompt_presets").join(preset_type.as_str());
        if !dir.exists() {
            log::info!("Prompt preset seed directory not found: {}", dir.display());
            continue;
        }

        let existing_ids: std::collections::HashSet<String> = storage
            .list(preset_type)?
            .into_iter()
            .map(|p| p.id)
            .collect();

        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            let content = std::fs::read_to_string(&path)?;
            let seed: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
                crate::error::EngineError::Parse(format!(
                    "Invalid preset seed {}: {e}",
                    path.display()
                ))
            })?;

            let id = seed["id"].as_str().unwrap_or("default").to_string();
            if existing_ids.contains(&id) {
                continue;
            }

            let preset = PromptPreset {
                id: id.clone(),
                name: seed["name"].as_str().unwrap_or("Default").to_string(),
                prompt_text: seed["prompt_text"].as_str().unwrap_or("").to_string(),
                is_default: true,
                preset_type,
            };
            storage.save(&preset)?;
            log::info!("Seeded {} prompt preset: {}", preset_type.as_str(), id);
        }
    }

    Ok(())
}
